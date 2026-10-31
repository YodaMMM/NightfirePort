//! walkreal probe #23 (level-0c-gate-band-crossing): the walkway family. Dumps (1) every object
//! in every map whose name contains "walkway" or "turb" (name, hash, map, collision tri count),
//! (2) every placement that resolves to one of those objects (map, index, class, translation),
//! (3) the raw collision primitive-type census of obj_turb2_walkway01 / obj_turb3_walkway04 vs
//! what the port's parser keeps, over the band slab. Diagnostic only, --nocapture.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::levelbin::LevelBin;
use nf_formats::placement as plc;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn probe_band9_walkways() {
    let isopath = iso();
    let img = IsoImage::open(&isopath).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000c.bin")).expect("0700000c.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let _ = LevelBin::parse(&data).expect("levelbin");
    let level = Level::parse(&data).expect("level");

    // 1. The walkway/turbine object family across all maps.
    eprintln!("=== objects named *walkway* / *turb* (any map) ===");
    let mut family: Vec<(usize, usize, String, u32)> = Vec::new();
    for (mi, m) in level.maps.iter().enumerate() {
        for (oi, o) in m.objects.iter().enumerate() {
            let n = o.name.trim().to_lowercase();
            if n.contains("walkway") || n.contains("turb") {
                eprintln!("map{mi} obj{oi} \"{}\" hash {:08x} collision {}", o.name.trim(), o.hashcode, o.collision.is_some());
                family.push((mi, oi, o.name.trim().to_string(), o.hashcode));
            }
        }
    }

    // 2. Every placement resolving into the family.
    eprintln!("=== placements of the family ===");
    for (mi, m) in level.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
            let Some((om, oi)) = target else { continue };
            if !family.iter().any(|&(fm, fo, _, _)| fm == om && fo == oi) { continue; }
            let t = p.translation;
            let fname = family.iter().find(|&&(fm, fo, _, _)| fm == om && fo == oi).unwrap().2.clone();
            // extents of the resolved object's collision under this placement
            let mut ext = String::new();
            if let Some(col) = level.maps[om].objects[oi].collision.as_ref() {
                let matrix = plc::object_matrix_for_class(p.flags & 0xff, p.creates_cel(), p.rotation, p.translation, p.quaternion());
                let (mut x0, mut x1, mut y0, mut y1, mut z0, mut z1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN, f32::MAX, f32::MIN);
                let mut n = 0usize;
                for tri in col.triangles() {
                    for v in tri.iter().map(|&v| plc::transform(&matrix, v)) {
                        x0 = x0.min(v[0]); x1 = x1.max(v[0]); y0 = y0.min(v[1]); y1 = y1.max(v[1]); z0 = z0.min(v[2]); z1 = z1.max(v[2]);
                    }
                    n += 1;
                }
                if n > 0 { ext = format!(" [{n} tris x {x0:.2}..{x1:.2} y {y0:.2}..{y1:.2} z {z0:.2}..{z1:.2}]"); }
            }
            eprintln!("map{mi} plc{pi} -> map{om} obj{oi} \"{fname}\" class {:#04x} t ({:.2},{:.2},{:.2}){ext}",
                p.flags & 0xff, t[0], t[1], t[2]);
        }
    }

    // 3. The two band walkways: raw primitive census (all tris by type the parser exposes) and
    //    their transformed x extents.
    let main = level.main_map().expect("main map");
    for want in ["obj_turb2_staircase01", "obj_turb2_staircase02", "obj_turb2_staircase03", "obj_turb3_staircase01", "obj_turb3_staircase02", "obj_turb3_staircase03", "obj_turb2_supportbeam01", "obj_turb2_supportbeam02", "obj_turb3_supportbeam01", "obj_turb3_supportbeam02", "obj_turb2_pulley"] {
        for (oi, o) in level.maps[main].objects.iter().enumerate() {
            if !o.name.trim().eq_ignore_ascii_case(want) { continue; }
            let Some(col) = o.collision.as_ref() else { eprintln!("{want}: no collision"); continue; };
            // its placement(s)
            for (pi, p) in level.maps[main].placements.iter().enumerate() {
                let tgt = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                    else if (p.index as usize) < level.maps[main].objects.len() { Some((main, p.index as usize)) } else { None };
                if tgt != Some((main, oi)) { continue; }
                let matrix = plc::object_matrix_for_class(p.flags & 0xff, p.creates_cel(), p.rotation, p.translation, p.quaternion());
                let mut xs = (f32::MAX, f32::MIN); let mut ys = (f32::MAX, f32::MIN); let mut zs = (f32::MAX, f32::MIN);
                let mut n = 0usize;
                for t in col.triangles() {
                    let w: Vec<[f32; 3]> = t.iter().map(|&v| plc::transform(&matrix, v)).collect();
                    for v in &w { xs.0 = xs.0.min(v[0]); xs.1 = xs.1.max(v[0]); ys.0 = ys.0.min(v[1]); ys.1 = ys.1.max(v[1]); zs.0 = zs.0.min(v[2]); zs.1 = zs.1.max(v[2]); }
                    n += 1;
                }
                eprintln!("{want} plc{pi}: {n} tris x {:.2}..{:.2} y {:.2}..{:.2} z {:.2}..{:.2}", xs.0, xs.1, ys.0, ys.1, zs.0, zs.1);
            }
        }
    }
    eprintln!("ok");
}
