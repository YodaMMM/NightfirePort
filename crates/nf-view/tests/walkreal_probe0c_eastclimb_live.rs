//! walkreal probe #22 (level-0c-gate-band-crossing): the east climb map. Down-rays y 12->2 at
//! 0.5 over x 60..76, z −213..−203 — every floor from the pit to the east band, looking for the
//! slope/stair that connects the ground route (path15's 6.57 nodes) to the east band (10.34).
//! Diagnostic only, --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_east_climb() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(30);
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("=== east climb map (y 12->2, 0.5 grid, x 60..76 z -213..-203) ===");
    eprintln!("        x: {}", (0..33).map(|i| format!("{:>6}", 60.0 + 0.5 * i as f32)).collect::<String>());
    for zi in 0..21 {
        let z = -213.0f32 + 0.5 * zi as f32;
        let mut row = format!("z {z:7.1}: ");
        for i in 0..33 {
            let x = 60.0f32 + 0.5 * i as f32;
            match sim.world.segment_hit([x, 12.0, z], [x, 2.0, z]) {
                Some((tt, _)) => row.push_str(&format!("{:5.2} ", 12.0 - 10.0 * tt)),
                None => row.push_str("   -- "),
            }
        }
        eprintln!("{row}");
    }
    eprintln!("=== the same, x 42..60 (the pit west half) ===");
    eprintln!("        x: {}", (0..37).map(|i| format!("{:>6}", 42.0 + 0.5 * i as f32)).collect::<String>());
    for zi in 0..21 {
        let z = -213.0f32 + 0.5 * zi as f32;
        let mut row = format!("z {z:7.1}: ");
        for i in 0..37 {
            let x = 42.0f32 + 0.5 * i as f32;
            match sim.world.segment_hit([x, 12.0, z], [x, 2.0, z]) {
                Some((tt, _)) => row.push_str(&format!("{:5.2} ", 12.0 - 10.0 * tt)),
                None => row.push_str("   -- "),
            }
        }
        eprintln!("{row}");
    }
    eprintln!("ok");
}
