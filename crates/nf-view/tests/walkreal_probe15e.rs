//! probe 07000015's #526 shot geometry (level-15-agent-doors): which stand marks give the
//! bullet a clear flight to the ankle beam (11.3, 40.70, 71.18), east and west of the rim.
//! Teleport probe (assertion-free diagnostic; never runs in CI) — run --ignored --nocapture.

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe_526_shot_lines() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000015.bin", false, None, 2).unwrap());
    w.wait_enabled(3000, "the silo arrival");
    w.wait(60);
    w.scene.sim.as_mut().unwrap().drones.hold_npcs = true;
    let mine = [11.3f32, 40.70, 71.18];
    let marks: &[(&str, [f32; 3])] = &[
        ("east 14.5 z70.3", [14.5, 41.4, 70.3]),
        ("east 15.5 z70.3", [15.5, 41.4, 70.3]),
        ("east 16.5 z70.5", [16.5, 41.4, 70.5]),
        ("east 17.5 z70.6", [17.5, 41.4, 70.6]),
        ("east 14.5 z72.1", [14.5, 41.4, 72.1]),
        ("east 15.5 z72.1", [15.5, 41.4, 72.1]),
        ("east 16.5 z71.9", [16.5, 41.4, 71.9]),
        ("east 17.5 z71.8", [17.5, 41.4, 71.8]),
        ("east 14.5 z69.5", [14.5, 41.4, 69.5]),
        ("east 16.5 z69.6", [16.5, 41.4, 69.6]),
        ("east 14.5 crouch", [14.5, 41.4, 71.2]),
        ("east 12.8 (v1)", [12.8, 41.4, 71.2]),
    ];
    for (label, at) in marks {
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond.pos = *at;
            sim.bond.vel = [0.0; 3];
        }
        w.wait(6);
        let sim = w.scene.sim.as_ref().unwrap();
        let bp = sim.bond.pos;
        let eye = [bp[0], bp[1] + sim.bond.eye - sim.bond.crouch_drop, bp[2]];
        let raw = sim.world.segment(eye, mine);
        let bul = sim.world.segment_bullet(eye, mine);
        let d = ((bp[0] - mine[0]).powi(2) + (bp[1] - mine[1]).powi(2) + (bp[2] - mine[2]).powi(2)).sqrt();
        eprintln!("[p526] {label}: bond ({:.2},{:.2},{:.2}) eye_y {:.2} d {d:.2} raw {raw:?} bullet {bul:?}",
            bp[0], bp[1], bp[2], eye[1]);
    }
}
