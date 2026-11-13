//! Scratch probe (level-0b-files-plate-hazard, run 2): WHERE does the camera sight latch on the
//! walk's exec-floor lane? The real-damage walks die to gt #291 (the terminal-room mouth gun,
//! p4 = 28 = ch 0x1C ← OR #289 [0x27, 0x28] = the two cameras' outs, p5 = 78 = 0x4E = the files
//! take's disable [D audit; M 0x180498 case 0]). This probe replays the walk's own legs (the
//! walker's try_walk_to, door handling included) from the ring's east side in short pieces and
//! prints the leg where ch 0x1C / 0x27 / 0x28 rise (the sight latch) and the turrets' states.

#[path = "common/walker.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn ch_line(sim: &nf_view::scene::Sim) -> String {
    format!("0x1C {} 0x27 {} 0x28 {} 0x0E {}", sim.drones.channels.on(0x1c), sim.drones.channels.on(0x27), sim.drones.channels.on(0x28), sim.drones.channels.on(0x0e))
}

fn gt_line(sim: &nf_view::scene::Sim) -> String {
    sim.drones.gt_turrets.iter().enumerate()
        .map(|(i, t)| format!("gt{i}/#{}/s{}", t.placement, t.state))
        .collect::<Vec<_>>().join(" ")
}

#[test]
fn probe_camera_latch_spot() {
    let mut w = Walker::new(nf_view::load(&iso(), "0700000b.bin", false).expect("load 0b"));
    w.wait(30);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [10.21, 47.3, 8.20];
        sim.bond.vel = [0.0; 3];
    }
    for (label, x, y, z, r) in [
        ("ring n12", 6.54f32, 47.3, 7.03, 1.0),
        ("ring n13", 2.18, 47.3, 7.53, 1.0),
        ("mouth north", -1.05, 47.4, -1.6, 1.0),
        ("terminal room", -1.0, 47.5, -5.5, 1.2),
        ("dome lane mid", -1.1, 47.5, -9.0, 1.0),
        ("the files plate", -1.15, 47.5, -12.6, 1.0),
    ] {
        let t0 = w.total_ticks;
        let ok = w.try_walk_to(Wp::new(x, y, z).r(r), label);
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[latch] leg {label} ok {ok} ticks {} at ({:.2},{:.2},{:.2}) {} | {}",
            w.total_ticks - t0, sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2], ch_line(sim), gt_line(sim));
        if sim.drones.channels.on(0x1c) { eprintln!("[latch] ch 0x1C latched during leg {label}"); break; }
    }
}
