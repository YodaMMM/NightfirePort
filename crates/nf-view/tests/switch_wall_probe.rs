//! level-14-switch-wall probe: what the disc puts around the E-use switch oDroneTrigger1b #725
//! (-60.30, 0.61, 38.37) and how the M.Hatch indicator ScriptPlayers (script 0x06000708, triggers
//! ch 0x85/0x54) pose their objects. Assertion-free dumps feed the report; the sim test pins the
//! data/sim link.

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

fn level() -> Level {
    let iso = iso().expect("NF_ISO");
    Level::parse(&read_archive(&iso, "07000014.bin")).expect("parse")
}

const SWITCH: [f32; 3] = [-60.30, 0.61, 38.37];

/// The switch's neighbourhood on the disc: every placement of ANY kind within 4 units, the
/// resolved object (name, glist +0x44 word, bounds), and the M.Hatch family by hash.
#[test]
#[ignore = "assertion-free dump; run with --ignored"]
fn dump_near_switch_and_hatch() {
    let lv = level();
    let mm = lv.main_map().unwrap_or(0);
    let resolve = |lv: &Level, mi: usize, p: &nf_formats::placement::Placement| -> Option<(usize, usize)> {
        if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
        else if (p.index as usize) < lv.maps[mi].objects.len() { Some((mi, p.index as usize)) }
        else { None }
    };
    println!("== placements within 4.0 of the switch {SWITCH:?} ==");
    for (mi, m) in lv.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            let t = p.translation;
            let d = ((t[0] - SWITCH[0]).powi(2) + (t[1] - SWITCH[1]).powi(2) + (t[2] - SWITCH[2]).powi(2)).sqrt();
            if d > 4.0 { continue; }
            let name = resolve(&lv, mi, p).map(|(m, o)| lv.maps[m].objects[o].name.trim().to_string())
                .unwrap_or_else(|| format!("UNRESOLVED {:08x}", p.hashcode));
            let params: Vec<String> = p.params.iter().map(|(a, b)| format!("{a}:{b:#x}")).collect();
            let glist = resolve(&lv, mi, p).map(|(m, o)| {
                let ob = &lv.maps[m].objects[o];
                format!("glist+0x44 {:#x} sphere c({:.2},{:.2},{:.2}) r{:.2} leaves {}",
                    ob.flags, ob.centre[0], ob.centre[1], ob.centre[2], ob.radius, ob.gfx.leaves.len())
            }).unwrap_or_else(|| "no object".into());
            println!("m{mi} #{pi} kind {:#04x} \"{name}\" @({:.2},{:.2},{:.2}) d={d:.2} params [{}] {glist} plc-flags {:#x}",
                p.flags & 0xff, t[0], t[1], t[2], params.join(" "), p.flags);
        }
    }
    println!("== the M.Hatch family by hash ==");
    for hash in [0x0200_0724u32, 0x0200_0725, 0x0200_0a6b, 0x0200_072b] {
        match lv.by_hash.get(&hash) {
            Some(&(m, o)) => {
                let ob = &lv.maps[m].objects[o];
                println!("hash {hash:08x} m{m} #{o} \"{}\" glist+0x44 {:#x} sphere c({:.2},{:.2},{:.2}) r{:.2} leaves {} collision {}",
                    ob.name.trim(), ob.flags, ob.centre[0], ob.centre[1], ob.centre[2], ob.radius, ob.gfx.leaves.len(), ob.collision.is_some());
            }
            None => println!("hash {hash:08x} UNRESOLVED in the archive"),
        }
    }
    println!("== the M.Hatch SP placements (0xd9 with script 0x6000708) and their dummy twins ==");
    for (mi, m) in lv.maps.iter().enumerate() {
        for (pi, p) in m.placements.iter().enumerate() {
            let k = p.flags & 0xff;
            let hatchy = (k == 0xd9 && p.params.iter().any(|&(a, b)| a == 0 && b == 0x0600_0708))
                || (k == 0x31 && matches!(p.hashcode, 0x0200_0724 | 0x0200_0725));
            if !hatchy { continue; }
            let params: Vec<String> = p.params.iter().map(|(a, b)| format!("{a}:{b:#x}")).collect();
            println!("m{mi} #{pi} kind {k:#04x} @({:.2},{:.2},{:.2}) hash {:08x} plc-flags {:#x} params [{}]",
                p.translation[0], p.translation[1], p.translation[2], p.hashcode, p.flags, params.join(" "));
        }
    }
    println!("== script 0x06000708 decode ==");
    let Some(nis) = lv.nis.get(&0x0600_0708) else { println!("NOT IN THE LIB"); return };
    println!("camera {} loop {} length {} streams {} keys {}", nis.camera, nis.loop_start, nis.length, nis.streams.len(), nis.keys.len());
    for (si, st) in nis.streams.iter().enumerate() {
        println!(" s{si} first_key {} last_key {} cmds {}", st.first_key, st.last_key, st.commands.len());
        for (ki, k) in nis.keys.iter().enumerate().take(st.last_key.min(nis.keys.len() as u16) as usize).skip(st.first_key.min(nis.keys.len() as u16) as usize) {
            println!("   key {ki}: pos ({:.3},{:.3},{:.3},{:.3}) quat ({:.3},{:.3},{:.3},{:.3}) scale {:.3} w24 {:?} w28 {:.3} t {:.2}",
                k.pos[0], k.pos[1], k.pos[2], k.pos[3], k.quat[0], k.quat[1], k.quat[2], k.quat[3], k.scale, k.w24, k.w28, k.time);
        }
        for cmd in st.decode() { println!("   cmd {cmd:?}"); }
    }
}

