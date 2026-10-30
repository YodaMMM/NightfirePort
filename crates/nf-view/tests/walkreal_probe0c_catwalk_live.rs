//! walkreal probe #20 (level-0c-gate-band-crossing): the catwalk access. Dumps (1) every 0x22
//! ladder placement with the scene's parsed ladder extents, (2) every disc tri at y 15.0..16.5
//! over x 40..80 (the full catwalk census with its x extent per object), (3) the catwalk's west
//! end neighbourhood tris (x 40..46, z −208..−204, y 10..17).
//! Diagnostic only, --nocapture.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::placement as plc;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_catwalk_access() {
    let isopath = iso();
    // 1. The scene ladders.
    {
        let mut w = Walker::new(nf_view::load_with_carry(&isopath, "0700000c.bin", false, None).unwrap());
        w.wait_enabled(3000, "the intro");
        w.wait(30);
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("=== ladders (sim) {} ===", sim.ladders.len());
        for (i, l) in sim.ladders.iter().enumerate() {
            eprintln!("ladder{i}: {:?}", l);
        }
    }
    let img = IsoImage::open(&isopath).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000c.bin")).expect("0700000c.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let level = Level::parse(&data).expect("level");
    let main = level.main_map().expect("main map");

    // 2. The catwalk census.
    eprintln!("=== catwalk tris y 15.0..16.5 x 40..80 ===");
    for (mi, m) in level.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() } else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
            let Some((om, oi)) = target else { continue };
            let Some(col) = level.maps[om].objects[oi].collision.as_ref() else { continue };
            let matrix = plc::object_matrix_for_class(p.flags & 0xff, p.creates_cel(), p.rotation, p.translation, p.quaternion());
            let mut n = 0usize;
            let mut xs: Vec<f32> = Vec::new();
            for t in col.triangles() {
                let w: Vec<[f32; 3]> = t.iter().map(|&v| plc::transform(&matrix, v)).collect();
                let cy = (w[0][1] + w[1][1] + w[2][1]) / 3.0;
                if !(15.0..=16.5).contains(&cy) { continue; }
                let cx = (w[0][0] + w[1][0] + w[2][0]) / 3.0;
                if !(40.0..=80.0).contains(&cx) { continue; }
                let e1 = [w[1][0] - w[0][0], w[1][1] - w[0][1], w[1][2] - w[0][2]];
                let e2 = [w[2][0] - w[0][0], w[2][1] - w[0][1], w[2][2] - w[0][2]];
                let nn = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
                let l = (nn[0] * nn[0] + nn[1] * nn[1] + nn[2] * nn[2]).sqrt().max(1e-9);
                if nn[1] / l < 0.5 { continue; } // up-facing only
                n += 1;
                xs.push(cx);
            }
            if n > 0 {
                eprintln!("\"{}\" plc{pi}: {n} up-facing tris, x {:.1}..{:.1}",
                    level.maps[om].objects[oi].name.trim(), xs.iter().cloned().fold(f32::MAX, f32::min), xs.iter().cloned().fold(f32::MIN, f32::max));
            }
        }
    }

    // 3. The catwalk west end tris.
    eprintln!("=== west-end tris x 40..47 z -209..-203 y 10..17 ===");
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
                if !(40.0..=47.0).contains(&cx) || !(-209.0..=-203.0).contains(&cz) || !(10.0..=17.0).contains(&cy) { continue; }
                eprintln!("\"{}\" plc{pi} tri ({:.2},{:.2},{:.2})|({:.2},{:.2},{:.2})|({:.2},{:.2},{:.2})",
                    level.maps[om].objects[oi].name.trim(),
                    w[0][0], w[0][1], w[0][2], w[1][0], w[1][1], w[1][2], w[2][0], w[2][1], w[2][2]);
            }
        }
    }
    eprintln!("ok");
}
