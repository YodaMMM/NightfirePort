//! probe: identify the level object whose walkable rim sits at (93.4, −46.3) on 07000007 —
//! the post-E wedge spot drone-bump-collision-9 diagnosed (capsule rests at y ≈ 43.215 over
//! the 43.0 passage floor). Prints every main-map instance whose world AABB overlaps the spot
//! band with a top face near the floor. Assertion-free, --nocapture.

use nf_formats::level::Level;

fn main() {
    let iso = nf_formats::iso9660::IsoImage::open(&std::env::var("NF_ISO").expect("NF_ISO")).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).expect("FILES.BIN");
    let e = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000007.bin")).expect("07000007.bin");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64).expect("read");
    let level = Level::parse(&data).expect("parse 07000007");
    let (instances, _missing) = level.instances(true);
    let spot = [93.0f32, -47.5f32];
    println!("[rim] {} instances", instances.len());
    for inst in instances.iter() {
        let om = &level.maps[inst.map].objects[inst.object];
        // World AABB from the placement matrix applied to the model bounds' 8 corners.
        let tx = |p: [f32; 3]| -> [f32; 3] { nf_formats::placement::transform(&inst.matrix, p) };
        let mut lo = [f32::MAX; 3];
        let mut hi = [f32::MIN; 3];
        for i in 0..8 {
            let c = [if i & 1 != 0 { om.max[0] } else { om.min[0] }, if i & 2 != 0 { om.max[1] } else { om.min[1] }, if i & 4 != 0 { om.max[2] } else { om.min[2] }];
            let w = tx(c);
            for k in 0..3 { lo[k] = lo[k].min(w[k]); hi[k] = hi[k].max(w[k]); }
        }
        // Overlap on xz with the spot band and a top face near the passage floor 43.0.
        if hi[0] < spot[0] - 2.5 || lo[0] > spot[0] + 2.5 { continue; }
        if hi[2] < spot[1] - 2.5 || lo[2] > spot[1] + 2.5 { continue; }
        if !(42.5..47.5).contains(&hi[1]) { continue; }
        println!("[rim] obj map{} #{} name {:?} hash {:#x} pos ({:.2},{:.2},{:.2})",
            inst.map, inst.object, om.name, om.hashcode,
            inst.matrix[3][0], inst.matrix[3][1], inst.matrix[3][2]);
        println!("[rim]   world AABB x {:.2}..{:.2}  y {:.2}..{:.2}  z {:.2}..{:.2}", lo[0], hi[0], lo[1], hi[1], lo[2], hi[2]);
        println!("[rim]   params {:?} flags {:#x}", inst.placement.params, inst.placement.flags);
    }
}