/// The data/sim link pinned: at load the three M.Hatch SPs' mode-0x40 preloads spawn the locked
/// light 02000724 at each SP's own position (stream-0 keys × the SP base), the port's draw slots
/// exist for all three script entities, and the 0x3b console states read what the report lists.
#[test]
fn the_m_hatch_lights_spawn_at_load_and_the_console_states_match_the_disc() {
    let mut scene = nf_view::load_with_carry(&iso().expect("NF_ISO"), "07000014.bin", false, None).unwrap();
    for _ in 0..10 {
        let (c, l) = { let s = scene.sim.as_ref().expect("sim"); (s.bond.pos, [s.bond.pos[0], s.bond.pos[1], s.bond.pos[2]]) };
        scene.sim.as_mut().expect("sim").step(c, l, false, false);
    }
    let sim = scene.sim.as_ref().expect("sim");
    let ch = &sim.drones.channels;
    // The preloads leave ch 0x85/0x54 off [M SP_CreateScriptPlayer 0x1db9a8:72 switch_channels[p6]=0].
    assert!(!ch.on(0x85) && !ch.on(0x54), "the M.Hatch trigger channels start off [M 0x1db9a8:72]");
    // The locked light lives at each SP position the moment the level is up.
    let mut found = Vec::new();
    for (i, o) in sim.drones.scripts.objects.list.iter().enumerate() {
        if let Some(nf_game::nis::Obj::Entity { hash: 0x0200_0724, matrix, src, .. }) = o {
            found.push((i, *src, [matrix[3][0], matrix[3][1], matrix[3][2]]));
        }
    }
    let near = |p: [f32; 3], q: [f32; 3]| (p[0] - q[0]).abs() < 0.05 && (p[1] - q[1]).abs() < 0.05 && (p[2] - q[2]).abs() < 0.05;
    assert!(found.iter().any(|(_, _, t)| near(*t, [-52.40, 2.64, 40.14])),
        "the west locked light poses at SP #1273 (-52.40, 2.64, 40.14), got {found:?}");
    assert!(found.iter().any(|(_, _, t)| near(*t, [26.90, 2.69, -25.78])),
        "east locked light 1 poses at SP #1276 (26.90, 2.69, -25.78), got {found:?}");
    assert!(found.iter().any(|(_, _, t)| near(*t, [28.69, 2.69, -25.78])),
        "east locked light 2 poses at SP #1278 (28.69, 2.69, -25.78), got {found:?}");
    assert_eq!(found.len(), 3, "exactly the three SPs' locked lights, one per SP, got {found:?}");
    // The unlocked light and the third object are NOT live yet (they come at the replay's t30/t31).
    assert!(!sim.drones.scripts.objects.list.iter().any(|o| matches!(o, Some(nf_game::nis::Obj::Entity { hash: 0x0200_0725 | 0x0200_0a6b, .. }))),
        "no unlocked light / third object before the trigger");
    // The three electricity-console DynObjects, in placement order (the create order = the slot order).
    let dyns: Vec<_> = sim.drones.worldobjs.dynobjs.iter()
        .filter(|d| (d.pos[0] - (-62.43)).abs() < 0.1 && (d.pos[2] - 38.59).abs() < 0.1).collect();
    assert_eq!(dyns.len(), 3, "the three electricity-console 0x3b objects at (-62.43, 38.59), got {:?}", dyns.iter().map(|d| (d.ch_a, d.ch_b, d.mode)).collect::<Vec<_>>());
    // Active: no gate (p0=0), B=0x8c, mode 0 — visible from load (base flags, no 0x30 hide).
    let active = dyns.iter().find(|d| d.ch_a == 0 && d.ch_b == 0x8c).expect("CONSOLE_Electricity_Active (p0=0, p1=0x8c)");
    assert!(active.flags & 0x30 == 0, "the Active screen shows from load (no 0x30 hide word), flags {:#x}", active.flags);
    // Inactive: gated on 0x8c (created hidden), B=0x5b — hidden until the sabotage arms.
    let inactive = dyns.iter().find(|d| d.ch_a == 0x8c).expect("CONSOLE_Electricity_Inactive (p0=0x8c)");
    assert!(inactive.flags & 0x30 != 0, "the Inactive screen starts hidden (p0!=0 → |0x30 [M 0x18dbf8:27-29]), flags {:#x}", inactive.flags);
    // Static: gated on 0x5b, B=0 — hidden until 0x5b arms.
    let stat = dyns.iter().find(|d| d.ch_a == 0x5b).expect("CONSOLE_Electricity_Static (p0=0x5b)");
    assert!(stat.flags & 0x30 != 0, "the Static screen starts hidden, flags {:#x}", stat.flags);
}

