//! perf-240e: a before/after identity probe for sim-side speed-ups. Per level, N ticks of a fixed
//! scripted input (walk, turn, strafe, fire) from the level start; after every tick the full
//! `Debug` text of every drone (shortest round-trip floats: bit-exact for finite values), Bond's
//! position/yaw/health and the shared RNG state are folded into one FNV-1a 64 stream, and every
//! 60 ticks the GPU palette (bones + colours) is hashed at a fixed view. Run before and after a
//! change and diff the `[state-hash]` lines: any divergence in the sim or the drawn pose shows.
//!
//! `#[ignore]`, opt-in: `NF_ISO`, levels from `NF_HASH_LEVELS` (default 07000007,0700000a),
//! ticks from `NF_HASH_TICKS` (default 1200).

use nf_view::scene::ViewOpts;

fn fnv(h: &mut u64, bytes: &[u8]) { for &b in bytes { *h ^= b as u64; *h = h.wrapping_mul(0x1000_0000_01b3); } }

fn input_at(t: u32) -> (nf_game::player::Input, bool) {
    let mut i = nf_game::player::Input::default();
    let phase = (t / 90) % 6;
    match phase {
        0 => { i.forward = 1.0; }
        1 => { i.forward = 1.0; i.turn = 0.6; }
        2 => { i.strafe = 1.0; i.forward = 0.5; }
        3 => { i.turn = -0.8; }
        4 => { i.forward = -1.0; i.strafe = -0.5; }
        _ => { i.forward = 1.0; i.turn = 0.3; }
    }
    (i, t % 37 < 4 && t > 200)
}

#[test]
#[ignore]
fn perf_state_hash_per_level() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let levels = std::env::var("NF_HASH_LEVELS").unwrap_or("07000007,0700000a".into());
    let ticks: u32 = std::env::var("NF_HASH_TICKS").ok().and_then(|v| v.parse().ok()).unwrap_or(1200);
    for lv in levels.split(',') {
        let name = format!("{}.bin", lv.trim());
        let mut scene = match nf_view::load(&iso, &name, false) { Ok(s) => s, Err(e) => { eprintln!("[state-hash] {name} not loadable: {e}"); continue } };
        let (layout, _) = scene.skin_layout();
        if let Some(sim) = scene.sim.as_ref() { eprintln!("[grid] {name} {}", sim.world.grid_stats()); }
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        let mut pal_h = 0xcbf2_9ce4_8422_2325u64;
        let t0 = std::time::Instant::now();
        for t in 0..ticks {
            let (inp, fire) = input_at(t);
            let Some(sim) = scene.sim.as_mut() else { break };
            sim.step_bond(inp, fire, false, false);
            let sim = scene.sim.as_ref().unwrap();
            let d = &sim.drones;
            fnv(&mut h, &d.frame.to_le_bytes());
            fnv(&mut h, &d.rng.x.to_le_bytes());
            fnv(&mut h, &d.rng.y.to_le_bytes());
            fnv(&mut h, &d.rng.draws.to_le_bytes());
            for c in sim.bond.pos { fnv(&mut h, &c.to_bits().to_le_bytes()); }
            fnv(&mut h, &sim.bond.yaw.to_bits().to_le_bytes());
            fnv(&mut h, &d.health.health.to_bits().to_le_bytes());
            fnv(&mut h, format!("{:?}", d.list).as_bytes());
            if t % 60 == 59 {
                let (mut p, mut c) = (Vec::new(), Vec::new());
                scene.skin_palette_into(&layout, None, t as f32, ViewOpts::default(), None, &mut p, &mut c);
                fnv(&mut pal_h, bytemuck::cast_slice(&p));
                fnv(&mut pal_h, bytemuck::cast_slice(&c));
                eprintln!("[state-hash] {name} t{:04} state {h:016x} palette {pal_h:016x}", t + 1);
            }
        }
        let sim = scene.sim.as_ref();
        eprintln!("[state-hash] {name} final state {h:016x} palette {pal_h:016x} drones {} bond {:?} ({:.1} s)",
            sim.map_or(0, |s| s.drones.list.len()), sim.map(|s| s.bond.pos), t0.elapsed().as_secs_f64());
    }
}
