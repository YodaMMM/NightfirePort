//! walkreal probe (level-0c-secret-door-reach-3): the walkway's south lip. The reach-2 probe hung
//! the body at (13.24, 15.77, −153.65) on the slow walk-off; the [L 2026-10-31] fresh-boot run says
//! the game's Bond "stops at z −153.65, no drop" too. Two crossing candidates from the reach-2
//! report, both on real input: the FULL-SPEED run through the lip, and a GROUNDED jump fired
//! 2–3 ticks before the edge. Diagnostic, --nocapture.
#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn run(label: &str, jump_at: Option<f32>, fwd: f32) {
    let isopath = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&isopath, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(30);
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [13.5, 15.8, -150.0];
        sim.bond.vel = [0.0; 3];
        sim.bond.yaw = std::f32::consts::PI; // face south (+z is north here; PI = -z? set below)
    }
    // Face south: yaw such that forward pushes -z. The walker's yaw convention: 0 = +z? use the
    // walk helper instead — steer to a target 2 past the edge every tick.
    let start = w.total_ticks;
    let mut jumped = false;
    for _ in 0..400u32 {
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        let grounded = w.scene.sim.as_ref().unwrap().bond.grounded;
        if grounded && p[2] < -155.5 && p[1] < 16.0 {
            let under = w.scene.sim.as_ref().unwrap().world
                .segment_hit([p[0], p[1] + 0.5, p[2]], [p[0], p[1] - 4.0, p[2]])
                .map(|(tt, _)| p[1] + 0.5 - 4.0 * tt);
            eprintln!("[lip] {label}: LANDED ({:.2},{:.2},{:.2}) grounded under {under:?} (+{} ticks)",
                p[0], p[1], p[2], w.total_ticks - start);
            return;
        }
        if grounded && w.total_ticks - start > 240 {
            eprintln!("[lip] {label}: STUCK at ({:.2},{:.2},{:.2}) grounded {}",
                p[0], p[1], p[2], grounded);
            return;
        }
        let jump = match jump_at {
            Some(z) => grounded && !jumped && p[2] <= z,
            None => false,
        };
        if jump { jumped = true; eprintln!("[lip] {label}: jump fired at ({:.2},{:.2},{:.2})", p[0], p[1], p[2]); }
        w.tick(Input { forward: fwd, jump, ..Default::default() }, false);
    }
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    eprintln!("[lip] {label}: timeout at ({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
}

#[test]
fn probe_walkway_lip() {
    // 0. the slow walk-off baseline (the reach-2 hang).
    run("slow-walk", None, 0.35);
    // 1. the full-speed run through the lip.
    run("run-through", None, 1.0);
    // 2. the pre-edge jump at z −153.2 (2–3 ticks before the −153.65 edge at run speed).
    run("jump-at--153.2", Some(-153.2), 1.0);
    // 3. the pre-edge jump at z −152.6 (earlier).
    run("jump-at--152.6", Some(-152.6), 1.0);
}
