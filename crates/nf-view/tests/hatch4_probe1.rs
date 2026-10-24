//! level-0a-office-hatch-4 probe 1: the spawn cel's world contents — every static object whose
//! bounds contain the spawn column (x −3.93, z 10.40), with its y range and its tris' y values,
//! plus the game-side placement records near the cab. Assertion-free.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

use nf_formats::iso9660::IsoImage;

#[test]
#[ignore = "assertion-free dump; --ignored --nocapture"]
fn dump_spawn_column_world() {
    let iso = iso();
    let w = Walker::new(nf_view::load(&iso, "0700000a.bin", false).unwrap());
    let sim = w.scene.sim.as_ref().unwrap();
    // The port's own floor answer at the spawn.
    for y in [0.5f32, 1.0, 2.0] {
        println!("port floor_below from y{y}: {:?}", sim.world.floor_below([-3.9319, y, 10.3978], 6.0));
    }
    // The static Geo's objects containing the column.
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000a.bin")).expect("archive");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lv = nf_formats::level::Level::parse(&data).expect("parse");
    println!("\n=== port static objects containing the spawn column ===");
    for (lo, hi) in sim.world.objects_with_bound([-3.9319, 0.0, 10.3978]) {
        println!("obj lo ({:.3},{:.3},{:.3}) hi ({:.3},{:.3},{:.3})", lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]);
    }
    println!("\n=== the plate/cab pieces' collision bounds (plc 58/712/733 + the SS 524/525) ===");
    for (mi, m) in lv.maps.iter().enumerate() {
        for (pi, plc) in m.placements.iter().enumerate() {
            if pi != 58 && pi != 712 && pi != 733 && pi != 524 && pi != 525 { continue; }
            let Some(target) = (if plc.by_hash() { lv.by_hash.get(&plc.hashcode).copied() } else if (plc.index as usize) < m.objects.len() { Some((mi, plc.index as usize)) } else { None }) else { println!("plc#{pi}: no object"); continue };
            let o = &lv.maps[target.0].objects[target.1];
            println!("plc#{pi} kind {:#04x} obj {} collision {}", plc.flags & 0xff, o.name.trim(), o.collision.is_some());
            if let Some(col) = &o.collision {
                let (mut ylo, mut yhi) = (f32::INFINITY, f32::NEG_INFINITY);
                for (tri, _) in col.triangles_with_material().iter() {
                    for v in tri { ylo = ylo.min(v[1]); yhi = yhi.max(v[1]); }
                }
                println!("   local tri y range {:.4}..{:.4}; placement y {:.4} -> world {:.4}..{:.4}", ylo, yhi, plc.translation[1], ylo + plc.translation[1], yhi + plc.translation[1]);
            }
        }
    }
    println!("\n=== map37 objects 0..8 + any name with lift/cab/elev ===");
    for (oi, o) in lv.maps[lv.main_map().unwrap()].objects.iter().enumerate() {
        if oi < 8 || o.name.to_lowercase().contains("lift") || o.name.to_lowercase().contains("cab") || o.name.to_lowercase().contains("elev") {
            println!("obj{oi} {} collision {} flags {:#x}", o.name.trim(), o.collision.is_some(), o.flags);
        }
    }
    println!("\n=== PLACED tris with a face at world y in -0.02..0.002 within 3 of the spawn xz ===");
    let (instances, _celc) = lv.instances(false);
    for inst in &instances {
        let o = &lv.maps[inst.map].objects[inst.object];
        let Some(col) = &o.collision else { continue };
        let m = &inst.matrix;
        for (ti, (tri, _mat)) in col.triangles_with_material().iter().enumerate() {
            let wp = |v: &[f32; 3]| -> [f32; 3] {
                [
                    m[0][0] * v[0] + m[1][0] * v[1] + m[2][0] * v[2] + m[3][0],
                    m[0][1] * v[0] + m[1][1] * v[1] + m[2][1] * v[2] + m[3][1],
                    m[0][2] * v[0] + m[1][2] * v[1] + m[2][2] * v[2] + m[3][2],
                ]
            };
            let w = [wp(&tri[0]), wp(&tri[1]), wp(&tri[2])];
            let lo = [w.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min), w.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min), w.iter().map(|p| p[2]).fold(f32::INFINITY, f32::min)];
            let hi = [w.iter().map(|p| p[0]).fold(f32::NEG_INFINITY, f32::max), w.iter().map(|p| p[1]).fold(f32::NEG_INFINITY, f32::max), w.iter().map(|p| p[2]).fold(f32::NEG_INFINITY, f32::max)];
            if lo[0] <= -3.93 && hi[0] >= -3.93 && lo[2] <= 10.40 && hi[2] >= 10.40 && lo[1] < 0.3 && hi[1] > -0.3 {
                // The plane y at the spawn xz (the ray (0,-1,0) hit).
                let e1 = [w[1][0] - w[0][0], w[1][1] - w[0][1], w[1][2] - w[0][2]];
                let e2 = [w[2][0] - w[0][0], w[2][1] - w[0][1], w[2][2] - w[0][2]];
                let n = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
                let nl = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
                if nl > 1e-9 && n[1].abs() / nl > 0.7 {
                    let d = -(n[0] * w[0][0] + n[1] * w[0][1] + n[2] * w[0][2]);
                    let py = -(n[0] * (-3.93) + n[2] * 10.40 + d) / n[1];
                    if (-0.08..0.12).contains(&py) {
                        println!("map{} obj{} {} tri{} plane y at spawn {:.4} (ys {:.3}/{:.3}/{:.3})",
                            inst.map, inst.object, o.name.trim(), ti, py, w[0][1], w[1][1], w[2][1]);
                    }
                }
            }
        }
    }
    println!("\n=== the level's placements near the cab (x −6..0, z 8..13) ===");
    for (mi, m) in lv.maps.iter().enumerate() {
        for (i, p) in m.placements.iter().enumerate() {
            let (x, y, z) = (p.translation[0], p.translation[1], p.translation[2]);
            if (-8.0..2.0).contains(&x) && (6.0..15.0).contains(&z) {
                let ps = p.params.iter().map(|(j, v)| format!("p{j}={v:#x}")).collect::<Vec<_>>().join(",");
                println!("map{mi} plc#{i} kind {:#04x} ({:.4},{:.4},{:.4}) params [{ps}]", p.flags & 0xff, x, y, z);
            }
        }
    }
}

#[path = "common/walker.rs"]
mod walker;
use walker::Walker;
