//! walkreal probe #9 (level-0c-every-objective-blockers): the yard level under the display
//! district — the blocker-3 descent (display ground -> deck 2.4 -> yard 0.0 -> warehouse 0.9).
//! Diagnostic only, run with --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn down(sim: &nf_view::scene::Sim, x: f32, y0: f32, y1: f32, z: f32) -> Option<f32> {
    sim.world.segment_hit([x, y0, z], [x, y1, z]).map(|(t, _)| y0 - (y0 - y1) * t)
}

#[test]
fn probe_blockers3() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(60);
    let sim = w.scene.sim.as_ref().unwrap();

    eprintln!("=== yard level (y 4 -> -0.5) x 0..56 step 2, z -126..-178 step 2 ===");
    eprintln!("        x: {}", (0..29).map(|i| format!("{:>5}", i * 2)).collect::<String>());
    for zi in 0..27 {
        let z = -126.0f32 - zi as f32 * 2.0;
        let mut row = format!("z {z:7.1}: ");
        for i in 0..29 {
            let x = i as f32 * 2.0;
            match down(sim, x, 4.0, -0.5, z) {
                Some(y) => row.push_str(&format!("{y:5.2}")),
                None => row.push_str("   --"),
            }
            row.push(' ');
        }
        eprintln!("{row}");
    }

    // The deck's south end: north-south rays at x 16, y 2.6 (deck centre height) from z -154 -> -162.
    eprintln!("=== deck south rays (x 14/16/18, y 2.6, z -152 -> -164) ===");
    for x in [14.0f32, 16.0, 18.0] {
        for y in [1.2f32, 2.6] {
            let hit = sim.world.segment_hit([x, y, -152.0], [x, y, -164.0]).map(|(t, _)| -152.0 - 12.0 * t);
            eprintln!("x {x:5.1} y {y:4.1}: first face at z {:?}", hit);
        }
    }
    // The display ground's east edge: east-west rays at y 6.8 (ground centre) from x 12 -> 18, z -114..-150.
    eprintln!("=== display-ground east rays (x 12 -> 18) ===");
    for z in [-114.0f32, -120.0, -130.0, -140.0, -150.0] {
        for y in [6.8f32, 4.0, 2.6] {
            let hit = sim.world.segment_hit([12.0, y, z], [18.0, y, z]).map(|(t, _)| 12.0 + 6.0 * t);
            eprintln!("z {z:7.1} y {y:4.1}: first face at x {:?}", hit);
        }
    }
    // The yard -> warehouse belt: floors x 20..60 step 1 at z -166..-178 (the plates approach).
    eprintln!("=== warehouse belt (y 4 -> -0.5) x 20..60 step 2, z -164..-178 step 2 ===");
    eprintln!("        x: {}", (0..21).map(|i| format!("{:>5}", 20 + i * 2)).collect::<String>());
    for zi in 0..8 {
        let z = -164.0f32 - zi as f32 * 2.0;
        let mut row = format!("z {z:7.1}: ");
        for i in 0..21 {
            let x = 20.0f32 + i as f32 * 2.0;
            match down(sim, x, 4.0, -0.5, z) {
                Some(y) => row.push_str(&format!("{y:5.2}")),
                None => row.push_str("   --"),
            }
            row.push(' ');
        }
        eprintln!("{row}");
    }
    eprintln!("ok");
}
