//! walkreal probe (level-0c-whole-walk-east-route): the fine step scan of the east climb.
//! Down-rays (y 12 -> 0) on a 0.5 grid over x 77..89, z −212.5..−206.5 — the turb4
//! staircase03/02/01 chain (extents [D]: plc37 y 0.16..7.19, plc36 y 6.16..8.13, plc35 y
//! 9.00..11.20, x 78.14..87.59). Diagnostic only, --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_east_climb3() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700000c.bin", false, None).unwrap());
    w.wait_enabled(3000, "the intro");
    w.wait(30);
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("=== east climb fine map (down-rays y 12->0; rows z -212.5..-206.5 step 0.5, cols x 77..89 step 0.5) ===");
    let mut header = String::from("z\\x   ");
    for i in 0..=24 { header.push_str(&format!("{:>6.1}", 77.0 + 0.5 * i as f32)); }
    eprintln!("{}", header);
    for zi in 0..=12 {
        let z = -212.5f32 + 0.5 * zi as f32;
        let mut row = format!("{z:>6.1} ");
        for i in 0..=24 {
            let x = 77.0f32 + 0.5 * i as f32;
            match sim.world.segment_hit([x, 12.0, z], [x, 0.0, z]) {
                Some((t, _)) => row.push_str(&format!("{:>6.2}", 12.0 - 12.0 * t)),
                None => row.push_str("     ."),
            }
        }
        eprintln!("{row}");
    }
    // The band top edge and the gate approach floor, for the resume legs.
    eprintln!("=== the east band + gate approach (down-rays y 13->8; x 87..97 step 0.5 at z -207.5, -209.0, -209.5) ===");
    for z in [-207.5f32, -209.0, -209.5] {
        let mut row = format!("{z:>6.1} ");
        for i in 0..=20 {
            let x = 87.0f32 + 0.5 * i as f32;
            match sim.world.segment_hit([x, 13.0, z], [x, 8.0, z]) {
                Some((t, _)) => row.push_str(&format!("{:>6.2}", 13.0 - 5.0 * t)),
                None => row.push_str("     ."),
            }
        }
        eprintln!("{row}");
    }
    eprintln!("ok");
}
