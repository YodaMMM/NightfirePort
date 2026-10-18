//! level-16 walk blocker tooling: find door[0]'s lock 0x24 writer on the disc data.
//! Dumps every main-map placement that carries 0x24 in any param, every placement near the
//! door[0] approach (z −100..−130), every trigger-family placement with its params, and the
//! door-kind (0xdb) placements with their lock channels. Opt-in NF_ISO; prints only.
//! Evidence: placement records [D] via nf-formats::level.

#[path = "common/walker.rs"]
mod walker;

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

#[test]
#[ignore = "assertion-free dump; run with --ignored"]
fn dump_07000016_writers_and_the_door0_region() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000016.bin");
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().expect("main map");
    let placements = &lv.maps[mm].placements;
    println!("main-map placements: {}", placements.len());

    let name_of = |p: &nf_formats::placement::Placement| -> String {
        let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
            else if (p.index as usize) < lv.maps[mm].objects.len() { Some((mm, p.index as usize)) } else { None };
        target.map(|(m, o)| lv.maps[m].objects[o].name.trim().to_string())
            .unwrap_or_else(|| format!("UNRESOLVED hash {:08x}", p.hashcode))
    };
    let params_of = |p: &nf_formats::placement::Placement| p.params.iter().map(|(a, b)| format!("{a}:{b:#x}")).collect::<Vec<_>>().join(" ");

    // 1. Any placement with 0x24 in a param KEY or VALUE (a writer's out-channel slot).
    println!("\n== placements carrying 0x24 in a param (key or value) ==");
    for (i, p) in placements.iter().enumerate() {
        if p.params.iter().any(|&(k, v)| k == 0x24 || v == 0x24) {
            let t = p.translation;
            println!("#{i} kind {:#04x} \"{}\" @({:.2},{:.2},{:.2}) params [{}] hash {:08x} flags {:#x}",
                p.flags & 0xff, name_of(p), t[0], t[1], t[2], params_of(p), p.hashcode, p.flags);
        }
    }

    // 2. Everything near door[0]'s approach (z −100..−130) — the panel should be beside the door.
    println!("\n== placements in the door[0] region (z −100..−130) ==");
    for (i, p) in placements.iter().enumerate() {
        let t = p.translation;
        if t[2] > -100.0 || t[2] < -130.0 { continue; }
        println!("#{i} kind {:#04x} \"{}\" @({:.2},{:.2},{:.2}) params [{}] hash {:08x}",
            p.flags & 0xff, name_of(p), t[0], t[1], t[2], params_of(p), p.hashcode);
    }

    // 3. All door-kind (0xdb) placements with their lock channels (par 3).
    println!("\n== door-kind (0xdb) placements ==");
    for (i, p) in placements.iter().enumerate() {
        if p.flags & 0xff != 0xdb || p.flags & 0xa000 != 0 { continue; }
        let t = p.translation;
        let lock = p.params.iter().find(|(k, _)| *k == 3).map_or(0, |&(_, v)| v);
        let open = p.params.iter().find(|(k, _)| *k == 2).map_or(0, |&(_, v)| v);
        println!("#{i} \"{}\" @({:.2},{:.2},{:.2}) ch_open {open:#x} ch_lock {lock:#x} params [{}]",
            name_of(p), t[0], t[1], t[2], params_of(p));
    }
}

#[test]
#[ignore = "assertion-free dump; run with --ignored"]
fn dump_07000016_everything() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000016.bin");
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().expect("main map");
    let placements = &lv.maps[mm].placements;
    let name_of = |p: &nf_formats::placement::Placement| -> String {
        let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
            else if (p.index as usize) < lv.maps[mm].objects.len() { Some((mm, p.index as usize)) } else { None };
        target.map(|(m, o)| lv.maps[m].objects[o].name.trim().to_string())
            .unwrap_or_else(|| format!("UNRESOLVED hash {:08x}", p.hashcode))
    };
    let params_of = |p: &nf_formats::placement::Placement| p.params.iter().map(|(a, b)| format!("{a}:{b:#x}")).collect::<Vec<_>>().join(" ");
    for (i, p) in placements.iter().enumerate() {
        let t = p.translation;
        let s = p.scale();
        println!("#{i} kind {:#04x} \"{}\" @({:.2},{:.2},{:.2}) scale ({:.2},{:.2},{:.2}) params [{}] hash {:08x}",
            p.flags & 0xff, name_of(p), t[0], t[1], t[2], s[0], s[1], s[2], params_of(p), p.hashcode);
    }
}

