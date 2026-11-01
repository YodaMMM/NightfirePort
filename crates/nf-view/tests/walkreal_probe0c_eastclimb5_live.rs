//! walkreal probe (level-0c-east-climb): the FULL east-zone surface census. Three sections:
//! (1) every collision placement intersecting x 38..100, y 8..18, z −236..−202 (the turbine
//! rooms + both bands + the climb zone), sorted by y0 — the placement inventory;
//! (2) per-object up-facing tri levels (normal.y > 0.6, centroid y 9..17, x 38..82) with each
//! level's x/z bbox — stairs show as many levels 0.2..0.6 apart;
//! (3) every placement (any class, collision or not) whose translation sits in x 38..100,
//! z −240..−200 — the actors (triggers, icons, SS, ladders) for the objective-order read.
//! Diagnostic only, --nocapture.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::placement as plc;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_east_climb5() {
    let isopath = iso();
    let img = IsoImage::open(&isopath).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000c.bin")).expect("0700000c.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let level = Level::parse(&data).expect("level");

    // 1. The placement inventory in the east zone.
    eprintln!("=== collision placements x 38..100 y 8..18 z -236..-202 (sorted by y0) ===");
    let (bx0, bx1, by0, by1, bz0, bz1) = (38.0f32, 100.0f32, 8.0f32, 18.0f32, -236.0f32, -202.0f32);
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
            if x1 < bx0 || x0 > bx1 || y1 < by0 || y0 > by1 || z1 < bz0 || z0 > bz1 { continue; }
            let name = level.maps[om].objects[oi].name.trim();
            rows.push((y0, format!("map{mi} plc{pi} \"{}\" class {:#04x} {n}t x {x0:.2}..{x1:.2} y {y0:.2}..{y1:.2} z {z0:.2}..{z1:.2}",
                name, p.flags & 0xff)));
        }
    }
    rows.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    for (_, s) in &rows { eprintln!("{s}"); }
    eprintln!("total {} placements", rows.len());

    // 2. Per-object up-facing levels.
    eprintln!("=== up-facing tri levels y 9..17 x 38..82 (per object; stairs = many close levels) ===");
    let mut objs: Vec<(String, Vec<(f32, f32, f32, f32, f32)>)> = Vec::new(); // (name, (y, x0, x1, z0, z1))
    for (mi, m) in level.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
            let Some((om, oi)) = target else { continue };
            let Some(col) = level.maps[om].objects[oi].collision.as_ref() else { continue };
            let matrix = plc::object_matrix_for_class(p.flags & 0xff, p.creates_cel(), p.rotation, p.translation, p.quaternion());
            let mut levels: Vec<(f32, f32, f32, f32, f32)> = Vec::new();
            for t in col.triangles() {
                let w: Vec<[f32; 3]> = t.iter().map(|&v| plc::transform(&matrix, v)).collect();
                let cx = (w[0][0] + w[1][0] + w[2][0]) / 3.0;
                let cy = (w[0][1] + w[1][1] + w[2][1]) / 3.0;
                let cz = (w[0][2] + w[1][2] + w[2][2]) / 3.0;
                if !(9.0..=17.0).contains(&cy) || !(38.0..=82.0).contains(&cx) { continue; }
                let e1 = [w[1][0] - w[0][0], w[1][1] - w[0][1], w[1][2] - w[0][2]];
                let e2 = [w[2][0] - w[0][0], w[2][1] - w[0][1], w[2][2] - w[0][2]];
                let nn = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
                let l = (nn[0] * nn[0] + nn[1] * nn[1] + nn[2] * nn[2]).sqrt().max(1e-9);
                if nn[1] / l < 0.6 { continue; }
                let x0 = w.iter().map(|v| v[0]).fold(f32::MAX, f32::min);
                let x1 = w.iter().map(|v| v[0]).fold(f32::MIN, f32::max);
                let z0 = w.iter().map(|v| v[2]).fold(f32::MAX, f32::min);
                let z1 = w.iter().map(|v| v[2]).fold(f32::MIN, f32::max);
                levels.push((cy, x0, x1, z0, z1));
            }
            if levels.is_empty() { continue; }
            levels.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            objs.push((format!("map{mi} plc{pi} \"{}\"", level.maps[om].objects[oi].name.trim()), levels));
        }
    }
    objs.sort_by(|a, b| a.0.cmp(&b.0));
    for (name, levels) in &objs {
        // group levels within 0.15
        let mut groups: Vec<(f32, usize, f32, f32, f32, f32)> = Vec::new(); // (y, n, x0, x1, z0, z1)
        for &(y, x0, x1, z0, z1) in levels {
            match groups.last_mut() {
                Some(g) if (g.0 - y).abs() < 0.15 => {
                    g.1 += 1;
                    g.2 = g.2.min(x0); g.3 = g.3.max(x1); g.4 = g.4.min(z0); g.5 = g.5.max(z1);
                    g.0 = (g.0 + y) / 2.0;
                }
                _ => groups.push((y, 1, x0, x1, z0, z1)),
            }
        }
        let s = groups.iter().map(|(y, n, x0, x1, z0, z1)| format!("{y:.2}x{n} [{x0:.1}..{x1:.1},{z0:.1}..{z1:.1}]")).collect::<Vec<_>>().join("  ");
        eprintln!("{name}: {}", if s.len() > 400 { format!("{}...", &s[..400]) } else { s });
    }

    // 3. Every placement's translation in the east zone (any class).
    eprintln!("=== ALL placements translated into x 38..100 z -240..-200 (any class) ===");
    let mut all: Vec<String> = Vec::new();
    for (mi, m) in level.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            let t = p.translation;
            if !(38.0..=100.0).contains(&t[0]) || !(-240.0..=-200.0).contains(&t[2]) { continue; }
            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
            let name = target.map(|(om, oi)| level.maps[om].objects[oi].name.trim().to_string())
                .unwrap_or_else(|| format!("UNRESOLVED {:08x}", p.hashcode));
            let params: Vec<String> = p.params.iter().take(8).map(|(a, b)| format!("{a}:{b:#x}")).collect();
            all.push(format!("map{mi} plc{pi} \"{}\" class {:#04x} @({:.2},{:.2},{:.2}) p[{}]",
                name, p.flags & 0xff, t[0], t[1], t[2], params.join(" ")));
        }
    }
    all.sort();
    for s in &all { eprintln!("{s}"); }
    eprintln!("total {} translations", all.len());
    eprintln!("ok");
}
