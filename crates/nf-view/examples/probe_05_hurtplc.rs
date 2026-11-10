//! Scratch (level-05-basin-floor): the disc's hurt (0xE4) and plunge (0xEA) placements plus every
//! placement whose RESOLVED object carries a collision block whose tris (transformed by the
//! placement matrix) meet the east-basin floor band (x 78..172, y 2..9, z 94..148) — the direct
//! disc answer to "where are the valley-floor quads east of the lip".
//! Run: NF_ISO=<iso> cargo run -p nf-view --release --example probe_05_hurtplc

use nf_formats::level::Level;

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = nf_formats::iso9660::IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

const BAND: ([f32; 3], [f32; 3]) = ([78.0, 2.0, 94.0], [172.0, 9.0, 148.0]);

fn main() {
    let Some(iso) = std::env::var("NF_ISO").ok() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000005.bin");
    let lv = Level::parse(&data).expect("parse");
    let main = lv.main_map().unwrap_or(0);

    println!("== hurt 0xE4 / plunge 0xEA placements (main map) ==");
    for (pi, p) in lv.maps[main].placements.iter().enumerate() {
        let kind = p.flags & 0xff;
        if !matches!(kind, 0xe4 | 0xea) { continue; }
        let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() } else { Some((main, p.index as usize)) };
        let obj = target.map(|(m, o)| &lv.maps[m].objects[o]);
        let name = obj.map(|o| o.name.as_str()).unwrap_or("?");
        let tris = obj.and_then(|o| o.collision.as_ref()).map(|c| c.triangles().len());
        let scale = p.scale();
        println!("  plc {pi} kind {kind:#04x} hash {:08x} \"{name}\" @ {:?} scale {:?}", p.hashcode, p.translation, scale);
        println!("      flags {:#010x} params {:?} coll tris {tris:?}", p.flags, p.params);
        if let Some(o) = obj {
            if let Some(c) = &o.collision {
                let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
                let mat = nf_formats::placement::object_matrix(p.rotation, p.translation);
                for t in c.triangles() { for v in &t { let w = nf_formats::placement::transform(&mat, *v); for k in 0..3 { lo[k] = lo[k].min(w[k]); hi[k] = hi[k].max(w[k]); } } }
                println!("      transformed coll bbox {:?}", (lo, hi));
            }
        }
    }

    println!("\n== every placement whose object's transformed collision tris meet the band ==");
    // Walk every map's placements (any kind), resolve the object, transform its collision tris,
    // keep those with at least one vertex inside the band.
    for (mi, m) in lv.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
            let Some((om, oi)) = target else { continue };
            let Some(c) = lv.maps[om].objects[oi].collision.as_ref() else { continue };
            let mat = nf_formats::placement::object_matrix_for_class(p.flags & 0xff, p.creates_cel(), p.rotation, p.translation, p.quaternion());
            let mut hits = 0usize;
            let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
            for t in c.triangles() {
                for v in &t {
                    let w = nf_formats::placement::transform(&mat, *v);
                    for k in 0..3 { lo[k] = lo[k].min(w[k]); hi[k] = hi[k].max(w[k]); }
                    if (BAND.0[0]..=BAND.1[0]).contains(&w[0]) && (BAND.0[1]..=BAND.1[1]).contains(&w[1]) && (BAND.0[2]..=BAND.1[2]).contains(&w[2]) { hits += 1; }
                }
            }
            if hits > 0 {
                let name = &lv.maps[om].objects[oi].name;
                println!("  map {mi} plc {pi} kind {:#04x} flags {:#010x} \"{}\" hash {:08x} tris {} in-band verts {hits} bbox {:?}", p.flags & 0xff, p.flags, name, p.hashcode, c.triangles().len(), (lo, hi));
            }
        }
    }
}