/// The trench-room floor grids: where does the upper west level (the SecurityStation, y 6.6)
/// connect to the main floor? '#' = walkable within `depth` below the plane.
#[test]
#[ignore = "assertion-free dump; run with --ignored"]
fn dump_07000016_floor_grids() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let data = read_archive(&iso, "07000016.bin");
    let lv = Level::parse(&data).expect("parse");
    let _ = lv;
    let scene = nf_view::load(&iso, "07000016.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    let grid = |x0: f32, x1: f32, z0: f32, z1: f32, y: f32, step: f32, depth: f32, label: &str| {
        eprintln!("=== {label}: y {y} depth {depth} ===");
        let mut z = z1;
        while z >= z0 {
            let mut row = format!("z {:7.1}: ", z);
            let mut x = x0;
            while x <= x1 {
                let f = sim.world.floor_below([x, y, z], depth);
                match f {
                    Some(fy) => { let d = y - fy;
                        if d < 0.0 { row.push('^'); } else if d < 2.5 { row.push('#'); } else { row.push('.'); } }
                    None => row.push(' '),
                }
                x += step;
            }
            eprintln!("{row}");
            z -= step;
        }
    };
    // The upper west level (the station at y 6.6) and its connections: sample at y 7.6.
    grid(-20.0, 24.0, -122.0, -84.0, 7.6, 2.0, 3.0, "upper deck");
    // The west strip ascent hunt: fine grids catching intermediate stairs (x −16..0).
    grid(-16.0, 0.0, -104.0, -82.0, 7.6, 1.0, 6.0, "west strip from y 7.6 (depth 6: floors 1.6..7.6)");
    grid(-16.0, 0.0, -104.0, -82.0, 4.0, 1.0, 3.0, "west strip at y 4 (floors 1..4)");
    grid(-16.0, 0.0, -104.0, -82.0, 6.0, 1.0, 2.0, "west strip at y 6 (floors 4..6)");
    // The main floor (y ~0..1) and the trench (−4.8): sample at y 1.0.
    grid(-20.0, 24.0, -122.0, -84.0, 1.0, 2.0, 3.0, "main floor");
    // The trench floor.
    grid(-20.0, 24.0, -106.0, -84.0, -4.0, 2.0, 3.0, "trench floor");
    // Exact floor heights (the topmost floor below y 12) as a height table.
    eprintln!("=== exact floor heights NE (the stairs hunt, x −2..30) ===");
    let mut z = -64.0;
    while z >= -90.0 {
        let mut row = format!("z {:7.1}: ", z);
        let mut x = -2.0;
        while x <= 30.0 {
            match sim.world.floor_below([x, 12.0, z], 20.0) {
                Some(fy) => row.push_str(&format!("{fy:5.1} ")),
                None => row.push_str("  --- "),
            }
            x += 2.0;
        }
        eprintln!("{row}");
        z -= 2.0;
    }
    eprintln!("=== exact floor heights (topmost below y 12) ===");
    let mut z = -82.0;
    while z >= -120.0 {
        let mut row = format!("z {:7.1}: ", z);
        let mut x = -18.0;
        while x <= 24.0 {
            match sim.world.floor_below([x, 12.0, z], 20.0) {
                Some(fy) => row.push_str(&format!("{fy:5.1} ")),
                None => row.push_str("  --- "),
            }
            x += 2.0;
        }
        eprintln!("{row}");
        z -= 2.0;
    }
}

