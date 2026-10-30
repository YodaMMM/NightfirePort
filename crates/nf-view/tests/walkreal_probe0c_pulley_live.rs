//! walkreal probe #14 (level-0c-gate-band-crossing): obj_turb2_pulley's shape — the walkable
//! (up-facing, normal.y > 0.7) tri census by height band, plus its north-end tris near the band's
//! south face. Diagnostic only, --nocapture.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::placement as plc;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_pulley_shape() {
    let isopath = iso();
    let img = IsoImage::open(&isopath).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000c.bin")).expect("0700000c.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let level = Level::parse(&data).expect("level");
    let main = level.main_map().expect("main map");

    for (oi, o) in level.maps[main].objects.iter().enumerate() {
        if !o.name.trim().eq_ignore_ascii_case("obj_turb2_pulley") { continue; }
        let Some(col) = o.collision.as_ref() else { continue };
        let p = &level.maps[main].placements[19];
        let matrix = plc::object_matrix_for_class(p.flags & 0xff, p.creates_cel(), p.rotation, p.translation, p.quaternion());
        let mut bands: std::collections::BTreeMap<i32, usize> = Default::default();
        let mut walk: Vec<String> = Vec::new();
        for t in col.triangles() {
            let w: Vec<[f32; 3]> = t.iter().map(|&v| plc::transform(&matrix, v)).collect();
            let e1 = [w[1][0] - w[0][0], w[1][1] - w[0][1], w[1][2] - w[0][2]];
            let e2 = [w[2][0] - w[0][0], w[2][1] - w[0][1], w[2][2] - w[0][2]];
            let n = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
            let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-9);
            let ny = n[1] / l;
            let cy = (w[0][1] + w[1][1] + w[2][1]) / 3.0;
            *bands.entry((cy * 2.0).round() as i32).or_default() += 1;
            if ny > 0.7 {
                let cx = (w[0][0] + w[1][0] + w[2][0]) / 3.0;
                let cz = (w[0][2] + w[1][2] + w[2][2]) / 3.0;
                let sz = (w[0][2] - w[1][2]).powi(2) + (w[2][2] - w[1][2]).powi(2);
                let sx = (w[0][0] - w[1][0]).powi(2) + (w[2][0] - w[1][0]).powi(2);
                walk.push(format!("y {cy:.2} x {cx:.2} z {cz:.2} size {:.2}", (sx + sz).sqrt().min(9.0)));
            }
        }
        eprintln!("== pulley height bands (0.5, all tris) ==");
        for (k, v) in bands { eprintln!("  y ~{:>5}: {v} tris", k as f32 / 2.0); }
        eprintln!("== walkable tris ({}): ==", walk.len());
        for w in walk.iter().take(60) { eprintln!("  {w}"); }
    }
    eprintln!("ok");
}
