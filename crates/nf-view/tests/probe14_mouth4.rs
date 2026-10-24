//! probe14_mouth4: the vent-room east crossing topology. The mouth3 attribution named the
//! AirConditioningRoom's own x≈41.534 wall face as the z=65.45 blocker, and the face tris
//! (7713/7717/7719/7723/7724/7726/7727) run z 62.2..67.2 FULL height — the room reads SEALED
//! east in the port's world, while the game ships a route east past lid SP #328. Assertion-free
//! probe: map every opening out of the room (floor holes at the lid spots, the north end, the
//! crossing gaps) and name the owners of the crossing geometry.
use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;
use nf_formats::placement::{object_matrix, transform};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

fn tri_box(t: &[[f32; 3]; 3]) -> ([f32; 3], [f32; 3]) {
    let lo = [t[0][0].min(t[1][0]).min(t[2][0]), t[0][1].min(t[1][1]).min(t[2][1]), t[0][2].min(t[1][2]).min(t[2][2])];
    let hi = [t[0][0].max(t[1][0]).max(t[2][0]), t[0][1].max(t[1][1]).max(t[2][1]), t[0][2].max(t[1][2]).max(t[2][2])];
    (lo, hi)
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe14_mouth4_topo() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000014.bin");
    let lv = Level::parse(&data).expect("parse");
    let (instances, _) = lv.instances(false);

    // A. the sim-world crossing grid: x 40.4..45.4 step 0.25, z 63.2..66.4 step 0.2,
    //    floor (ray down from 3.3) and ceiling (up from floor to 5.5).
    let scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    let world = &scene.sim.as_ref().unwrap().world;
    println!("== A. crossing grid: [floor|ceiling], x 40.4..45.4 step 0.25, z 63.2..66.4 step 0.2 ==");
    let mut z = 66.4f32;
    while z >= 63.19 {
        let mut row = format!("z {:5.2}:", z);
        let mut x = 40.4f32;
        while x <= 45.41 {
            let f = world.segment([x, 3.3, z], [x, -1.0, z]).map(|t| 3.3 + (-1.0 - 3.3) * t);
            let c = world.segment([x, f.unwrap_or(3.3) + 0.02, z], [x, 5.5, z]).map(|t| f.unwrap_or(3.3) + 0.02 + (5.5 - f.unwrap_or(3.3)) * t);
            row.push_str(&match (f, c) {
                (Some(fv), Some(cv)) => format!("[{:.2}|{:.2}]", fv, cv),
                (Some(fv), None) => format!("[{:.2}|open]", fv),
                (None, _) => "[----|----]".to_string(),
            });
            x += 0.25;
        }
        println!("{row}");
        z -= 0.2;
    }
    // B. the room interior: floor holes? ray down from 3.0 across the room z 63.4..67.6.
    println!("== B. room floor map (down from 3.0), x 40.0..41.8 step 0.2, z 63.4..67.6 step 0.2 ==");
    let mut z = 67.6f32;
    while z >= 63.39 {
        let mut row = format!("z {:5.2}:", z);
        let mut x = 40.0f32;
        while x <= 41.81 {
            let f = world.segment([x, 3.0, z], [x, -1.0, z]).map(|t| 3.0 + (-1.0 - 3.0) * t);
            row.push_str(&match f {
                Some(fv) => format!("{:>6.2}", fv),
                None => "  ----".to_string(),
            });
            x += 0.2;
        }
        println!("{row}");
        z -= 0.2;
    }
    // C. the north: floor/ceiling along x 40.0..48.0 at z 67.0..71.0 (step 0.5), ray from 6.0 down.
    println!("== C. north map (ray down from 6.0 to -8), x 40..48 step 0.5, z 71..67 step 0.5 ==");
    let mut z = 71.0f32;
    while z >= 66.99 {
        let mut row = format!("z {:5.2}:", z);
        let mut x = 40.0f32;
        while x <= 48.01 {
            let f = world.segment([x, 6.0, z], [x, -8.0, z]).map(|t| 6.0 + (-8.0 - 6.0) * t);
            row.push_str(&match f {
                Some(fv) => format!("{:>6.2}", fv),
                None => "  ----".to_string(),
            });
            x += 0.5;
        }
        println!("{row}");
        z -= 0.5;
    }
    drop(scene);

    // D. owners of every tri meeting the crossing box (x 41.4..45.0, y 2.0..4.2, z 63.4..66.2).
    println!("== D. crossing-box owners (x 41.4..45.0, y 2.0..4.2, z 63.4..66.2) [D level data] ==");
    let blo = [41.4f32, 2.0, 63.4];
    let bhi = [45.0f32, 4.2, 66.2];
    for (pi, inst) in instances.iter().enumerate() {
        let Some(c) = &lv.maps[inst.map].objects[inst.object].collision else { continue };
        let o = &lv.maps[inst.map].objects[inst.object];
        let m = object_matrix(inst.placement.rotation, inst.placement.translation);
        let mut kept = 0usize;
        let mut zmin = f32::MAX; let mut zmax = f32::MIN; let mut ymin = f32::MAX; let mut ymax = f32::MIN;
        for t in c.triangles() {
            let tw: [[f32; 3]; 3] = t.map(|p| transform(&m, p));
            let (lo, hi) = tri_box(&tw);
            if (0..3).all(|k| hi[k] >= blo[k] && lo[k] <= bhi[k]) {
                kept += 1;
                zmin = zmin.min(lo[2]); zmax = zmax.max(hi[2]);
                ymin = ymin.min(lo[1]); ymax = ymax.max(hi[1]);
            }
        }
        if kept > 0 {
            println!("plc #{pi} \"{}\" kind {:#x} glist {:#x} kept {kept} y {ymin:.2}..{ymax:.2} z {zmin:.2}..{zmax:.2}",
                o.name.trim(), inst.placement.flags & 0xffff, o.flags);
        }
    }
    // E. the wall face: every tri with all |x−41.534|<0.005 and y 2.3..4.0, sorted by z range.
    println!("== E. wall-face tris at |x−41.534|<0.005 (any owner), y meeting 2.3..4.0, z sorted ==");
    let mut faces: Vec<(f32, f32, f32, f32, usize, String)> = Vec::new();
    for (pi, inst) in instances.iter().enumerate() {
        let Some(c) = &lv.maps[inst.map].objects[inst.object].collision else { continue };
        let m = object_matrix(inst.placement.rotation, inst.placement.translation);
        for t in c.triangles() {
            let tw: [[f32; 3]; 3] = t.map(|p| transform(&m, p));
            if !tw.iter().all(|p| (p[0] - 41.534).abs() < 0.005) { continue; }
            let (lo, hi) = tri_box(&tw);
            if hi[1] < 2.3 || lo[1] > 4.0 { continue; }
            let o = &lv.maps[inst.map].objects[inst.object];
            faces.push((lo[2], hi[2], lo[1], hi[1], pi, o.name.trim().to_string()));
        }
    }
    faces.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    for (z0, z1, y0, y1, pi, name) in &faces {
        println!("plc #{pi} \"{name}\" z {z0:.3}..{z1:.3} y {y0:.3}..{y1:.3}");
    }
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe14_mouth4_navweb() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000014.bin");
    let lv = Level::parse(&data).expect("parse");
    // Every main-map AI path node in the vent region (x 36..50, y −2..7, z 52..72) with its links'
    // neighbour positions — the game's own walkability graph [D block 0x05].
    for (mi, map) in lv.maps.iter().enumerate() {
        let Some(net) = &map.ai_network else { continue };
        for (pi, path) in net.paths.iter().enumerate() {
            if path.is_boundary() { continue; }
            let adj = path.adjacency();
            for (ni, n) in path.nodes.iter().enumerate() {
                let p = n.game_pos();
                if !(36.0..50.0).contains(&p[0]) || !(-2.0..7.0).contains(&p[1]) || !(52.0..72.0).contains(&p[2]) { continue; }
                let neigh: Vec<String> = adj[ni].iter().filter_map(|&li| path.links.get(li as usize).map(|l| {
                    let o = if l.a as usize == ni { l.b as usize } else { l.a as usize };
                    match path.nodes.get(o) { Some(on) => format!("({:.1},{:.1},{:.1})", on.game_pos()[0], on.game_pos()[1], on.game_pos()[2]), None => "?".to_string() }
                })).collect();
                println!("map {mi} path {pi} node {ni}: ({:.2}, {:.2}, {:.2}) -> {}", p[0], p[1], p[2], neigh.join(" "));
            }
        }
    }
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe14_mouth4_lip_rest() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    let world = &scene.sim.as_ref().unwrap().world;
    // What does the sim world hold under the rest point (40.80, y 3.0, z 63.29)?
    for z in [62.6f32, 62.9, 63.1, 63.29, 63.5, 63.7, 63.9] {
        let hits: Vec<String> = (0..3).filter_map(|k| {
            let x = 40.4f32 + 0.4 * k as f32;
            world.segment([x, 3.0, z], [x, -1.5, z]).map(|t| format!("x{x:.1}->{:.2}", 3.0 + (-1.5 - 3.0) * t))
        }).collect();
        println!("z {z:.2}: {}", hits.join("  "));
    }
    // The near-tris around the rest point with their owners (from the scoped world's own buckets).
    let tris = world.tris_near([40.3, 2.2, 62.9], [41.6, 2.6, 63.8]);
    println!("tris_near (40.3..41.6, 2.2..2.6, 62.9..63.8): {} tris", tris.len());
    for t in tris.iter().take(24) {
        let lo = [t[0][0].min(t[1][0]).min(t[2][0]), t[0][1].min(t[1][1]).min(t[2][1]), t[0][2].min(t[1][2]).min(t[2][2])];
        let hi = [t[0][0].max(t[1][0]).max(t[2][0]), t[0][1].max(t[1][1]).max(t[2][1]), t[0][2].max(t[1][2]).max(t[2][2])];
        println!("  lo [{:.3}, {:.3}, {:.3}] hi [{:.3}, {:.3}, {:.3}]", lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]);
    }
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe14_mouth4_scoped_vs_flat() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    let world = &scene.sim.as_ref().unwrap().world;
    // The rest column under the wedge (40.98, z 59.4..62.9): flat segment vs every cel's scoped hit.
    for z in [59.4f32, 60.3, 61.4, 61.8, 62.4, 62.9] {
        let flat = world.segment([40.98, 3.0, z], [40.98, -2.0, z]).map(|t| 3.0 + (-2.0 - 3.0) * t);
        let mut scoped: Vec<String> = Vec::new();
        for cel in 0..24usize {
            if let Some((t, _n)) = world.segment_hit_scoped(Some(cel), [40.98, 3.0, z], [40.98, -2.0, z]) {
                scoped.push(format!("cel{}:{:.2}", cel, 3.0 + (-2.0 - 3.0) * t));
            }
        }
        println!("z {z:.1}: flat {flat:?}  scoped [{}]", scoped.join(" "));
    }
}