#[test]
fn the_m_hatch_draw_slots_pose_and_the_0x85_replay_swaps_to_the_unlocked_light() {
    let mut scene = nf_view::load_with_carry(&iso().expect("NF_ISO"), "07000014.bin", false, None).unwrap();
    for _ in 0..10 {
        let (c, l) = { let s = scene.sim.as_ref().expect("sim"); (s.bond.pos, [s.bond.pos[0], s.bond.pos[1], s.bond.pos[2]]) };
        scene.sim.as_mut().expect("sim").step(c, l, false, false);
    }
    let opts = nf_view::scene::ViewOpts { reinforcements: false, firing: false, right: [1.0, 0.0, 0.0], up: [0.0, 1.0, 0.0] };
    let at = |scene: &nf_view::scene::Scene, want: [f32; 3]| -> bool {
        let fs = scene.frame_state_for(Some(0), 0.0, opts, true);
        fs.attach.iter().any(|m| (m[3][0] - want[0]).abs() < 0.05 && (m[3][1] - want[1]).abs() < 0.05 && (m[3][2] - want[2]).abs() < 0.05)
    };
    assert!(at(&scene, [-52.40, 2.64, 40.14]), "a posed draw slot at the west locked light (SP #947)");
    assert!(at(&scene, [26.90, 2.69, -25.78]), "a posed draw slot at east locked light 1 (SP #950)");
    assert!(at(&scene, [28.69, 2.69, -25.78]), "a posed draw slot at east locked light 2 (SP #952)");
    // The Active console screen draws through the world-instance slot pass (lock_slot), not the
    // attachment pass — its headless proof is the sim state (the visible flags word) plus the
    // build/sim slot-base alignment (sensors 14 = 14, dynobjs 45 = 45), asserted below.
    {
        let sim = scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.sensors.list.len(), 14, "the build-side sensor count matches the sim's (the dynobj slot base aligns)");
        assert_eq!(sim.drones.worldobjs.dynobjs.len(), 45, "every 0x3b placement got its DynObject");
        let active = sim.drones.worldobjs.dynobjs.iter().find(|d| d.ch_a == 0 && d.ch_b == 0x8c).unwrap();
        assert!(!active.dead && active.flags & 0x8010 == 0, "the Active screen's slot draws (not dead, no hide bits), flags {:#x}", active.flags);
    }

    // The replay: the cascade's ch 0x85 fires the west SP (mask 1 = the out-channel bit [M SP_Hit
    // 0x1dc470:17-19]); the script's t31 Entity re-points the light to 02000725, t54 sets the keep
    // flag, End(61) kills the SP — the light stays.
    let sim = scene.sim.as_mut().unwrap();
    sim.drones.channels.set_script(0x85, 1, sim.drones.frame);
    for _ in 0..80 {
        let (c, l) = { let s = scene.sim.as_ref().expect("sim"); (s.bond.pos, [s.bond.pos[0], s.bond.pos[1], s.bond.pos[2]]) };
        scene.sim.as_mut().expect("sim").step(c, l, false, false);
    }
    let sim = scene.sim.as_ref().unwrap();
    let sc = &sim.drones.scripts;
    let west = sc.objects.list.iter().flatten().find_map(|o| match o {
        nf_game::nis::Obj::Entity { hash: 0x0200_0725, matrix, src: (0x0600_0708, 0), .. } =>
            Some([matrix[3][0], matrix[3][1], matrix[3][2]]),
        _ => None });
    let near = |t: [f32; 3]| (t[0] - (-52.40)).abs() < 0.05 && (t[1] - 2.64).abs() < 0.05 && (t[2] - 40.14).abs() < 0.05;
    assert!(west.map(near).unwrap_or(false), "the west light swapped to the unlocked mesh in place, got {west:?}");
    let locked: Vec<[f32; 3]> = sc.objects.list.iter().flatten().filter_map(|o| match o {
        nf_game::nis::Obj::Entity { hash: 0x0200_0724, matrix, src: (0x0600_0708, 0), .. } =>
            Some([matrix[3][0], matrix[3][1], matrix[3][2]]),
        _ => None }).collect();
    assert_eq!(locked.len(), 2, "the east pair stays locked, got {locked:?}");
    assert!(locked.iter().any(|t| (t[0] - 26.90).abs() < 0.05) && locked.iter().any(|t| (t[0] - 28.69).abs() < 0.05),
        "the east locked lights sit at their SPs, got {locked:?}");
    // The west SP's replay ended (61 ticks) and the SP deleted itself (p4=1 → state 5 [M 0x1dbda8
    // case 2]); the kept light outlives it.
    let west_sp = sc.players.iter().find(|p| p.out == 0x85).expect("the west SP still mounted");
    assert!(west_sp.deleted, "the west SP deleted itself after the replay [M 0x1dbda8 case 2 → 5]");
    // The script's third object 02000a6b "GLOW_Light_1" (s2, parented to stream 0, t31 [D decode])
    // is LIVE in the sim — the game spawns it as a class-0 object with the gfx hash resolved through
    // the session hashtable (02000a6b lives in 0700000a.bin, not this archive [D archive scan; M
    // Script_CreateEntity 0x1d8968 fallthrough + hashtable_set_object_to_entity_gfx]). Its gfx block
    // is EMPTY on the disc (boxes 0 / leaves 0 [D PS2_GFX footer]) so neither game nor port draws
    // pixels from it — the assertion pins the spawn, not a draw.
    assert!(sc.objects.list.iter().any(|o| matches!(o, Some(nf_game::nis::Obj::Entity { hash: 0x0200_0a6b, src: (0x0600_0708, 2), .. }))),
        "the parented GLOW_Light_1 entity is live on stream 2 after the replay");
    // And the frame still poses the swapped light at the SP position.
    assert!(at(&scene, [-52.40, 2.64, 40.14]), "the unlocked light still posed after the SP died (the keep flag)");
}

