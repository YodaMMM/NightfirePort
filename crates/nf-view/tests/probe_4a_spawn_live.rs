//! probe (level-4a-every-objective): why does the floor spawner #233 not release with the
//! aside-park (z −12.4) any more? Prints, every 60 ticks through the −40 pass: Bond's pose,
//! the spawner's arm/latch/wave state, ch 0x17, and per-template the release-screen legs —
//! the 2-D min-dist [M 0x137cb0:30] and the three legs of `drone_position_visible` [M 0x13a7a0]
//! (in_view frustum, the bullet-ray clear set, the actor ray). Opt-in via NF_ISO.

#[path = "common/walker.rs"]
mod walker;
use nf_game::player::Input;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn key_y(y0: f32, t: u32) -> f32 {
    (y0 - ((t as f32 - 90.0).max(0.0)) * (150.0 / 4950.0)).max(y0 - 150.0)
}

#[test]
fn probe_spawner_gate_legs_at_the_aside_park() {
    let iso = iso();
    let scene = nf_view::load(&iso, "0700004a.bin", false).unwrap();
    let mut w = Walker::new(scene);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond = nf_game::player::Bond::stand_at(&sim.world, [3.0, -7.0, -3.4], 0.0);
    }
    let y0 = w.scene.sim.as_ref().unwrap().bond.pos[1];
    let park_z = std::env::var("PROBE_Z").ok().and_then(|s| s.parse::<f32>().ok()).unwrap_or(-3.4);
    for t in 0..2600u32 {
        if t % 30 == 0 {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [3.0, key_y(y0, t) + 1.2, park_z];
            sim.drones.health.health = 100.0;
        }
        w.tick(Input::default(), false);
        if t % 60 != 0 { continue; }
        let sim = w.scene.sim.as_ref().unwrap();
        let Some(sp) = sim.drones.spawners.iter().find(|s| s.placement == 233) else { continue };
        let b = &sim.bond;
        let mut line = format!("[probe] t{t} bond ({:.2},{:.2},{:.2}) gnd {} ch17 {} sp active {} imm {} wave {} spawned {} tmpl {}",
            b.pos[0], b.pos[1], b.pos[2], b.grounded, sim.drones.channels.on(0x17),
            sp.active, sp.immediate, sp.wave, sp.spawned.len(), sp.templates.len());
        if t % 300 == 0 {
            for (k, &ti) in sp.templates.iter().enumerate() {
                let dr = &sim.drones.list[ti];
                let q = dr.pos;
                let dd = ((q[0] - b.pos[0]).powi(2) + (q[2] - b.pos[2]).powi(2)).sqrt();
                let eye = [b.pos[0], b.pos[1] + b.eye, b.pos[2]];
                let bullet_clear = [q, [q[0], q[1] - 0.8, q[2]]].iter()
                    .map(|p| w.scene.sim.as_ref().unwrap().world.segment_bullet(eye, *p).is_none())
                    .collect::<Vec<_>>();
                let actor_clear = w.scene.sim.as_ref().unwrap().world.segment_actor(eye, q).is_none();
                line.push_str(&format!(" | t{k} ({:.2},{:.2},{:.2}) d{dd:.1} rays{bullet_clear:?} actor{actor_clear}", q[0], q[1], q[2]));
            }
        }
        eprintln!("{line}");
        if !sp.spawned.is_empty() { eprintln!("[probe] SPAWNED at t{t}: {:?}", sp.spawned); break; }
    }
}
