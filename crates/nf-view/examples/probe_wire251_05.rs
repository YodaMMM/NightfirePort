//! Where does wire #251 run, and what stands near zip door #112? Instance matrices are the
//! world transforms the sim itself builds [D level.instances]. Scratch print, not committed.

use nf_formats::level::Level;

fn main() {
    let Some(iso_path) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let iso = nf_formats::iso9660::IsoImage::open(&iso_path).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&iso).expect("FILES.BIN");
    let e = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000005.bin")).expect("07000005.bin");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64).expect("read");
    let level = Level::parse(&data).expect("parse 07000005");
    let (instances, _missing) = level.instances(true);

    let by_hash = |h: u32| level.by_hash.get(&h).copied();
    let main_i = level.main_map().expect("main map");
    for (i, p) in level.maps[main_i].placements.iter().enumerate() {
        if i != 251 && i != 112 { continue; }
        let (m, o) = if p.by_hash() {
            by_hash(p.hashcode).unwrap_or((main_i, usize::MAX))
        } else { (main_i, p.index as usize) };
        let om = &level.maps[m].objects[o];
        println!("== placement #{i} id {} object {} flags {:#x} ==", p.index, om.name.trim(), p.flags);
        println!("   translation {:.2} {:.2} {:.2}  rot {:?} params {:?}", p.translation[0], p.translation[1], p.translation[2], p.rotation, p.params);
        let class = p.flags & 0xff;
        let mat = nf_formats::placement::object_matrix_for_class(class as u32, p.creates_cel(), p.rotation, p.translation, p.quaternion());
        if let Some(c) = &om.collision {
            let mut lo = [f32::MAX; 3]; let mut hi = [f32::MIN; 3];
            for (t, _) in c.triangles_with_material() {
                for v in t { for k in 0..3 { lo[k] = lo[k].min(v[k]); hi[k] = hi[k].max(v[k]); } }
            }
            println!("   obj tris {} bounds lo {:.2} {:.2} {:.2} hi {:.2} {:.2} {:.2}",
                c.triangles_with_material().len(), lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]);
            for sz in [[lo[0], lo[1], lo[2]], [hi[0], lo[1], lo[2]], [lo[0], hi[1], lo[2]], [hi[0], hi[1], lo[2]],
                       [lo[0], lo[1], hi[2]], [hi[0], lo[1], hi[2]], [lo[0], hi[1], hi[2]], [hi[0], hi[1], hi[2]]] {
                let w = nf_formats::placement::transform(&mat, sz);
                println!("   corner ({:+.2},{:+.2},{:+.2}) -> world ({:.2},{:.2},{:.2})", sz[0], sz[1], sz[2], w[0], w[1], w[2]);
            }
        } else {
            println!("   no collision");
        }
    }

    println!("\n== every placement within 25 u of the zip door (37.25, 49, −69) ==");
    for (i, p) in level.maps[main_i].placements.iter().enumerate() {
        let t = p.translation;
        let d = (t[0] - 37.25).powi(2) + (t[1] - 49.0).powi(2) + (t[2] + 69.0).powi(2);
        if d > 625.0 { continue; }
        let (m, o) = if p.by_hash() {
            match by_hash(p.hashcode) { Some(x) => x, None => { println!("pl #{i} <unresolved {:08x}> ({:.2},{:.2},{:.2})", p.hashcode, t[0], t[1], t[2]); continue; } }
        } else { (main_i, p.index as usize) };
        let om = &level.maps[m].objects[o];
        println!("pl #{i} id {} {} ({:.2},{:.2},{:.2})", p.index, om.name.trim(), t[0], t[1], t[2]);
    }
}
