//! level-4a-lift-3 probe c: the shaft column's collision around the live kill at y -95.80 —
//! floor_below + the nearest walls at (1.8, y, -3.45) for the whole descent, plus the same for
//! the platform's rider spot (4.26, y, -4.76). Assertion-free.

#[path = "common/walker.rs"]
mod walker;
use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
#[ignore = "assertion-free dump; run with --ignored --nocapture"]
fn dump_shaft_column_collision() {
    let iso = iso();
    let w = Walker::new(nf_view::load(&iso, "0700004a.bin", false).unwrap());
    let sim = w.scene.sim.as_ref().unwrap();
    for y in [-55.0f32] {
        for (tag, x, z) in [("s-1.0", 2.5f32, -1.0f32), ("s-1.4", 2.5, -1.4), ("s-1.7", 2.5, -1.7), ("s-2.0", 2.5, -2.0), ("s-2.5", 2.5, -2.5), ("s-3.0", 2.5, -3.0), ("s-4.0", 2.5, -4.0), ("s-5.0", 2.5, -5.0), ("e-2.0", 4.6, -2.0), ("e-3.0", 4.6, -3.0), ("e-4.0", 4.6, -4.0), ("w-2.0", 0.5, -2.0), ("w-3.0", 0.5, -3.0), ("w0", 0.5, 0.5), ("e6", 6.0, 0.5), ("n2", 2.5, 2.0), ("n3", 2.5, 3.0)] {
            let f = sim.world.floor_below([x, y, z], 8.0);
            let cu = sim.world.floor_below([x, y + 8.0, z], 8.0);
            println!("y{y:>7} {tag:7} ({x},{z}) floor_below8 {:.3?} floor_above8 {:.3?}",
                f, cu);
        }
    }
}
