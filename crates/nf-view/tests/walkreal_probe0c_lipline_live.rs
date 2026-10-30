//! walkreal probe #19 (level-0c-gate-band-crossing): ALL disc tris in the strip-lip box
//! x 52..57, z −207.2..−204.5, y 9..16 — the wall that stalls the run at x 53.02.
//! Diagnostic only, --nocapture.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::placement as plc;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_strip_lip_blockers() {
    let isopath = iso();
    let img = IsoImage::open(&isopath).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000c.bin")).expect("0700000c.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let level = Level::parse(&data).expect("level");
    let main = level.main_map().expect("main map");

    for (mi, m) in level.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() } else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
            let Some((om, oi)) = target else { continue };
            let Some(col) = level.maps[om].objects[oi].collision.as_ref() else { continue };
            let matrix = plc::object_matrix_for_class(p.flags & 0xff, p.creates_cel(), p.rotation, p.translation, p.quaternion());
            for t in col.triangles() {
                let w: Vec<[f32; 3]> = t.iter().map(|&v| plc::transform(&matrix, v)).collect();
                let cx = (w[0][0] + w[1][0] + w[2][0]) / 3.0;
                let cy = (w[0][1] + w[1][1] + w[2][1]) / 3.0;
                let cz = (w[0][2] + w[1][2] + w[2][2]) / 3.0;
                if !(52.0..=57.0).contains(&cx) || !(-207.2..=-204.5).contains(&cz) || !(9.0..=16.0).contains(&cy) { continue; }
                let e1 = [w[1][0] - w[0][0], w[1][1] - w[0][1], w[1][2] - w[0][2]];
                let e2 = [w[2][0] - w[0][0], w[2][1] - w[0][1], w[2][2] - w[0][2]];
                let n = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
                let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt().max(1e-9);
                eprintln!("\"{}\" plc{pi} tri ({:.2},{:.2},{:.2})|({:.2},{:.2},{:.2})|({:.2},{:.2},{:.2}) n ({:.2},{:.2},{:.2})",
                    level.maps[om].objects[oi].name.trim(),
                    w[0][0], w[0][1], w[0][2], w[1][0], w[1][1], w[1][2], w[2][0], w[2][1], w[2][2], n[0] / l, n[1] / l, n[2] / l);
            }
        }
    }
    eprintln!("ok");
}