/// Can the game's own AI-point graph route Bond from the arrival up to the SecurityStation
/// trigger (out 0x24)? Prints the nav route and, if walkable, whether the station's E fires
/// ch 0x24 (door[0]'s unlock).
#[test]
#[ignore = "assertion-free probe; run with --ignored"]
fn probe_station_route_and_use() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let mut scene = nf_view::load_with_carry(&iso, "07000016.bin", false, None).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000 { sim.step_bond(Default::default(), false, false, false); if !sim.disabled { break; } }
    for _ in 0..100 { sim.step_bond(Default::default(), false, false, false); }
    let start = sim.bond.pos;
    let station = [-15.2, 6.0, -94.6];
    match walker::nav_route(sim, start, station.into()) {
        Some(r) => {
            eprintln!("nav route ({} legs):", r.len());
            for p in &r { eprintln!("  ({:.1},{:.1},{:.1})", p[0], p[1], p[2]); }
        }
        None => eprintln!("nav_route: NONE (the AI graph does not connect)"),
    }
    eprintln!("ladders: {:?}", walker::ladders(sim));
    eprintln!("bond at ({:.2},{:.2},{:.2})", start[0], start[1], start[2]);
}

/// The live script entities (the nis objects the level's scripts drive): find the "lift".
#[test]
#[ignore = "assertion-free probe; run with --ignored"]
fn probe_script_objects() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let scene = nf_view::load(&iso, "07000016.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    let list = &sim.drones.scripts.objects.list;
    eprintln!("script objects: {}", list.len());
    for (i, o) in list.iter().enumerate() {
        if let Some(o) = o {
            let m = match o { nf_game::nis::Obj::Entity { matrix, .. } => Some(*matrix), _ => None };
            if let Some(m) = m {
                eprintln!("obj[{i}] at ({:.2},{:.2},{:.2})", m[3][0], m[3][1], m[3][2]);
            }
        }
    }
}

/// The live script players (SPs) with their scripts + where their objects sit: find the lift.
#[test]
#[ignore = "assertion-free probe; run with --ignored"]
fn probe_script_players() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let scene = nf_view::load(&iso, "07000016.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    let sp = &sim.drones.scripts.players;
    eprintln!("script players: {}", sp.len());
    for p in sp {
        let m = p.live_matrix;
        eprintln!("sp plc {} scripts {:08x}/{:08x} live({:.2},{:.2},{:.2}) out {:#x} gate {:#x} mask {:#x} activated {}",
            p.placement, p.scripts[0], p.scripts[1], m[3][0], m[3][1], m[3][2], p.out, p.gate, p.mask, p.activated);
    }
    eprintln!("script objects: {}", sim.drones.scripts.objects.list.len());
    for (i, o) in sim.drones.scripts.objects.list.iter().enumerate() {
        if let Some(o) = o {
            if let nf_game::nis::Obj::Entity { matrix, hash, .. } = o {
                eprintln!("obj[{i}] hash {hash:08x} at ({:.2},{:.2},{:.2})", matrix[3][0], matrix[3][1], matrix[3][2]);
            }
        }
    }
}

/// The level's SimpleScripts (0xE2): channels, use-masks, and the key travel (first vs last key).
#[test]
#[ignore = "assertion-free probe; run with --ignored"]
fn probe_ss_scripts() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let scene = nf_view::load(&iso, "07000016.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    eprintln!("SS list: {} entries", sim.drones.sss.list.len());
    for s in &sim.drones.sss.list {
        let first = s.keys.first().map(|k| k.pos);
        let last = s.keys.last().map(|k| k.pos);
        let f = first.map(|p| format!("({:.1},{:.1},{:.1})", p[0], p[1], p[2])).unwrap_or_default();
        let l = last.map(|p| format!("({:.1},{:.1},{:.1})", p[0], p[1], p[2])).unwrap_or_default();
        eprintln!("ss plc {} ch {:#x} state {:#x} mask {:#x} keys {} rest {:.1} wrap {} from {} to {}",
            s.placement, s.channel, s.state, s.mask, s.keys.len(), s.rest, s.wrap_flag, f, l);
    }
}

/// Two-layer fine floor scan of the west half: the topmost floor below y 12 and the next floor
/// below it (to see through the slabs). A walkable ascent shows as a smooth gradient.
#[test]
#[ignore = "assertion-free probe; run with --ignored"]
fn probe_west_floors_fine() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let scene = nf_view::load(&iso, "07000016.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    let mut z = -78.0;
    while z >= -116.0 {
        let mut row = format!("z {:7.1} T:", z);
        let mut x = -2.0;
        while x <= 24.0 {
            match sim.world.floor_below([x, 12.0, z], 20.0) {
                Some(fy) => row.push_str(&format!("{fy:5.1} ")),
                None => row.push_str("  --- "),
            }
            x += 1.0;
        }
        row.push_str("  U:");
        let mut x = -18.0;
        while x <= -2.0 {
            let u = sim.world.floor_below([x, 12.0, z], 20.0)
                .and_then(|top| if top < 11.0 { sim.world.floor_below([x, top - 0.05, z], 20.0) } else { None });
            match u { Some(fy) if fy > -20.0 => row.push_str(&format!("{fy:5.1} ")), _ => row.push_str("  --- ") }
            x += 1.0;
        }
        eprintln!("{row}");
        z -= 1.0;
    }
}

