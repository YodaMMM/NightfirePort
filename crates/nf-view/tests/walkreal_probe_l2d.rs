//! quick: deep-pit scan below the shaft bottom at the hand-back column
#[path = "common/walker.rs"]
mod walker;
use walker::Walker;
fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }
#[test]
fn probe_deep_pit() {
    let mut w = Walker::new(nf_view::load(&iso(), "0700004a.bin", false).unwrap());
    for _ in 0..5 { w.tick(Default::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("== deep scan: floor_below from -155, depth 120 ==");
    for x in [-6f32, -4.34, -2.0, 0.0, 1.8, 3.06, 4.2, 6.0] {
        for z in [-8f32, -6.83, -4.0, -2.0, -1.0, 0.0, 2.0, 4.0] {
            if let Some(f) = sim.world.floor_below([x, -155.0, z], 120.0) {
                eprintln!("  HIT x {x} z {z}: floor {f:.2}");
            }
        }
    }
    eprintln!("== down-ray from the hand-back exactly, depth 400 ==");
    eprintln!("  {:?}", sim.world.floor_below([1.8, -107.0, -6.83], 400.0));
    eprintln!("== the lobby strip's south edge: where does the -152 floor end in z? ==");
    for z in [-2f32, -1.5, -1.0, -0.5, 0.0] {
        eprintln!("  x 3.0 z {z}: {:?}", sim.world.floor_below([3.0, -150.0, z], 12.0));
    }
}
