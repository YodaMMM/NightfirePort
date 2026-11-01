//! walkreal probe (level-0c-whole-walk-east-route): the east climb map. Dumps every collision
//! placement whose transformed extents intersect the east box x 55..100, y 5..13, z −203..−215
//! (the east ground 6.17 → the east band 10.34 climb zone), one line per placement, sorted by y.
//! Diagnostic only, --nocapture.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::levelbin::LevelBin;
use nf_formats::placement as plc;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_east_climb2() {
    let isopath = iso();
    let img = IsoImage::open(&isopath).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000c.bin")).expect("0700000c.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let _ = LevelBin::parse(&data).expect("levelbin");
    let level = Level::parse(&data).expect("level");

    // The east climb box.
    let (bx0, bx1, by0, by1, bz0, bz1) = (55.0f32, 100.0f32, 5.0f32, 13.0f32, -215.0f32, -203.0f32);

    let mut rows: Vec<(f32, String)> = Vec::new();
    for (mi, m) in level.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
            let Some((om, oi)) = target else { continue };
            let Some(col) = level.maps[om].objects[oi].collision.as_ref() else { continue };
            let matrix = plc::object_matrix_for_class(p.flags & 0xff, p.creates_cel(), p.rotation, p.translation, p.quaternion());
            let (mut x0, mut x1, mut y0, mut y1, mut z0, mut z1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN, f32::MAX, f32::MIN);
            let mut n = 0usize;
            for tri in col.triangles() {
                for v in tri.iter().map(|&v| plc::transform(&matrix, v)) {
                    x0 = x0.min(v[0]); x1 = x1.max(v[0]); y0 = y0.min(v[1]); y1 = y1.max(v[1]); z0 = z0.min(v[2]); z1 = z1.max(v[2]);
                }
                n += 1;
            }
            if n == 0 { continue; }
            // intersect the box?
            if x1 < bx0 || x0 > bx1 || y1 < by0 || y0 > by1 || z1 < bz0 || z0 > bz1 { continue; }
            let name = level.maps[om].objects[oi].name.trim();
            rows.push((y0, format!("map{mi} plc{pi} \"{}\" class {:#04x} {n}t x {x0:.2}..{x1:.2} y {y0:.2}..{y1:.2} z {z0:.2}..{z1:.2}",
                name, p.flags & 0xff)));
        }
    }
    rows.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    for (_, s) in &rows { eprintln!("{s}"); }
    eprintln!("total {} placements in the east climb box", rows.len());
}