/// The SecurityStation ascent probe: walk the east ramp up to the upper ring, west to the
/// station, press E, and report whether ch 0x24 (door[0]'s unlock) fires.
#[test]
#[ignore = "assertion-free probe; run with --ignored"]
fn probe_station_walk() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let mut scene = nf_view::load_with_carry(&iso, "07000016.bin", false, None).expect("load");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..3000 { sim.step_bond(Default::default(), false, false, false); if !sim.disabled { break; } }
    for _ in 0..100 { sim.step_bond(Default::default(), false, false, false); }
    drop(scene);
    let mut w = walker::Walker::new(nf_view::load_with_carry(&iso, "07000016.bin", false, None).expect("reload"));
    w.wait_enabled(3000, "arrival");
    w.wait(100);
    let route: &[(&str, walker::Wp)] = &[
        ("corridor", walker::Wp::new(4.3, 0.6, -72.5).r(1.2)),
        ("door7", walker::Wp::new(4.9, -0.1, -78.0).r(1.2)),
        ("east", walker::Wp::new(10.0, 0.0, -80.5).r(1.3)),
        ("ramp base", walker::Wp::new(14.5, 0.0, -88.5).r(1.3)),
        ("ramp mid", walker::Wp::new(14.5, 4.2, -82.5).r(1.3)),
        ("ring east", walker::Wp::new(14.5, 5.8, -79.0).r(1.3)),
        ("ring n2", walker::Wp::new(8.0, 5.8, -78.5).r(1.4)),
        ("ring n3", walker::Wp::new(0.0, 5.8, -78.5).r(1.4)),
        ("ring n4", walker::Wp::new(-8.0, 5.8, -80.0).r(1.4)),
        ("ring w1", walker::Wp::new(-14.0, 5.8, -84.0).r(1.4)),
        ("ring w2", walker::Wp::new(-14.0, 5.8, -92.0).r(1.4)),
        ("station", walker::Wp::at(-15.2, 5.8, -94.6, walker::Act::Use { use_ticks: 90 }).r(1.0)),
    ];
    for (label, wp) in route {
        let ok = w.try_walk_to(*wp, label);
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("[{label}] ok={ok} bond=({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
        if !ok { break; }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("ch 0x24 on: {}", sim.drones.channels.on(0x24));
    eprintln!("door[0] ch_lock now: {:#x} state {:#x}", sim.drones.doors.list[0].ch_lock, sim.drones.doors.list[0].state);
    eprintln!("stuck: {:?}", w.report.stuck_events.iter().take(6).collect::<Vec<_>>());
}

/// Isolate the east ramp climb: park Bond at the ramp's south foot and walk north up x 14.5.
#[test]
#[ignore = "assertion-free probe; run with --ignored"]
fn probe_ramp_climb() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let mut w = walker::Walker::new(nf_view::load_with_carry(&iso, "07000016.bin", false, None).expect("load"));
    w.wait_enabled(3000, "arrival");
    // Teleport to reach (allowed: the test then walks the ramp itself).
    w.scene.sim.as_mut().unwrap().bond = nf_game::player::Bond::new([14.0, 1.2, -90.5], 0.0);
    for _ in 0..30 { w.tick(Default::default(), false); }
    let p = w.scene.sim.as_ref().unwrap().bond.pos;
    eprintln!("parked at ({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
    let legs = [
        (14.5, 0.0, -89.0), (14.5, 0.6, -87.5), (14.5, 1.6, -86.0), (14.5, 2.4, -84.5),
        (14.5, 3.5, -83.0), (14.5, 4.2, -82.0), (14.5, 5.2, -80.5), (14.5, 5.9, -79.0),
        (12.0, 5.9, -78.5), (6.0, 5.9, -78.5), (0.0, 5.9, -78.5), (-8.0, 5.9, -80.0),
        (-14.0, 5.9, -84.0), (-14.0, 5.9, -92.0),
    ];
    for (i, &(x, y, z)) in legs.iter().enumerate() {
        let ok = w.try_walk_to(walker::Wp::new(x, y, z).r(1.2), &format!("leg{i} ({x},{y},{z})"));
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("[leg{i} target ({x},{y},{z})] ok={ok} bond=({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
        if !ok { break; }
    }
}

/// The full real-path station run: arrival → corridor → door[7] → east lane → the ramp's south
/// foot → up → the ring west → the SecurityStation → E → ch 0x24?
#[test]
#[ignore = "assertion-free probe; run with --ignored"]
fn probe_station_full_walk() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let mut w = walker::Walker::new(nf_view::load_with_carry(&iso, "07000016.bin", false, None).expect("load"));
    w.wait_enabled(3000, "arrival");
    w.wait(100);
    let route: &[(&str, walker::Wp)] = &[
        ("corridor", walker::Wp::new(4.3, 0.6, -72.5).r(1.2)),
        ("door7 through", walker::Wp::new(4.9, -0.1, -78.0).r(1.2)),
        ("lane east", walker::Wp::new(10.0, 0.0, -82.0).r(1.3)),
        ("lane s", walker::Wp::new(12.0, 0.0, -88.0).r(1.3)),
        ("ramp foot", walker::Wp::new(13.8, 0.0, -90.8).r(1.1)),
        ("ramp 1", walker::Wp::new(14.5, 0.6, -87.5).r(1.2)),
        ("ramp 2", walker::Wp::new(14.5, 2.4, -84.5).r(1.2)),
        ("ramp 3", walker::Wp::new(14.5, 4.6, -81.5).r(1.2)),
        ("ring top", walker::Wp::new(14.5, 5.9, -79.0).r(1.2)),
        ("ring w1", walker::Wp::new(6.0, 5.9, -78.5).r(1.4)),
        ("ring w2", walker::Wp::new(-4.0, 5.9, -79.0).r(1.4)),
        ("ring w3", walker::Wp::new(-11.0, 5.9, -81.0).r(1.4)),
        ("ring s1", walker::Wp::new(-13.0, 5.9, -86.0).r(1.4)),
        ("station w", walker::Wp::new(-16.5, 5.9, -88.0).r(1.3)),
        ("station s", walker::Wp::new(-16.3, 5.9, -93.5).r(1.2)),
        ("station E", walker::Wp::at(-15.2, 5.9, -94.6, walker::Act::Use { use_ticks: 120 }).r(1.2)),
    ];
    for (label, wp) in route {
        let ok = w.try_walk_to(*wp, label);
        let p = w.scene.sim.as_ref().unwrap().bond.pos;
        eprintln!("[{label}] ok={ok} bond=({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
        if !ok { break; }
    }
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("ch 0x24 on: {}", sim.drones.channels.on(0x24));
    eprintln!("door[0] ch_lock {:#x} state {:#x}", sim.drones.doors.list[0].ch_lock, sim.drones.doors.list[0].state);
}

/// The south room (shuttle bay) floors: where is the pit mouth, and where does the y≈1 floor
/// cross it? Also the ch-0x81 trigger's exact bbox.
#[test]
#[ignore = "assertion-free probe; run with --ignored"]
fn probe_south_room_floors() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let scene = nf_view::load(&iso, "07000016.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    for t in &sim.drones.triggers {
        if t.out == 0x81 || t.out == 0x7b {
            eprintln!("trig out {:#x} mask {:#x} lo {:?} hi {:?}", t.out, t.mask, t.lo, t.hi);
        }
    }
    let mut z = -164.0;
    while z >= -181.0 {
        let mut row = format!("z {:7.1} y1:", z);
        let mut x = 0.0;
        while x <= 6.0 {
            match sim.world.floor_below([x, 3.0, z], 4.5) {
                Some(fy) => row.push_str(&format!("{fy:5.2} ")),
                None => row.push_str("  ---  "),
            }
            x += 0.5;
        }
        eprintln!("{row}");
        z -= 1.0;
    }
}
