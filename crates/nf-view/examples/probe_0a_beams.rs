//! level-0a-ring-beams-3 probe: dump the 0a laser beams' seg_beam status, motion keys, plane
//! records and posed triangles — the raw data the crossing re-timing needs. Prints only.
fn main() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut scene = nf_view::load(&iso, "0700000a.bin", false).expect("load 0700000a");
    let sim = scene.sim.as_mut().unwrap();
    let beams: Vec<usize> = vec![607, 614, 621, 625, 651, 652, 653, 654];
    for t in &sim.drones.triggers {
        if beams.contains(&t.placement) || matches!(t.out, 0x1f | 0x20 | 0x21 | 0x22 | 0x24 | 0x25 | 0x26) {
            println!("== placement {} kind {} out {:#04x} mask {:#x} seg_beam {} planes {} tris {}",
                t.placement, t.kind, t.out, t.mask, t.seg_beam, t.planes.len(), t.tris.len());
            println!("   lo {:?} hi {:?}", t.lo, t.hi);
            match &t.motion {
                Some(m) => {
                    println!("   motion: last {} step {} t {}", m.last, m.step, m.t);
                    for (k, key) in m.keys.iter().enumerate() {
                        println!("   key {k}: {:?}", key);
                    }
                    for (i, (n, d)) in m.local_planes.iter().enumerate().take(4) { println!("   local plane {i}: n {n:?} d {d}"); }
                }
                None => println!("   motion: None (static)"),
            }
        }
    }
}
