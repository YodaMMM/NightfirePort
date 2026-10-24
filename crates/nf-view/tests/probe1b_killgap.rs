//! level-1b-drake-4 probe: the walker-kill vs raw-kill ch-0x0e gap. Runs the SAME Drake fight
//! twice in one binary — (A) the walker path (Walker::new, the arrival skipped, w.shoot) and
//! (B) the raw step_bond path (the probe1b_blockers::probe_drake_death shape, the arrival played
//! out) — dumping Drake's mode/state/health/death_channel/corpse and the channels around the
//! death so the divergence names itself. Assertion-free, run with --ignored --nocapture.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn p3(p: [f32; 3]) -> String { format!("({:.2},{:.2},{:.2})", p[0], p[1], p[2]) }

fn dump(sim: &nf_view::scene::Sim, i: usize, tag: &str) {
    let d = &sim.drones.list[i];
    eprintln!("[{tag}] drake mode={:#x} state={:#x} hp={:.1} death_ch={} special={:#x} corpse={} enabled={} pos {}",
        d.mode, d.state, d.health, d.death_channel, d.special_death, d.corpse, d.enabled, p3(d.pos));
    eprintln!("[{tag}] ch0e {} ch0b {} ch100 {} mission_status {}",
        sim.drones.channels.on(0x0e) as u8, sim.drones.channels.on(0x0b) as u8,
        sim.drones.channels.on(100) as u8, sim.drones.mission.status());
}

fn spawn_drake(sim: &mut nf_view::scene::Sim) -> usize {
    let frame = sim.drones.frame;
    sim.drones.channels.set(0x0a, 1, frame);
    for _ in 0..900u32 {
        sim.step_bond(Input::default(), false, false, false);
        if let Some(i) = sim.drones.list.iter().position(|d| d.present && !d.corpse && d.skin & 0xffff == 0xba) {
            return i;
        }
    }
    panic!("Drake never spawned");
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe_kill_gap_a_b() {
    let iso = iso();
    // ── A: the walker path (the ignored test's shape) ─────────────────────────────────────────
    {
        let mut w = Walker::new(nf_view::load(&iso, "0700001b.bin", false).unwrap());
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.disabled = false;
            sim.bond.substate = nf_game::player::SS_WALK;
        }
        w.wait(3000);
        w.skip_cutscenes = false;
        let i = {
            let sim = w.scene.sim.as_mut().unwrap();
            spawn_drake(sim)
        };
        eprintln!("== A (walker): Drake spawned idx {i} ==");
        dump(w.scene.sim.as_ref().unwrap(), i, "A spawn");
        w.scene.sim.as_mut().unwrap().bond.pos = [-4.0, -1.9, 0.2];
        w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
        w.wait(30);
        // The walker's shoot with per-100-tick dumps via the public pieces: run shoot in a thread? No —
        // replicate its inner loop shape manually here with the same step_bond calls.
        let killed = w.shoot(i, "Drake");
        eprintln!("== A: w.shoot returned {killed} ==");
        for _ in 0..200 { w.scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
        dump(w.scene.sim.as_ref().unwrap(), i, "A after");
    }
    // ── B: the raw step_bond path (the probe_drake_death shape) ────────────────────────────────
    {
        let mut scene = nf_view::load(&iso, "0700001b.bin", false).unwrap();
        let sim = scene.sim.as_mut().unwrap();
        sim.disabled = false;
        for _ in 0..2 { sim.step_bond(Input::default(), false, false, false); }
        sim.bond.substate = nf_game::player::SS_WALK;
        for _ in 0..3000 { sim.step_bond(Input::default(), false, false, false); }
        let i = spawn_drake(sim);
        eprintln!("== B (raw): Drake spawned idx {i} ==");
        dump(sim, i, "B spawn");
        sim.bond.pos = [-4.0, -1.9, 0.2];
        sim.bond.vel = [0.0; 3];
        for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
        let mut killed = false;
        for t in 0..2400u32 {
            let dp = sim.drones.list[i].pos;
            let bp = sim.bond.pos;
            let dx = dp[0] - bp[0]; let dz = dp[2] - bp[2];
            let dy = dp[1] + 0.3 - (bp[1] + sim.bond.eye);
            let desired = dx.atan2(dz);
            let err = desired - sim.bond.yaw;
            let want_pitch = (dy / dxz(bp, dp)).atan() / std::f32::consts::FRAC_PI_2;
            let perr = want_pitch - sim.bond.pitch;
            let on = err.abs() < 0.03 && perr.abs() < 0.04 && t % 16 == 0;
            sim.step_bond(Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() }, on, false, false);
            if t % 100 == 0 { dump(sim, i, &format!("B t{t}")); }
            if sim.drones.list[i].corpse || sim.drones.list[i].health <= 0.0 { killed = true; break; }
        }
        eprintln!("== B: killed {killed} ==");
        for _ in 0..200 { sim.step_bond(Input::default(), false, false, false); }
        dump(sim, i, "B after");
    }
}

fn wrap_pi(a: f32) -> f32 { a - (a / std::f32::consts::PI).round() * std::f32::consts::PI }
fn dxz(a: [f32; 3], b: [f32; 3]) -> f32 { ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() }
