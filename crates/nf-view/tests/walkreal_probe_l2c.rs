//! quick: horizontal rays at the hand-back level — do shaft walls exist at (1.8,-6.83,-107)?
#[path = "common/walker.rs"]
mod walker;
use walker::Walker;
fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }
#[test]
fn probe_shaft_walls() {
    let mut w = Walker::new(nf_view::load(&iso(), "0700004a.bin", false).unwrap());
    for _ in 0..5 { w.tick(Default::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    let c = [1.8f32, -107.66, -6.83];
    eprintln!("== horizontal rays from the composed hand-back (1.8,-107.66,-6.83), 30 units ==");
    for (label, d) in [("+x", [1.0, 0.0, 0.0]), ("-x", [-1.0, 0.0, 0.0]), ("+z", [0.0, 0.0, 1.0]), ("-z", [0.0, 0.0, -1.0])] {
        let to = [c[0] + d[0] * 30.0, c[1], c[2] + d[2] * 30.0];
        eprintln!("  {label}: {:?}", sim.world.segment(c, to));
    }
    eprintln!("== horizontal rays at the guards' level (-103, z -0.58) for comparison ==");
    let g = [-4.34f32, -103.0, -0.58];
    for (label, d) in [("+x", [1.0, 0.0, 0.0]), ("-x", [-1.0, 0.0, 0.0]), ("+z", [0.0, 0.0, 1.0]), ("-z", [0.0, 0.0, -1.0])] {
        let to = [g[0] + d[0] * 30.0, g[1], g[2] + d[2] * 30.0];
        eprintln!("  {label}: {:?}", sim.world.segment(g, to));
    }
    // and at the boarding level (-53.76, the parked cab's level): the cab is dynamic so use t0.
    eprintln!("== vertical rays just outside the walkway edge (z -0.5 vs z -1.5) at y -103 ==");
    for z in [-1.5f32, -1.0, -0.5, 0.0, 0.5] {
        eprintln!("  x -4.34 z {z}: {:?}", sim.world.floor_below([-4.34, -101.0, z], 8.0));
    }
}
