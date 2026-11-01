//! walkreal probe (level-0c-east-climb): the east strip fine map + room05/room04 levels.
//! (1) down-rays y 14 -> 0 on a 0.5 grid over x 78..98, z −213..−205 — every surface column of
//! the east climb zone INCLUDING the gate deck and room_turbine05;
//! (2) per-object up-facing levels (normal.y > 0.6, centroid y 5..18) for x 78..100 — the room04
//! east part, room05, walkways, staircases;
//! (3) the room floors' tris at y 10.3..11.0, per tri, rooms only.
//! Diagnostic only, --nocapture.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::placement as plc;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// Ray-down (double-sided) over every placement's transformed triangles; returns the highest hit y.
fn ray_down(level: &Level, x: f32, y0: f32, y1: f32, z: f32) -> Option<f32> {
    let len = y0 - y1;
    let dir = [0.0f32, -1.0, 0.0];
    let mut best: Option<f32> = None;
    for (mi, m) in level.maps.iter().enumerate() {
        for p in &m.placements {
            let resolved: Option<(usize, usize)> = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
            let Some((rm, oi)) = resolved else { continue };
            let Some(col) = level.maps[rm].objects[oi].collision.as_ref() else { continue };
            let matrix = plc::object_matrix_for_class(p.flags & 0xff, p.creates_cel(), p.rotation, p.translation, p.quaternion());
            for t in col.triangles() {
                let w: Vec<[f32; 3]> = t.iter().map(|&v| plc::transform(&matrix, v)).collect();
                let (v0, v1, v2) = (w[0], w[1], w[2]);
                let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
                let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
                let pv = [dir[1] * e2[2] - dir[2] * e2[1], dir[2] * e2[0] - dir[0] * e2[2], dir[0] * e2[1] - dir[1] * e2[0]];
                let det = e1[0] * pv[0] + e1[1] * pv[1] + e1[2] * pv[2];
                if det.abs() < 1e-9 { continue; }
                let inv = 1.0 / det;
                let tv = [x - v0[0], y0 - v0[1], z - v0[2]];
                let u = (tv[0] * pv[0] + tv[1] * pv[1] + tv[2] * pv[2]) * inv;
                if !(0.0..=1.0).contains(&u) { continue; }
                let qv = [tv[1] * e1[2] - tv[2] * e1[1], tv[2] * e1[0] - tv[0] * e1[2], tv[0] * e1[1] - tv[1] * e1[0]];
                let v = (dir[0] * qv[0] + dir[1] * qv[1] + dir[2] * qv[2]) * inv;
                if v < 0.0 || u + v > 1.0 { continue; }
                let t = (e2[0] * qv[0] + e2[1] * qv[1] + e2[2] * qv[2]) * inv;
                if !(0.0..=len).contains(&t) { continue; }
                let y = y0 - t;
                best = Some(best.map_or(y, |b: f32| b.max(y)));
            }
        }
    }
    best
}

#[test]
fn probe_east_climb6() {
    let isopath = iso();
    let img = IsoImage::open(&isopath).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000c.bin")).expect("0700000c.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let level = Level::parse(&data).expect("level");

    // 1. The fine map.
    eprintln!("=== east strip fine map (down-rays y 14->0; rows z -241..-203 step 0.5, cols x 78..98 step 0.5) ===");
    let mut header = String::from("z\\x   ");
    for i in 0..=84 { header.push_str(&format!("{:>6.1}", 38.0 + 0.5 * i as f32)); }
    eprintln!("{}", header);
    for zi in 0..=76 {
        let z = -241.0f32 + 0.5 * zi as f32;
        let mut row = format!("{z:>6.1} ");
        for i in 0..=84 {
            let x = 38.0f32 + 0.5 * i as f32;
            match ray_down(&level, x, 14.0, 0.0, z) {
                Some(y) => row.push_str(&format!("{:>6.2}", y)),
                None => row.push_str("     ."),
            }
        }
        eprintln!("{row}");
    }

    // 2. Per-object levels x 78..100.
    eprintln!("=== up-facing tri levels y 5..18 x 78..100 (per object) ===");
    let mut objs: Vec<(String, Vec<(f32, f32, f32, f32, f32)>)> = Vec::new();
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
                if !(5.0..=18.0).contains(&cy) || !(78.0..=100.0).contains(&cx) { continue; }
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
        let mut groups: Vec<(f32, usize, f32, f32, f32, f32)> = Vec::new();
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
        eprintln!("{name}: {}", if s.len() > 1200 { format!("{}...", &s[..1200]) } else { s });
    }

    // 3. The room floors at y 10.3..11.0, per tri, rooms only.
    eprintln!("=== room_turbine* floor tris y 10.3..11.0 (per tri) ===");
    for (mi, m) in level.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
            let Some((om, oi)) = target else { continue };
            let name = level.maps[om].objects[oi].name.trim();
            if !name.starts_with("room_turbine") { continue; }
            let Some(col) = level.maps[om].objects[oi].collision.as_ref() else { continue };
            let matrix = plc::object_matrix_for_class(p.flags & 0xff, p.creates_cel(), p.rotation, p.translation, p.quaternion());
            for t in col.triangles() {
                let w: Vec<[f32; 3]> = t.iter().map(|&v| plc::transform(&matrix, v)).collect();
                let cy = (w[0][1] + w[1][1] + w[2][1]) / 3.0;
                if !(10.3..=11.0).contains(&cy) { continue; }
                let e1 = [w[1][0] - w[0][0], w[1][1] - w[0][1], w[1][2] - w[0][2]];
                let e2 = [w[2][0] - w[0][0], w[2][1] - w[0][1], w[2][2] - w[0][2]];
                let nn = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
                let l = (nn[0] * nn[0] + nn[1] * nn[1] + nn[2] * nn[2]).sqrt().max(1e-9);
                if nn[1] / l < 0.6 { continue; }
                eprintln!("\"{}\" plc{pi} tri ({:.2},{:.2},{:.2})|({:.2},{:.2},{:.2})|({:.2},{:.2},{:.2})",
                    name, w[0][0], w[0][1], w[0][2], w[1][0], w[1][1], w[1][2], w[2][0], w[2][1], w[2][2]);
            }
        }
    }
    eprintln!("ok");
}
