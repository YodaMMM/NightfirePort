//! probe14_walk_under: can the real-input crawl leave the store room through the west floor
//! chute (z≈63.6) and traverse the under-vent level south? Assertion-free probe.
use nf_formats::iso9660::IsoImage;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe14_crawl_south_into_chute() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let _ = read_archive(&iso, "07000014.bin");
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..60 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
    sim.bond.crouched = true;
    for _ in 0..15 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
    // face south (yaw pi: forward [sy,0,cy] = [0,0,-1])
    let target_yaw = std::f32::consts::PI;
    for i in 0..900 {
        let yaw_err = target_yaw - sim.bond.yaw;
        let inp = nf_game::player::Input {
            turn: yaw_err.clamp(-0.08, 0.08),
            forward: if i < 30 { 0.0 } else { 1.0 },
            ..Default::default()
        };
        sim.step_bond(inp, false, false, false);
        if i % 30 == 0 {
            let b = &sim.bond;
            eprintln!("tick {i}: pos ({:.3}, {:.3}, {:.3}) yaw {:.3} hb {} gnd {} cc {:.3}",
                b.pos[0], b.pos[1], b.pos[2], b.yaw, b.head_bump, b.grounded_history != 0, b.cc);
        }
        if sim.bond.pos[2] < 54.0 { eprintln!("reached z<54 at tick {i}"); break; }
    }
    let b = &sim.bond;
    println!("final pos ({:.3}, {:.3}, {:.3}) yaw {:.3}", b.pos[0], b.pos[1], b.pos[2], b.yaw);
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe14_lid_tris() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000014.bin");
    let lv = nf_formats::level::Level::parse(&data).expect("parse");
    let (instances, _) = lv.instances(false);
    for (pi, inst) in instances.iter().enumerate() {
        let o = &lv.maps[inst.map].objects[inst.object];
        if !o.name.contains("Vent_StoreRoom_Graphic") { continue; }
        println!("plc #{pi} \"{}\" kind {:#x} trans {:?} rot {:?}", o.name.trim(),
            inst.placement.flags & 0xffff, inst.placement.translation, inst.placement.rotation);
        let m = nf_formats::placement::object_matrix(inst.placement.rotation, inst.placement.translation);
        if let Some(c) = &o.collision {
            for t in c.triangles() {
                let tw: [[f32; 3]; 3] = t.map(|p| nf_formats::placement::transform(&m, p));
                println!("   {:?}", tw);
            }
        } else { println!("   no collision block"); }
    }
}

#[test]
#[ignore = "assertion-free probe dump; run with --ignored"]
fn probe14_floor_owners() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000014.bin");
    let lv = nf_formats::level::Level::parse(&data).expect("parse");
    let (instances, _) = lv.instances(false);
    // (a) spawn floor band: y 2.3..2.5, x 40.1..41.5, z 65.0..65.8
    // (b) the mouth wall band: x 41.45..41.62, y 2.4..3.9, z 62.0..67.2
    for (label, blo, bhi) in [
        ("spawn-floor y2.4 x40.1..41.5 z65.0..65.8", [40.1f32, 2.3, 65.0], [41.5f32, 2.5, 65.8]),
        ("mouth-wall x41.45..41.62 y2.4..3.9 z62..67.2", [41.45f32, 2.4, 62.0], [41.62f32, 3.9, 67.2]),
    ] {
        println!("== owners of {label} ==");
        for (pi, inst) in instances.iter().enumerate() {
            let Some(c) = &lv.maps[inst.map].objects[inst.object].collision else { continue };
            let o = &lv.maps[inst.map].objects[inst.object];
            let m = nf_formats::placement::object_matrix(inst.placement.rotation, inst.placement.translation);
            let mut n = 0usize;
            for t in c.triangles() {
                let tw: [[f32; 3]; 3] = t.map(|p| nf_formats::placement::transform(&m, p));
                let lo = [tw[0][0].min(tw[1][0]).min(tw[2][0]), tw[0][1].min(tw[1][1]).min(tw[2][1]), tw[0][2].min(tw[1][2]).min(tw[2][2])];
                let hi = [tw[0][0].max(tw[1][0]).max(tw[2][0]), tw[0][1].max(tw[1][1]).max(tw[2][1]), tw[0][2].max(tw[1][2]).max(tw[2][2])];
                if (0..3).all(|k| hi[k] >= blo[k] && lo[k] <= bhi[k]) { n += 1; }
            }
            if n > 0 {
                println!("  plc #{pi} \"{}\" kind {:#x} 0x8000 {} glist {:#x} tris {n}", o.name.trim(),
                    inst.placement.flags & 0xffff, inst.placement.creates_cel(), o.flags);
            }
        }
    }
}
