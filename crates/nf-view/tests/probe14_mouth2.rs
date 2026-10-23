//! probe14_mouth2: the eastward crawl wedge at the duct mouth. Assertion-free probe.
//! A. [D] every placement with collision tris in the wider mouth box (x 40.3..42.3, y 2.5..4.5,
//!    z 58.0..67.0): name, kind, 0x8000, glist, counts, extents.
//! B. full tris of the two "AirConBlockingPoly*" placements.
//! C. fine ceiling profile along the lanes z=63.7/65.4, x 39.5..42.5.
//! D. the real eastward crawl: steer yaw to +x, crawl, log the stop, print capsule + touching tris.
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
fn probe14_mouth2_east_crawl() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000014.bin");
    let lv = Level::parse(&data).expect("parse");
    let (instances, _) = lv.instances(false);

    println!("== A. mouth-region owners (x 40.3..42.3, y 2.5..4.5, z 58.0..67.0) [D level data] ==");
    let blo = [40.3f32, 2.5, 58.0];
    let bhi = [42.3f32, 4.5, 67.0];
    for (pi, inst) in instances.iter().enumerate() {
        let Some(c) = &lv.maps[inst.map].objects[inst.object].collision else { continue };
        let o = &lv.maps[inst.map].objects[inst.object];
        let m = object_matrix(inst.placement.rotation, inst.placement.translation);
        let mut n = 0usize;
        let mut ext = [[f32::MAX; 3], [f32::MIN; 3]];
        for t in c.triangles() {
            let tw: [[f32; 3]; 3] = t.map(|p| transform(&m, p));
            let (lo, hi) = tri_box(&tw);
            if meets(&lo, &hi, &blo, &bhi) {
                n += 1;
                for k in 0..3 { ext[0][k] = ext[0][k].min(lo[k]); ext[1][k] = ext[1][k].max(hi[k]); }
            }
        }
        if n > 0 {
            println!("plc #{pi} \"{}\" m{} o{} kind {:#x} 0x8000 {} glist {:#x} tris {} ext lo {:?} hi {:?}",
                o.name.trim(), inst.map, inst.object, inst.placement.flags & 0xffff,
                inst.placement.creates_cel(), o.flags, n, ext[0], ext[1]);
        }
    }

    println!("== B. the AirConBlockingPoly placements, full tris ==");
    for (pi, inst) in instances.iter().enumerate() {
        let o = &lv.maps[inst.map].objects[inst.object];
        if !o.name.contains("AirConBlockingPoly") { continue; }
        let m = object_matrix(inst.placement.rotation, inst.placement.translation);
        println!("plc #{pi} \"{}\" translation {:?}", o.name.trim(), inst.placement.translation);
        for t in lv.maps[inst.map].objects[inst.object].collision.as_ref().unwrap().triangles() {
            let tw: [[f32; 3]; 3] = t.map(|p| transform(&m, p));
            println!("   {:?}", tw);
        }
    }

    println!("== C. fine ceiling profile along the lanes ==");
    let scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    let world = &scene.sim.as_ref().unwrap().world;
    for &lane_z in &[63.7f32, 65.4] {
        println!("-- lane z={lane_z} --");
        let mut x = 39.5f32;
        while x <= 42.5 {
            let floor = world.segment([x, 3.2, lane_z], [x, 0.0, lane_z]).map(|t| 3.2 - 3.2 * t);
            let f = floor.unwrap_or(f32::NAN);
            let ceil = world.segment([x, f + 0.02, lane_z], [x, 5.5, lane_z]).map(|t| f + 0.02 + (5.5 - f) * t);
            println!("x {x:5.2}: floor {f:7.3} ceil {:.3} clearance {:.3}",
                ceil.unwrap_or(f32::NAN), ceil.unwrap_or(f32::NAN) - f);
            x += 0.1;
        }
    }
    drop(scene);

    println!("== D. the real eastward crawl (yaw -> +x) ==");
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..60 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
        sim.bond.crouched = true;
        for _ in 0..15 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
        // steer yaw to +x (forward [sy,0,cy] = +x at yaw pi/2), then crawl
        let target_yaw = std::f32::consts::FRAC_PI_2;
        let mut stuck_ticks = 0;
        let mut prev = sim.bond.pos;
        for i in 0..600 {
            let yaw_err = target_yaw - sim.bond.yaw;
            let inp = nf_game::player::Input {
                turn: yaw_err.clamp(-0.08, 0.08),
                forward: if i < 60 { 0.0 } else { 1.0 },
                ..Default::default()
            };
            sim.step_bond(inp, false, false, false);
            let moved = (sim.bond.pos[0] - prev[0]).hypot(sim.bond.pos[2] - prev[2]);
            prev = sim.bond.pos;
            if i >= 60 && moved < 1e-4 { stuck_ticks += 1; } else { stuck_ticks = 0; }
            if i % 30 == 0 {
                let b = &sim.bond;
                eprintln!("tick {i}: pos {:?} yaw {:.3} cc {:.3} head_bump {} grounded {} moved {moved:.5}",
                    b.pos, b.yaw, b.cc, b.head_bump, b.grounded_history != 0, );
            }
            if stuck_ticks > 40 { eprintln!("stuck at tick {i}"); break; }
        }
        let b = &sim.bond;
        let top = b.pos[1] + 0.1 - (b.cc - 0.55) + 0.55;
        let bot = b.pos[1] + 0.45 - b.cc - 0.55;
        println!("final pos {:?} yaw {:.3} head_bump {} cc {:.3} capsule top {top:.3} bot {bot:.3}",
            b.pos, b.yaw, b.head_bump, b.cc);
        let tris = sim.world.tris_near([b.pos[0] - 0.7, bot - 0.2, b.pos[2] - 0.7], [b.pos[0] + 0.7, top + 0.2, b.pos[2] + 0.7]);
        println!("tris meeting the capsule box (x {:.2}..{:.2}, y {bot:.2}..{top:.2}, z {:.2}..{:.2}):",
            b.pos[0] - 0.55, b.pos[0] + 0.55, b.pos[2] - 0.55, b.pos[2] + 0.55);
        let mut shown = 0;
        for t in &tris {
            let (lo, hi) = tri_box(t);
            if shown < 20 { println!("  tri lo {:?} hi {:?}", lo, hi); shown += 1; }
        }
        println!("  ({} total in the box)", tris.len());
    }
}