#[test]
#[ignore = "scan dump; run with --ignored"]
fn scan_archives_for_hatch_hashes() {
    let iso = iso().expect("NF_ISO");
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let want = [0x0200_0724u32, 0x0200_0725, 0x0200_0a6b, 0x0200_072b];
    for e in entries.iter().filter(|e| e.name.len() >= 10 && e.name[..2].eq_ignore_ascii_case("07") && e.name.to_ascii_lowercase().ends_with(".bin")) {
        let data = img.read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64);
        let Ok(data) = data else { continue };
        let Ok(lv) = Level::parse(&data) else { continue };
        let hits: Vec<&str> = want.iter().filter(|h| lv.by_hash.contains_key(h)).map(|h| match h { _ => "x" }).collect();
        if !hits.is_empty() {
            let which: Vec<String> = want.iter().filter(|h| lv.by_hash.contains_key(h)).map(|h| format!("{h:08x}")).collect();
            println!("{}: {}", e.name, which.join(" "));
        }
    }
    println!("scan done");
}

#[test]
#[ignore = "dump; run with --ignored"]
fn dump_0a_hatch_object() {
    let iso = iso().expect("NF_ISO");
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let e = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000a.bin")).expect("0a");
    let data = img.read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64).expect("read");
    let lv = Level::parse(&data).expect("parse");
    for h in [0x0200_0724u32, 0x0200_0725, 0x0200_0a6b, 0x0200_072b] {
        if let Some(&(mi, oi)) = lv.by_hash.get(&h) {
            let ob = &lv.maps[mi].objects[oi];
            println!("hash {h:08x}: m{mi} #{oi} \"{}\" glist+0x44 {:#x} sphere c({:.2},{:.2},{:.2}) r{:.2} leaves {} batches {:?} collision {}",
                ob.name.trim(), ob.flags, ob.centre[0], ob.centre[1], ob.centre[2], ob.radius,
                ob.gfx.leaves.len(), ob.gfx.leaves.iter().map(|l| l.batches.len()).collect::<Vec<_>>(), ob.collision.is_some());
        }
    }
}

