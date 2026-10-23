//! probe14_grapple: the real-input route out of the store room — stand in the tall north part
//! (z > 65.8, ceiling open), aim the Q-Claw so the bolt's flight clears the room shell's east
//! wall top (x=41.534, y 2.4..3.86 [D]), latch grapple #658 (45.30, 4.79, 62.93 [D]), reel,
//! release, and fall into the east duct. Assertion-free probe.
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
fn probe14_grapple_route() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let _ = read_archive(&iso, "07000014.bin");
    let mut scene = nf_view::load(&iso, "07000014.bin", false).unwrap();
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..60 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
    // stand up, walk to the tall north part of the store room
    sim.bond.crouched = false;
    for _ in 0..30 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
    // walk north-east to (41.0, ~, 66.8): face north (yaw 0), forward
    for i in 0..200 {
        let b = &sim.bond;
        let dy = 66.8 - b.pos[2];
        let inp = nf_game::player::Input {
            turn: ((0.0f32.atan2(1.0)) - b.yaw).clamp(-0.08, 0.08),
            forward: if dy > 0.15 { 1.0 } else { 0.0 },
            ..Default::default()
        };
        sim.step_bond(inp, false, false, false);
    }
    {
        let b = &sim.bond;
        println!("stance pos {:?} yaw {:.3} crouched {}", b.pos, b.yaw, b.crouched);
    }
    // the grapple points the sim knows
    let pts: Vec<[f32; 3]> = sim.drones.grapple_points.iter().map(|p| p.pos).collect();
    println!("grapple points: {:?} (want #658 at [45.30, 4.79, 62.93])", pts);
    // switch to the claw
    for _ in 0..40 {
        sim.step_bond(nf_game::player::Input { switch_to: Some(0x50), ..Default::default() }, false, false, false);
    }
    println!("held weapon {:#x}", sim.gun.weapon);
    // find a bolt-legal aim: the ray clears the world to the point's sphere AND comes within reach
    let head = sim.bond.head();
    let point = [45.30f32, 4.79, 62.93];
    let world = &sim.world;
    let mut aim: Option<([f32; 3], f32)> = None; // (dir, dist)
    for dp in -60..=60 {
        for dyaw in -60..=60 {
            let pitch = ((dp as f32) * 0.005).clamp(-0.95, 0.95);
            let yaw = (point[0] - head[0]).atan2(point[2] - head[2]) + (dyaw as f32) * 0.004;
            let (sp, cp) = (pitch * std::f32::consts::FRAC_PI_2).sin_cos();
            let (sy, cy) = yaw.sin_cos();
            let dir = [cp * sy, sp, cp * cy];
            let to_p = [point[0] - head[0], point[1] - head[1], point[2] - head[2]];
            let dist = (to_p[0] * to_p[0] + to_p[1] * to_p[1] + to_p[2] * to_p[2]).sqrt();
            let closest = [0, 1, 2].map(|k| head[k] + dir[k] * dist);
            let miss = ((closest[0] - point[0]).powi(2) + (closest[1] - point[1]).powi(2) + (closest[2] - point[2]).powi(2)).sqrt();
            if miss > 2.4 { continue; } // the point's sphere + the bolt radius 1.0 [M 0x12c5a0:92] (radius guess [S])
            // bolt-legal: the segment head -> closest point must clear the static world
            let end = [head[0] + dir[0] * dist, head[1] + dir[1] * dist, head[2] + dir[2] * dist];
            if world.segment_clear(head, end) {
                aim = Some((dir, dist));
                break;
            }
        }
        if aim.is_some() { break; }
    }
    let Some((dir, dist)) = aim else {
        println!("NO bolt-legal aim found (wall top x=41.534 y3.86 blocks every ray that reaches the point's sphere)");
        return;
    };
    println!("aim dir {:?} dist {:.2} clears the world", dir, dist);
    // steer Bond's aim there with the clamped per-tick turn/look, then fire
    let target_yaw = dir[0].atan2(dir[2]);
    let target_pitch = (dir[1] / (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt()).asin() / std::f32::consts::FRAC_PI_2;
    for i in 0..120 {
        let b = &sim.bond;
        let inp = nf_game::player::Input {
            turn: (target_yaw - b.yaw).clamp(-0.08, 0.08),
            look: (target_pitch - b.pitch).clamp(-0.08, 0.08),
            ..Default::default()
        };
        sim.step_bond(inp, false, false, false);
        if i > 100 {
            let d = sim.bond.look();
            let dot = d[0] * dir[0] + d[1] * dir[1] + d[2] * dir[2];
            if dot > 0.999 { break; }
        }
    }
    println!("aimed: look {:?} (target {dir:?})", sim.bond.look());
    // fire to latch
    let mut latched = false;
    for i in 0..10 {
        sim.step_bond(nf_game::player::Input::default(), true, false, false);
        if sim.bond.grapple.is_some() { latched = true; println!("latched at fire tick {i}: point {:?}", sim.bond.grapple.as_ref().unwrap().point); break; }
    }
    if !latched { println!("NO LATCH"); return; }
    // reel (no input), then release
    for i in 0..200 {
        sim.step_bond(nf_game::player::Input::default(), i >= 5, false, false);
        if i % 20 == 0 {
            let b = &sim.bond;
            eprintln!("reel tick {i}: pos {:?} phase {:?} substate {}", b.pos, b.grapple.as_ref().map(|g| g.phase), b.substate);
        }
        if sim.bond.grapple.is_none() { println!("detached at tick {i}, pos {:?}", sim.bond.pos); break; }
    }
    for i in 0..300 {
        sim.step_bond(nf_game::player::Input::default(), false, false, false);
        if i % 30 == 0 { eprintln!("fall tick {i}: pos {:?}", sim.bond.pos); }
        if sim.bond.grounded_history != 0 && i > 5 { break; }
    }
    let b = &sim.bond;
    println!("landed at {:?} grounded {}", b.pos, b.grounded_history != 0);
    // crawl south down the east duct toward the slot approach
    sim.bond.crouched = true;
    for _ in 0..15 { sim.step_bond(nf_game::player::Input::default(), false, false, false); }
    let target_yaw2 = std::f32::consts::PI;
    for i in 0..1200 {
        let inp = nf_game::player::Input {
            turn: (target_yaw2 - sim.bond.yaw).clamp(-0.08, 0.08),
            forward: 1.0,
            ..Default::default()
        };
        sim.step_bond(inp, false, false, false);
        if i % 60 == 0 { eprintln!("crawl tick {i}: pos {:?}", sim.bond.pos); }
        if sim.bond.pos[2] < 56.6 && sim.bond.pos[1] < 1.6 { println!("reached the slot approach at tick {i}: {:?}", sim.bond.pos); break; }
    }
    println!("final {:?}", sim.bond.pos);
}
