//! probe14_mouth3: what exactly stops the eastward crawl at x=40.98? Assertion-free probe.
//! 1. Capsule-push sweep along the lane z=65.45 (and 63.7), x 40.2..42.2, against the sim world.
//! 2. Per-instance attribution: rebuild a flat mini-world per mouth-region placement and repeat
//!    the sweep; the instance whose mini-world pushes names the blocker.
//! 3. Dump the blocking instance's tris near the contact.
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

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe14_mouth3_push_sweep() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000014.bin");
    let lv = Level::parse(&data).expect("parse");
    let (instances, _) = lv.instances(false);

    // The crawl-rest capsule the wedge sits at (from probe14_mouth2 D): pos (40.98, 3.05, 65.45)
    // crouched, cc 0.55 -> segment a=pos+0.1, b=pos-0.1, r 0.55.
    for &lane_z in &[65.45f32, 63.7] {
        println!("== push sweep, lane z={lane_z} (sim world, capsule a/b y 3.15/2.95, r 0.55) ==");
        let scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
        let world = &scene.sim.as_ref().unwrap().world;
        let mut x = 40.2f32;
        while x <= 42.3 {
            let a = [x, 3.15, lane_z];
            let b = [x, 2.95, lane_z];
            let (push, walkable) = world.capsule_push_scoped(None, a, b, 0.55, true, [1.0e9, 1.0e9, 1.0e9]);
            if push[0].abs() > 1e-5 || push[1].abs() > 1e-5 || push[2].abs() > 1e-5 {
                println!("x {x:5.2}: push [{:.5}, {:.5}, {:.5}] walkable {walkable}", push[0], push[1], push[2]);
            }
            x += 0.05;
        }
        drop(scene);
    }

    println!("== per-instance attribution (mini flat worlds over the lane z=65.45 sweep) ==");
    let blo = [40.0f32, 2.0, 63.0];
    let bhi = [43.2f32, 4.6, 67.0];
    for (pi, inst) in instances.iter().enumerate() {
        let Some(c) = &lv.maps[inst.map].objects[inst.object].collision else { continue };
        let o = &lv.maps[inst.map].objects[inst.object];
        let m = object_matrix(inst.placement.rotation, inst.placement.translation);
        let tris: Vec<[[f32; 3]; 3]> = c.triangles().into_iter()
            .map(|t| t.map(|p| transform(&m, p)))
            .collect();
        // keep only instances with any tri near the lane sweep band
        let near = tris.iter().any(|t| t.iter().any(|p|
            p[0] > blo[0] && p[0] < bhi[0] && p[1] > blo[1] && p[1] < bhi[1] && p[2] > blo[2] && p[2] < bhi[2]));
        if !near { continue; }
        let world = nf_game::world::CollisionWorld::from_triangles(tris.clone());
        let mut first_block: Option<f32> = None;
        let mut first_push = [0f32; 3];
        let mut x = 40.2f32;
        while x <= 42.3 {
            let a = [x, 3.15, 65.45];
            let b = [x, 2.95, 65.45];
            let (push, _) = world.capsule_push_scoped(None, a, b, 0.55, true, [1.0e9, 1.0e9, 1.0e9]);
            if push[0].abs() > 1e-5 || push[1].abs() > 1e-5 || push[2].abs() > 1e-5 {
                if first_block.is_none() { first_block = Some(x); first_push = push; }
            }
            x += 0.05;
        }
        if let Some(x0) = first_block {
            println!("BLOCKER plc #{pi} \"{}\" kind {:#x} glist {:#x} first push at x {x0:.2} push {:?} ({} tris)",
                o.name.trim(), inst.placement.flags & 0xffff, o.flags, first_push, tris.len());
        } else {
            println!("clear    plc #{pi} \"{}\" kind {:#x} glist {:#x} ({} tris)",
                o.name.trim(), inst.placement.flags & 0xffff, o.flags, tris.len());
        }
    }
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe14_room_shell_transform() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000014.bin");
    let lv = Level::parse(&data).expect("parse");
    let (instances, _) = lv.instances(false);
    for (pi, inst) in instances.iter().enumerate() {
        let o = &lv.maps[inst.map].objects[inst.object];
        let f16 = inst.placement.flags & 0xffff;
        if f16 != 0xc023 && f16 != 0xc047 { continue; }
        let nz_t = inst.placement.translation.iter().any(|&v| v.abs() > 1e-6);
        let nz_r = inst.placement.rotation.iter().any(|&v| v.abs() > 1e-6);
        println!("plc #{pi} \"{}\" kind {f16:#x} translation {:?} rotation {:?} (nonzero t {nz_t} r {nz_r})",
            o.name.trim(), inst.placement.translation, inst.placement.rotation);
    }
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe14_mouth_wall_tris() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000014.bin");
    let lv = Level::parse(&data).expect("parse");
    let (instances, _) = lv.instances(false);
    // plc #16 AirConditioningRoom (m54 o16): tris meeting the wall band x 41.3..41.8, y 2.2..3.9, z 60.0..67.0
    let inst = &instances[16];
    assert_eq!(inst.object, 16, "plc #16 is AirConditioningRoom");
    let c = lv.maps[inst.map].objects[inst.object].collision.as_ref().unwrap();
    println!("== AirConditioningRoom tris in the wall band (x 41.3..41.8, y 2.2..3.9, z 60..67) ==");
    for (ti, t) in c.triangles_with_material().into_iter().enumerate() {
        let tw: [[f32; 3]; 3] = t.0.map(|p| nf_formats::placement::transform(&inst.matrix, p));
        let lo = [tw[0][0].min(tw[1][0]).min(tw[2][0]), tw[0][1].min(tw[1][1]).min(tw[2][1]), tw[0][2].min(tw[1][2]).min(tw[2][2])];
        let hi = [tw[0][0].max(tw[1][0]).max(tw[2][0]), tw[0][1].max(tw[1][1]).max(tw[2][1]), tw[0][2].max(tw[1][2]).max(tw[2][2])];
        if hi[0] >= 41.3 && lo[0] <= 41.8 && hi[1] >= 2.2 && lo[1] <= 3.9 && hi[2] >= 60.0 && lo[2] <= 67.0 {
            println!("tri {ti} mat {:#x} lo {:?} hi {:?}", t.1, lo, hi);
        }
    }
    // also the same band for the full sim world: which runs (instance sources) supply tris there?
    println!("== the port's world: same-band tris per source run ==");
    let scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_ref().unwrap();
    let tris = sim.world.tris_near([41.3, 2.2, 60.0], [41.8, 3.9, 67.0]);
    println!("tris_near returned {} tris in the band box", tris.len());
    for t in tris.iter().take(40) {
        let lo = [t[0][0].min(t[1][0]).min(t[2][0]), t[0][1].min(t[1][1]).min(t[2][1]), t[0][2].min(t[1][2]).min(t[2][2])];
        let hi = [t[0][0].max(t[1][0]).max(t[2][0]), t[0][1].max(t[1][1]).max(t[2][1]), t[0][2].max(t[1][2]).max(t[2][2])];
        if hi[0] >= 41.4 && lo[0] <= 41.7 && hi[1] >= 2.3 && lo[1] <= 3.8 {
            println!("  lo {:?} hi {:?}", lo, hi);
        }
    }
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe14_who_blocks_6545() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000014.bin");
    let lv = Level::parse(&data).expect("parse");
    let (instances, _) = lv.instances(false);
    let scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    let world = &scene.sim.as_ref().unwrap().world;
    let (push, _) = world.capsule_push_scoped(None, [41.2, 3.15, 65.45], [41.2, 2.95, 65.45], 0.55, true, [1.0e9, 1.0e9, 1.0e9]);
    println!("sim world push at (41.2, 65.45): {push:?}");
    drop(scene);
    println!("== per-instance push at (41.2, y 3.15/2.95, z 65.45), r 0.55 ==");
    for (pi, inst) in instances.iter().enumerate() {
        let Some(c) = &lv.maps[inst.map].objects[inst.object].collision else { continue };
        let m = object_matrix(inst.placement.rotation, inst.placement.translation);
        let tris: Vec<[[f32; 3]; 3]> = c.triangles().into_iter().map(|t| t.map(|p| transform(&m, p))).collect();
        if !tris.iter().any(|t| t.iter().any(|p| p[0] > 40.5 && p[0] < 42.5 && p[1] > 2.0 && p[1] < 4.5 && p[2] > 63.5 && p[2] < 67.0)) { continue; }
        let w = nf_game::world::CollisionWorld::from_triangles(tris.clone());
        let (push, _) = w.capsule_push_scoped(None, [41.2, 3.15, 65.45], [41.2, 2.95, 65.45], 0.55, true, [1.0e9, 1.0e9, 1.0e9]);
        if push[0].abs() > 1e-5 || push[1].abs() > 1e-5 || push[2].abs() > 1e-5 {
            let o = &lv.maps[inst.map].objects[inst.object];
            println!("pusher plc #{pi} \"{}\" kind {:#x} glist {:#x} push {push:?}", o.name.trim(), inst.placement.flags & 0xffff, o.flags);
            // the pushing tris: those within r of the capsule
            for t in &tris {
                let lo = [t[0][0].min(t[1][0]).min(t[2][0]), t[0][1].min(t[1][1]).min(t[2][1]), t[0][2].min(t[1][2]).min(t[2][2])];
                let hi = [t[0][0].max(t[1][0]).max(t[2][0]), t[0][1].max(t[1][1]).max(t[2][1]), t[0][2].max(t[1][2]).max(t[2][2])];
                if hi[0] >= 40.6 && lo[0] <= 41.8 && hi[1] >= 2.3 && lo[1] <= 3.8 && hi[2] >= 64.8 && lo[2] <= 66.1 {
                    println!("   tri lo {:?} hi {:?}", lo, hi);
                }
            }
        }
    }
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe14_duct_floor_map() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    let world = &scene.sim.as_ref().unwrap().world;
    println!("== floor map (ray down from y 3.19; . = no floor, digit = floor y band) ==");
    print!("     ");
    let mut x = 40.0f32;
    while x <= 47.0 { print!("{:4.0}", x); x += 0.5; }
    println!();
    let mut z = 67.0f32;
    while z >= 54.5 {
        print!("z {:4.1} ", z);
        let mut x = 40.0f32;
        while x <= 47.0 {
            let f = world.segment([x, 3.19, z], [x, 0.0, z]).map(|t| 3.19 - 3.19 * t);
            let c = match f { None => '.', Some(y) if (2.3..2.5).contains(&y) => '2', Some(y) if (2.5..2.7).contains(&y) => '6', Some(y) if y < 2.3 => 'v', Some(y) if (2.7..3.19).contains(&y) => '^', Some(_) => '?' };
            print!("{c:4}", );
            x += 0.5;
        }
        println!();
        z -= 0.5;
    }
    println!("== ceiling map over floor+0.02 up to 5.5 (same grid; digit = ceil y band) ==");
    print!("     ");
    let mut x = 40.0f32;
    while x <= 47.0 { print!("{:4.0}", x); x += 0.5; }
    println!();
    let mut z = 67.0f32;
    while z >= 54.5 {
        print!("z {:4.1} ", z);
        let mut x = 40.0f32;
        while x <= 47.0 {
            let f = world.segment([x, 3.19, z], [x, 0.0, z]).map(|t| 3.19 - 3.19 * t);
            let c = match f {
                None => '.',
                Some(y) => {
                    let c = world.segment([x, y + 0.02, z], [x, 5.5, z]).map(|t| y + 0.02 + (5.5 - y) * t);
                    match c { None => 'o', Some(cy) if (3.7..3.9).contains(&cy) => '8', Some(cy) if cy < 3.7 => '<', Some(cy) if (3.9..4.5).contains(&cy) => '9', Some(_) => '+' }
                }
            };
            print!("{c:4}", );
            x += 0.5;
        }
        println!();
        z -= 0.5;
    }
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe14_mouth_fine_grid() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    let world = &scene.sim.as_ref().unwrap().world;
    println!("== fine grid: floor/ceiling, x 41.0..46.5 step 0.25, z 63.0..66.5 step 0.25 ==");
    let mut z = 66.5f32;
    while z >= 62.75 {
        let mut row = format!("z {:5.2}: ", z);
        let mut x = 41.0f32;
        while x <= 46.5 {
            let f = world.segment([x, 3.19, z], [x, -1.0, z]).map(|t| 3.19 + (-1.0 - 3.19) * t);
            let c = world.segment([x, f.unwrap_or(3.19) + 0.02, z], [x, 5.5, z]).map(|t| f.unwrap_or(3.19) + 0.02 + (5.5 - f.unwrap_or(3.19)) * t);
            row.push_str(&match (f, c) {
                (Some(fv), Some(cv)) => format!("[{:.2}|{:.2}]", fv, cv),
                (Some(fv), None) => format!("[{:.2}| open]", fv),
                (None, _) => "[----|----]".to_string(),
            });
            x += 0.5;
        }
        println!("{row}");
        z -= 0.25;
    }
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe14_full_vent_map() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    let world = &scene.sim.as_ref().unwrap().world;
    // floor level map: F2 = 2.4 duct floor, F0 = -0.6..0.6 low level, F1 = 0.6..2.3 mid, pit = none above -3.5
    println!("== floor band map x 40..48 step 0.5, z 71..54 step 0.5 (2=2.2..2.6, 1=0.6..2.2, 0=-0.7..0.6, _=-3.5..-0.7, .=none) ==");
    print!("      ");
    for i in 0..17 { print!("{:>4}", 40 + i); }
    println!();
    let mut z = 71.0f32;
    while z >= 54.0 {
        print!("z {:5.1} ", z);
        let mut x = 40.0f32;
        while x <= 48.0 {
            let f = world.segment([x, 5.0, z], [x, -3.5, z]).map(|t| 5.0 + (-3.5 - 5.0) * t);
            let c = match f {
                None => '.',
                Some(y) if (2.2..2.6).contains(&y) => '2',
                Some(y) if (0.6..2.2).contains(&y) => '1',
                Some(y) if (-0.7..0.6).contains(&y) => '0',
                Some(_) => '_',
            };
            print!("{c:>4}");
            x += 0.5;
        }
        println!();
        z -= 0.5;
    }
    println!("== ceiling map over the same grid (ray up from floor+0.02 to 5.5) ==");
    print!("      ");
    for i in 0..17 { print!("{:>4}", 40 + i); }
    println!();
    let mut z = 71.0f32;
    while z >= 54.0 {
        print!("z {:5.1} ", z);
        let mut x = 40.0f32;
        while x <= 48.0 {
            let f = world.segment([x, 5.0, z], [x, -3.5, z]).map(|t| 5.0 + (-3.5 - 5.0) * t);
            let c = match f {
                None => '.',
                Some(y) => match world.segment([x, y + 0.02, z], [x, 5.5, z]).map(|t| y + 0.02 + (5.5 - y) * t) {
                    None => 'o',
                    Some(cy) if (1.2..1.6).contains(&cy) => 'L',
                    Some(cy) if (3.6..4.0).contains(&cy) => 'R',
                    Some(cy) if cy < 1.2 => 'l',
                    Some(cy) if (1.6..3.6).contains(&cy) => 'm',
                    Some(_) => 'h',
                },
            };
            print!("{c:>4}");
            x += 0.5;
        }
        println!();
        z -= 0.5;
    }
}