#[test]
#[ignore = "dump; run with --ignored"]
fn dump_glow_gfx_raw() {
    let iso = iso().expect("NF_ISO");
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    for (name, want_hash) in [("0700000a.bin", 0x0200_0a6bu32), ("07000005.bin", 0)] {
        let e = entries.iter().find(|e| e.name.eq_ignore_ascii_case(name));
        let Some(e) = e else { continue };
        let data = img.read_bytes(bin.lba as u64 * 2048 + e.offset as u64, e.size as u64).expect("read");
        let lv = Level::parse(&data).expect("parse");
        if want_hash != 0 {
            if let Some(&(mi, oi)) = lv.by_hash.get(&want_hash) {
                let ob = &lv.maps[mi].objects[oi];
                println!("{name} {want_hash:08x}: boxes {} leaves {} skin {:x}", ob.gfx.boxes.len(), ob.gfx.leaves.len(), ob.gfx.skin_info);
            }
        } else {
            for (mi, m) in lv.maps.iter().enumerate() {
                for (oi, ob) in m.objects.iter().enumerate() {
                    if ob.name.contains("Corona") {
                        println!("{name} m{mi} #{oi} \"{}\" boxes {} leaves {} batches {:?} glist {:#x}",
                            ob.name.trim(), ob.gfx.boxes.len(), ob.gfx.leaves.len(),
                            ob.gfx.leaves.iter().map(|l| l.batches.len()).collect::<Vec<_>>(), ob.flags);
                    }
                }
            }
        }
    }
}

#[test]
#[ignore = "dump; run with --ignored"]
fn dump_console_object_indices() {
    let lv = level();
    let mm = lv.main_map().unwrap_or(0);
    for pi in [793usize, 806, 967, 794, 807] {
        let p = &lv.maps[mm].placements[pi];
        let ob = &lv.maps[mm].objects[p.index as usize];
        println!("#{pi} kind {:#04x} -> m54 object #{} name \"{}\" obj-hashcode {:08x} glist {:#x} centre {:?} r{:.2} yaw {:.3}",
            p.flags & 0xff, p.index, ob.name.trim(), ob.hashcode, ob.flags, ob.centre, ob.radius, p.rotation[1]);
    }
}
