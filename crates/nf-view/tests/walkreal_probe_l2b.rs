//! quick: does the port's static world have any shaft geometry at all?
#[path = "common/walker.rs"]
mod walker;
use walker::Walker;
fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }
#[test]
fn probe_shaft_statics() {
    let mut w = Walker::new(nf_view::load(&iso(), "0700004a.bin", false).unwrap());
    // A few ticks so set_dynamic has run (doors, platform registered).
    for _ in 0..5 { w.tick(Default::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("== guards' column x -4.34 z -0.58 ==");
    for y in [-96f32, -100.0, -104.0] {
        eprintln!("  from {y}: {:?}", sim.world.floor_below([-4.34, y, -0.58], 20.0));
    }
    eprintln!("== shaft cross-section floor scan at y -100 (depth 8) ==");
    for x in [-6f32, -4.0, -2.0, 0.0, 2.0, 4.0, 6.0] {
        for z in [-8f32, -6.0, -4.0, -2.0, 0.0, 2.0] {
            if let Some(f) = sim.world.floor_below([x, -100.0, z], 8.0) {
                eprintln!("  HIT x {x} z {z}: floor {f:.2}");
            }
        }
    }
    eprintln!("== same scan at y -10 (near the top) ==");
    for x in [-6f32, -4.0, -2.0, 0.0, 2.0, 4.0, 6.0] {
        for z in [-8f32, -6.0, -4.0, -2.0, 0.0, 2.0] {
            if let Some(f) = sim.world.floor_below([x, -10.0, z], 8.0) {
                eprintln!("  HIT x {x} z {z}: floor {f:.2}");
            }
        }
    }
    eprintln!("== the lobby bottom: y -150 (depth 12) ==");
    for x in [-6f32, -4.0, -2.0, 0.0, 2.0, 4.0, 6.0] {
        for z in [-8f32, -6.0, -4.0, -2.0, 0.0, 2.0] {
            if let Some(f) = sim.world.floor_below([x, -150.0, z], 12.0) {
                eprintln!("  HIT x {x} z {z}: floor {f:.2}");
            }
        }
    }
}
