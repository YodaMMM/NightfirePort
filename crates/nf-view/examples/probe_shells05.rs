//! Dump every trigger whose AABB overlaps the strip walk line (−25.2, 31, −10) — the shell
//! that fires ch 0xBE on the walked route — with its tris.
use nf_view::scene::Scene;
fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let scene = nf_view::load(&iso, "07000005.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    let probe = [[-25.2f32, 31.0, -10.0], [-25.2, 30.7, -10.4], [-23.5, 31.2, -12.0]];
    for t in &sim.drones.triggers {
        if t.out != 0xbe { continue; }
        let hit = probe.iter().any(|p| (0..3).all(|k| p[k] >= t.lo[k] && p[k] <= t.hi[k]));
        if !hit { continue; }
        println!("placement #{} kind {:#x} out {:#x} mask {:x}", t.placement, t.kind, t.out, t.mask);
        println!("  lo {:?} hi {:?}", t.lo, t.hi);
        for tri in &t.tris {
            println!("  tri {:?}", tri);
        }
    }
}
