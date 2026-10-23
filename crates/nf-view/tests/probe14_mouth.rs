//! probe14_mouth: whose geometry is the east duct mouth's lintel (the crawl wedges at x=40.98,
//! head_bump set, lanes z=63.7/65.4)? Assertion-free probe.
//!
//! A. [D] from the level data: every placement whose collision tris meet the mouth box
//!    (x 40.5..42.0, y 2.8..4.2, z 62.5..66.5), with name, kind, 0x8000, glist, tri count and
//!    the y-range the tris cover over the two crawl lanes.
//! B. The port-world ceiling profile along both lanes (the world the walker walks): for x steps,
//!    the lowest static tri above the duct floor in the crawl-head band.
//! C. The wedge reproduction: crouch, crawl east, print pos/head_bump and the tris around the
//!    capsule top when it stops.
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

fn meets(lo: &[f32; 3], hi: &[f32; 3], blo: &[f32; 3], bhi: &[f32; 3]) -> bool {
    (0..3).all(|k| hi[k] >= blo[k] && lo[k] <= bhi[k])
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe14_mouth_lintel_owner() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000014.bin");
    let lv = Level::parse(&data).expect("parse");
    let (instances, _) = lv.instances(false);
    println!("== A. mouth box owners (x 40.5..42.0, y 2.8..4.2, z 62.5..66.5) [D level data] ==");
    let blo = [40.5f32, 2.8, 62.5];
    let bhi = [42.0f32, 4.2, 66.5];
    for (pi, inst) in instances.iter().enumerate() {
        let Some(c) = &lv.maps[inst.map].objects[inst.object].collision else { continue };
        let o = &lv.maps[inst.map].objects[inst.object];
        let m = object_matrix(inst.placement.rotation, inst.placement.translation);
        let mut n = 0usize;
        let mut ymin = f32::MAX; let mut ymax = f32::MIN;
        let mut first: Option<[[f32; 3]; 3]> = None;
        for t in c.triangles() {
            let tw: [[f32; 3]; 3] = t.map(|p| transform(&m, p));
            let (lo, hi) = tri_box(&tw);
            if meets(&lo, &hi, &blo, &bhi) {
                n += 1;
                ymin = ymin.min(lo[1]); ymax = ymax.max(hi[1]);
                if first.is_none() { first = Some(tw); }
            }
        }
        if n > 0 {
            println!("plc #{pi} \"{}\" m{} o{} kind {:#x} 0x8000 {} glist {:#x} tris {} y {:.3}..{:.3}",
                o.name.trim(), inst.map, inst.object, inst.placement.flags & 0xffff,
                inst.placement.creates_cel(), o.flags, n, ymin, ymax);
            if let Some(t) = first { println!("   e.g. {:?}", t); }
        }
    }

    println!("== B. port-world ceiling profile along the crawl lanes ==");
    let scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    let world = &scene.sim.as_ref().unwrap().world;
    for &lane_z in &[63.7f32, 65.4] {
        println!("-- lane z={lane_z} --");
        let mut x = 39.0f32;
        while x <= 46.0 {
            // floor: ray down from y 3.2
            let floor = world.segment([x, 3.2, lane_z], [x, 0.0, lane_z]).map(|t| 3.2 - 3.2 * t);
            // ceiling: ray up from just above the floor
            let f = floor.unwrap_or(f32::NAN);
            let ceil = world.segment([x, f + 0.02, lane_z], [x, 5.5, lane_z]).map(|t| f + 0.02 + (5.5 - f) * t);
            // crawl top at rest: floor + cc(0.55) + capsule top (0.65) = floor + 1.2? head room check:
            println!("x {x:5.2}: floor {f:7.3} ceil {:.3} clearance {:.3}",
                ceil.unwrap_or(f32::NAN), ceil.unwrap_or(f32::NAN) - f);
            x += 0.25;
        }
    }
    drop(scene);

    println!("== C. wedge reproduction (crouch, crawl east along z=65.4) ==");
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..60 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
        sim.bond.crouched = true;
        for _ in 0..15 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
        let mut prev_x = sim.bond.pos[0];
        for i in 0..700 {
            let inp = nf_game::player::Input { forward: 1.0, ..Default::default() };
            sim.step_bond(inp, false, false, false);
            let moved = (sim.bond.pos[0] - prev_x).abs();
            prev_x = sim.bond.pos[0];
            if i % 50 == 0 || (i > 0 && moved < 1e-4 && i % 20 == 0) {
                let b = &sim.bond;
                eprintln!("tick {i}: pos {:?} cc {:.3} crouch_drop {:.3} head_bump {} grounded {} moved {moved:.5}",
                    b.pos, b.cc, b.crouch_drop, b.head_bump, b.grounded_history != 0);
            }
        }
        let b = &sim.bond;
        println!("final pos {:?} head_bump {} grounded {}", b.pos, b.head_bump, b.grounded_history != 0);
        // the capsule top tris
        let top = [b.pos[0], b.pos[1] + 0.1 - (b.cc - 0.55) + 0.55, b.pos[2]];
        println!("capsule top y {:.3} at x {:.2} z {:.2}", top[1], top[0], top[2]);
        let tris = sim.world.tris_near([top[0] - 1.5, top[1] - 0.6, top[2] - 1.5], [top[0] + 1.5, top[1] + 0.6, top[2] + 1.5]);
        println!("tris near the capsule top ({}):", tris.len());
        let mut shown = 0;
        for t in &tris {
            let (lo, hi) = tri_box(t);
            if hi[1] > top[1] - 0.35 && lo[1] < top[1] + 0.35 && shown < 12 {
                println!("  tri lo {:?} hi {:?}", lo, hi);
                shown += 1;
            }
        }
    }
}
