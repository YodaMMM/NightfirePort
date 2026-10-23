//! Probe: the pit blast rooms' collision layout — tri y-bands and
//! xz spans, plus the blocking polys, so the survive-the-pulse shelter is pinned. Opt-in via NF_ISO.
use nf_formats::{disc, iso9660::IsoImage, level::Level, levelbin::LevelBin};

fn bands(name: &str, tris: &[[[f32; 3]; 3]]) {
    println!("== {name}: {} tris", tris.len());
    let mut bs: Vec<(f32, f32, usize, [f32; 2], [f32; 2])> = Vec::new();
    for t in tris {
        let ylo = t.iter().map(|v| v[1]).fold(f32::MAX, f32::min);
        let yhi = t.iter().map(|v| v[1]).fold(-f32::MAX, f32::max);
        let xlo = t.iter().map(|v| v[0]).fold(f32::MAX, f32::min);
        let xhi = t.iter().map(|v| v[0]).fold(-f32::MAX, f32::max);
        let zlo = t.iter().map(|v| v[2]).fold(f32::MAX, f32::min);
        let zhi = t.iter().map(|v| v[2]).fold(-f32::MAX, f32::max);
        match bs.iter_mut().find(|(a, b, _, _, _)| ylo >= *a - 0.06 && ylo <= *b + 0.06) {
            Some((_, _, n, xl, zl)) => { *n += 1; xl[0] = xl[0].min(xlo); xl[1] = xl[1].max(xhi); zl[0] = zl[0].min(zlo); zl[1] = zl[1].max(zhi); }
            None => bs.push((ylo, yhi, 1, [xlo, xhi], [zlo, zhi])),
        }
    }
    bs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    for (ylo, yhi, n, x, z) in &bs {
        println!("   y {ylo:.2}..{yhi:.2}: {n} tris x {x:.2?} z {z:.2?}");
    }
}

fn main() {
    let Ok(iso_path) = std::env::var("NF_ISO") else { eprintln!("NF_ISO not set"); return };
    let iso = IsoImage::open(&iso_path).unwrap();
    let (entries, bin) = disc::files_bin_entries(&iso).unwrap();
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000016.bin")).expect("07000016.bin");
    let data = iso.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).unwrap();
    let _ = LevelBin::parse(&data).unwrap();
    let level = Level::parse(&data).expect("level parses");
    let mi = level.main_map().unwrap();
    let m = &level.maps[mi];
    for oi in [100usize, 101, 102, 103, 117, 118] {
        let o = &m.objects[oi];
        println!("== obj[{oi}] {} hash {:08x}", o.name, o.hashcode);
        if let Some(c) = o.collision.as_ref() { bands(&format!("obj[{oi}] {}", o.name), &c.triangles()); }
    }
}
