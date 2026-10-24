//! level-0a-office-hatch-4 probe 2: the stored PLANE records (3 s16 normal · 2^-14 + the i32
//! distance) of Room-M_lifts obj458's floor tris — the game's Intersect_RayGeom (0x10a8c8) tests
//! the plane, the port only the vertices. The plane y at the spawn xz is the game's hit.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

use nf_formats::iso9660::IsoImage;

#[test]
#[ignore = "assertion-free dump; --ignored --nocapture"]
fn dump_lift_floor_planes() {
    let iso = iso();
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000a.bin")).expect("archive");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lv = nf_formats::level::Level::parse(&data).expect("parse");
    let (instances, _) = lv.instances(true);
    for inst in &instances {
        if inst.map != lv.main_map().unwrap() || inst.object != 0 { continue; }
        let o = &lv.maps[inst.map].objects[inst.object];
        let col = o.collision.as_ref().unwrap();
        let m = &inst.matrix;
        for (ti, (tri, _)) in col.triangles_with_material().iter().enumerate() {
            let wp = |v: &[f32; 3]| [
                m[0][0] * v[0] + m[1][0] * v[1] + m[2][0] * v[2] + m[3][0],
                m[0][1] * v[0] + m[1][1] * v[1] + m[2][1] * v[2] + m[3][1],
                m[0][2] * v[0] + m[1][2] * v[1] + m[2][2] * v[2] + m[3][2]];
            let w = [wp(&tri[0]), wp(&tri[1]), wp(&tri[2])];
            let lo_y = w.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
            let hi_y = w.iter().map(|p| p[1]).fold(f32::NEG_INFINITY, f32::max);
            if hi_y > 0.2 || lo_y < -0.2 { continue; }
            let lo_x = w.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min);
            let hi_x = w.iter().map(|p| p[0]).fold(f32::NEG_INFINITY, f32::max);
            let lo_z = w.iter().map(|p| p[2]).fold(f32::INFINITY, f32::min);
            let hi_z = w.iter().map(|p| p[2]).fold(f32::NEG_INFINITY, f32::max);
            if !(lo_x <= -3.93 && hi_x >= -3.93 && lo_z <= 10.40 && hi_z >= 10.40) { continue; }
            // The stored plane: find this tri's leaf + plane index.
            for bx in col.boxes.iter().filter(|b| b.children.is_none()) {
                let range = bx.tris.start..bx.tris.end;
                if !range.contains(&(ti as u16)) { continue; }
                let base = bx.vblock as usize * 0x20;
                let t = &col.tris[ti];
                let p = t.plane as usize;
                let nw = |k: usize| col.words.get(base + p + k).copied().unwrap_or(0) as i16;
                let n = [nw(0) as f32 / 16384.0, nw(1) as f32 / 16384.0, nw(2) as f32 / 16384.0];
                let raw: Vec<i16> = (0..12).filter_map(|k| col.words.get(base + p + k).copied()).collect();
                println!("tri{ti} plane words at p{}: {:?} (hex {:02x?})", p, raw, raw.iter().map(|v| *v as u16).collect::<Vec<_>>());
                let w3 = col.words.get(base + p + 3).copied().unwrap_or(0) as u16 as i32;
                let w4 = col.words.get(base + p + 4).copied().unwrap_or(0) as u16 as i32;
                let dist32 = w3 | (w4 << 16);
                // The plane y at the spawn xz: y = (d - n.x*x - n.z*z) / n.y (d scaled 2^-14 like the normal).
                let d14 = dist32 as f32 / 16384.0;
                let py = if n[1].abs() > 1e-9 { (d14 - n[0] * (-3.93) - n[2] * 10.40) / n[1] } else { f32::NAN };
                let pyv = if n[1].abs() > 1e-9 { (d14 - n[0] * w[0][0] - n[2] * w[0][2]) / n[1] } else { f32::NAN };
                println!("tri{ti} verts y {:.4}/{:.4}/{:.4} plane n ({:.6},{:.6},{:.6}) d {} d14 {:.6} -> plane y at spawn {:.5} (at v0 {:.5})",
                    w[0][1], w[1][1], w[2][1], n[0], n[1], n[2], dist32, d14, py, pyv);
            }
        }
    }
}
