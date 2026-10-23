//! level-16-jetpack-escape-3 probe: the #309 hurt volume's per-band quad layout in WORLD coords
//! (placement pos (2.30,−22.70,−164.17)) — where exactly the 18 quads of each band sit, so gaps
//! (safe spots) are visible. Opt-in via NF_ISO.
use nf_formats::{disc, iso9660::IsoImage, level::Level, levelbin::LevelBin};

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
    // hurt placement #309 = place[309] → object index 182.
    let o = &m.objects[182];
    let c = o.collision.as_ref().expect("hurt collision");
    let tris = c.triangles();
    println!("{} tris", tris.len());
    // bucket by band (local y), print each quad's world xz (pos + local).
    let (px, py, pz) = (2.30f32, -22.70f32, -164.17f32);
    let mut bands: Vec<(f32, Vec<[[f32; 2]; 4]>)> = Vec::new();
    for t in tris.chunks(2) {
        let y = t[0][0][1];
        let e = bands.iter_mut().find(|(b, _)| (y - *b).abs() < 0.05);
        let quad = [[t[0][0][0] + px, t[0][0][2] + pz], [t[0][1][0] + px, t[0][1][2] + pz], [t[0][2][0] + px, t[0][2][2] + pz], [t[1][2][0] + px, t[1][2][2] + pz]];
        match e { Some((_, q)) => q.push(quad), None => bands.push((y, vec![quad])) }
    }
    bands.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    for (y, quads) in bands.iter() {
        let wy = y + py;
        println!("== band local y {y:.2} (world {wy:.2}): {} quads", quads.len());
        for q in quads {
            println!("   quad x {:.1}..{:.1}  z {:.1}..{:.1}", q.iter().map(|p| p[0]).fold(f32::MAX, f32::min), q.iter().map(|p| p[0]).fold(-f32::MAX, f32::max), q.iter().map(|p| p[1]).fold(f32::MAX, f32::min), q.iter().map(|p| p[1]).fold(-f32::MAX, f32::max));
        }
    }
}
