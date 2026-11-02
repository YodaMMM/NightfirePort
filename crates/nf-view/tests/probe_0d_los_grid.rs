//! probe (assertion-free, `--ignored`): the 0d yard LOS question for drone #31 — the walkreal
//! killer (skin 0x05000087 at (161.1,6.9,-235.1), weapon 0x1e). For a grid of ground cells on
//! the warehouse-ground walk (x 180-216, z -224..-268), which cells does the drone's eye ray
//! reach, and WHERE does the port's world put the first hit (static vs any set)? Also dumps the
//! placements standing in the corridor between the drone and the yard.
//! Run: NF_ISO=… cargo test -p nf-view --test probe_0d_los_grid -- --ignored --nocapture

#[path = "common/walker_b2.rs"]
mod walker;

use nf_formats::iso9660::IsoImage;
use nf_formats::level::Level;

fn read_archive(iso: &str, archive: &str) -> Vec<u8> {
    let img = IsoImage::open(iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case(archive)).expect("archive");
    img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read")
}

#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn probe_0d_los_grid_for_sniper_31() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");

    // Part A: the placements in the corridor box x 150..225, z -222..-272 (name, kind, pos, has-collision).
    {
        let data = read_archive(&iso, "0700000d.bin");
        let lv = Level::parse(&data).expect("parse");
        let mm = lv.main_map().expect("main map");
        eprintln!("== placements in x 150..225, z -222..-272 ==");
        for (i, p) in lv.maps[mm].placements.iter().enumerate() {
            let t = p.translation;
            if !(150.0..=225.0).contains(&t[0]) || !(-272.0..=-222.0).contains(&t[2]) { continue; }
            let k = p.flags & 0xff;
            let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < lv.maps[mm].objects.len() { Some((mm, p.index as usize)) } else { None };
            let (name, has_coll, bounds) = match target {
                Some((m, o)) => {
                    let ob = &lv.maps[m].objects[o];
                    (ob.name.trim().to_string(), ob.collision.is_some(), (ob.min, ob.max))
                }
                None => (format!("UNRESOLVED {:08x}", p.hashcode), false, ([0.0; 3], [0.0; 3])),
            };
            let params: Vec<String> = p.params.iter().map(|(a, b)| format!("{a}:{b:#x}")).collect();
            eprintln!("#{i} kind {k:#04x} \"{}\" @({:.2},{:.2},{:.2}) coll {} objbounds {:?}..{:?} params [{}] hash {:08x}",
                name, t[0], t[1], t[2], has_coll, bounds.0, bounds.1, params.join(" "), p.hashcode);
        }
    }

    // Part A2: every placement of the object classes that can carry the game's collision
    // (DynamicObject 0x3B, rigid body 0xD8, Destroy 0xE1, FuseBox 0x31, static 0x19/0x47 whose
    // name smells like a crate/container/block) in the wide box around the yard + the corridor.
    {
        let data = read_archive(&iso, "0700000d.bin");
        let lv = Level::parse(&data).expect("parse");
        let mm = lv.main_map().expect("main map");
        eprintln!("== wide sweep x 100..260, z -330..-190: 0x3B/0xD8/0xE1/0x31 + big-collision 0x19 ==");
        for (i, p) in lv.maps[mm].placements.iter().enumerate() {
            let t = p.translation;
            if !(100.0..=260.0).contains(&t[0]) || !(-330.0..=-190.0).contains(&t[2]) { continue; }
            let k = p.flags & 0xff;
            let interesting = matches!(k, 0x3b | 0xd8 | 0xe1 | 0x31);
            let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < lv.maps[mm].objects.len() { Some((mm, p.index as usize)) } else { None };
            let (name, has_coll, osize) = match target {
                Some((m, o)) => {
                    let ob = &lv.maps[m].objects[o];
                    (ob.name.trim().to_string(), ob.collision.is_some(),
                        (ob.max[0] - ob.min[0], ob.max[1] - ob.min[1], ob.max[2] - ob.min[2]))
                }
                None => (format!("UNRESOLVED {:08x}", p.hashcode), false, (0.0, 0.0, 0.0)),
            };
            let big = osize.0 > 2.0 && osize.2 > 2.0;
            if !interesting && !(big && has_coll) { continue; }
            eprintln!("#{i} kind {k:#04x} \"{}\" @({:.2},{:.2},{:.2}) coll {} size ({:.1},{:.1},{:.1}) hash {:08x}",
                name, t[0], t[1], t[2], has_coll, osize.0, osize.1, osize.2, p.hashcode);
        }
    }

    // Part B: the live sim — the drones in the region + the LOS grid with hit points.
    let mut w = walker::Walker::new(nf_view::load_with_carry(&iso, "0700000d.bin", false, None).unwrap());
    w.wait_enabled(3000, "the 0d intro");
    w.wait(60);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("== drones in x 150..225, z -222..-272 ==");
        for (i, d) in sim.drones.list.iter().enumerate() {
            if !(150.0..=225.0).contains(&d.pos[0]) || !(-272.0..=-222.0).contains(&d.pos[2]) { continue; }
            eprintln!("drone #{i} skin {:#x} weapon {:#x} pos ({:.2},{:.2},{:.2}) enabled {} los {}",
                d.skin, d.weapon, d.pos[0], d.pos[1], d.pos[2], d.enabled, d.los);
        }
    }
    let idx = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.list.iter().position(|d| d.skin == 0x0500_0087
            && (d.pos[0] - 161.1).abs() < 1.0 && (d.pos[2] + 235.1).abs() < 1.0)
            .unwrap_or_else(|| sim.drones.list.iter().position(|d| d.skin == 0x0500_0087).expect("drone 31"))
    };
    eprintln!("[grid] drone idx {idx}");

    // Part C: the cel graph — the game's sight pre-pass (`AINetwork_TestRayCels` 0x15ab78) against
    // this corridor. The drone's eye cel, the ground spots' cels, and whether the ray-BFS reaches
    // them; plus the raw cel boxes around the corridor.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let dr = &sim.drones.list[idx];
        let eye = [dr.pos[0], dr.pos[1] + 0.6, dr.pos[2]];
        let dcel = sim.world.find_cel(eye);
        eprintln!("[cels] drone cel {:?} (find_cel; stored drone cel n/a) player cel {:?} eye {:?}", dcel, sim.bond.cel, eye);
        let mut corridor: Vec<(String, [f32; 3], [f32; 3], bool)> = Vec::new();
        for (i, (lo, hi, room)) in sim.world.debug_scope_cells().iter().enumerate() {
            let over = lo[0] < 230.0 && hi[0] > 145.0 && lo[2] < -215.0 && hi[2] > -275.0;
            if over { corridor.push((format!("{i}"), *lo, *hi, *room)); }
        }
        eprintln!("[cels] {} scope cells overlap the corridor x 145..230, z -275..-215:", corridor.len());
        for (i, lo, hi, room) in corridor {
            eprintln!("[cels] cell {i} lo ({:.1},{:.1},{:.1}) hi ({:.1},{:.1},{:.1}) room {}",
                lo[0], lo[1], lo[2], hi[0], hi[1], hi[2], room);
        }
        for x in [180.0f32, 192.0, 200.0, 208.0, 214.0] {
            for z in [-268.0f32, -256.0, -244.0, -232.0] {
                for y in [-2.5f32, 0.5, 2.0] {
                    let target = [x, y, z];
                    let tc = sim.world.find_cel(target);
                    let reach = match (dcel, tc) {
                        (Some(dc), Some(_)) => {
                            let reached = sim.world.debug_ray_cels(dc, eye, target);
                            let hit = reached.iter().position(|&c| Some(c) == tc);
                            format!("target_cel {:?} bfs {} reached {:?}", tc, reached.len(), hit.is_some())
                        }
                        (None, _) => "DRONE CEL 0 → the game returns 0 (blind) [M 0x15ab78]".to_string(),
                        (_, None) => "TARGET CEL 0 → the game returns 0 (blind) [M 0x15ab78]".to_string(),
                    };
                    eprintln!("[raycels] x{x:.0} z{z:.0} y{y:.0} {reach}");
                }
            }
        }
    }

    // Sweep the warehouse ground / under-dock walk (the leg-1 route).
    for x in [180.0, 184.0, 188.0, 192.0, 196.0, 200.0, 204.0, 208.0, 212.0, 216.0] {
        for z in [-268.0, -264.0, -260.0, -256.0, -252.0, -248.0, -244.0, -240.0, -236.0, -232.0, -228.0, -224.0] {
            // The eye: the drone's head (pos + 0.6). The target: the bone heights 0.5/1.4 above the ground spot.
            for y in [-3.0f32, 0.0, 1.5] {
                {
                    let sim = w.scene.sim.as_mut().unwrap();
                    sim.bond.pos = [x, y, z];
                    sim.bond.vel = [0.0; 3];
                }
                w.scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
                let sim = w.scene.sim.as_ref().unwrap();
                let dr = &sim.drones.list[idx];
                let eye = [dr.pos[0], dr.pos[1] + 0.6, dr.pos[2]];
                let full = sim.world.segment_hit(eye, [x, y + 0.5, z]);
                let stat = sim.world.segment_hit_static(eye, [x, y + 0.5, z]);
                let fp = full.map(|(f, p)| format!("{f:.3}@({:.1},{:.1},{:.1})", p[0], p[1], p[2])).unwrap_or("CLEAR".into());
                let sp = stat.map(|(f, p)| format!("{f:.3}@({:.1},{:.1},{:.1})", p[0], p[1], p[2])).unwrap_or("CLEAR".into());
                eprintln!("[grid] x{x:.0} z{z:.0} y{y:.0} los {} dist {:.1} seen {} full {fp} stat {sp}",
                    dr.los, dr.dist, dr.seen_ticks);
            }
        }
    }
}

/// The early placements (#0-#140) across the whole map — where the shipping containers
/// (DynamicObjects #6/#7) and the level's big structures stand. Assertion-free.
#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn probe_0d_early_placements() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000d.bin")).expect("the level bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().expect("main map");
    for (i, p) in lv.maps[mm].placements.iter().enumerate() {
        if i > 140 { break; }
        let t = p.translation;
        let k = p.flags & 0xff;
        let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
            else if (p.index as usize) < lv.maps[mm].objects.len() { Some((mm, p.index as usize)) } else { None };
        let (name, has_coll, size) = match target {
            Some((m, o)) => {
                let ob = &lv.maps[m].objects[o];
                (ob.name.trim().to_string(), ob.collision.is_some(),
                    (ob.max[0] - ob.min[0], ob.max[1] - ob.min[1], ob.max[2] - ob.min[2]))
            }
            None => (format!("UNRESOLVED {:08x}", p.hashcode), false, (0.0, 0.0, 0.0)),
        };
        eprintln!("#{i} kind {k:#04x} \"{}\" @({:.2},{:.2},{:.2}) coll {} size ({:.1},{:.1},{:.1})",
            name, t[0], t[1], t[2], has_coll, size.0, size.1, size.2);
    }
}

/// The AI visibility volumes (0xF5/0xF8) of 0700000d + drone #31's sight picture at the yard
/// grid: sight_range, alert, half_fov, yaw-to-opponent, the volume factor and the visibility
/// value per cell — the game's sight gate (visibility ≥ 0.7 → the LOS ray at all). Assertion-free.
#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn probe_0d_ai_volumes_and_sight_picture() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    {
        let img = IsoImage::open(&iso).expect("open iso");
        let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000d.bin")).expect("the level bin");
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
        let lv = Level::parse(&data).expect("parse");
        let mm = lv.main_map().expect("main map");
        eprintln!("== 0xF5/0xF8 AI volume placements ==");
        for (i, p) in lv.maps[mm].placements.iter().enumerate() {
            if !matches!(p.flags, 0xf5 | 0xf8) { continue; }
            let t = p.translation;
            let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < lv.maps[mm].objects.len() { Some((mm, p.index as usize)) } else { None };
            let (name, size) = match target {
                Some((m, o)) => {
                    let ob = &lv.maps[m].objects[o];
                    (ob.name.trim().to_string(), (ob.max[0] - ob.min[0], ob.max[1] - ob.min[1], ob.max[2] - ob.min[2]))
                }
                None => (format!("UNRESOLVED {:08x}", p.hashcode), (0.0, 0.0, 0.0)),
            };
            let params: Vec<String> = p.params.iter().map(|(a, b)| format!("{a}:{b:#x}")).collect();
            eprintln!("#{i} flags {:#x} \"{}\" @({:.2},{:.2},{:.2}) size ({:.1},{:.1},{:.1}) params [{}]",
                p.flags, name, t[0], t[1], t[2], size.0, size.1, size.2, params.join(" "));
        }
    }
    let mut w = walker::Walker::new(nf_view::load_with_carry(&iso, "0700000d.bin", false, None).unwrap());
    w.wait_enabled(3000, "the 0d intro");
    w.wait(60);
    let idx = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.list.iter().position(|d| d.skin == 0x0500_0087
            && (d.pos[0] - 161.1).abs() < 1.0 && (d.pos[2] + 235.1).abs() < 1.0)
            .unwrap_or_else(|| sim.drones.list.iter().position(|d| d.skin == 0x0500_0087).expect("drone 31"))
    };
    // The volumes from the same parse the port builds (aivolume::build) — d.volumes is
    // per-tick Data, not reachable from the sim handle.
    let (vols, n_vols) = {
        let img = IsoImage::open(&iso).expect("open iso");
        let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000d.bin")).expect("the level bin");
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
        let lv = Level::parse(&data).expect("parse");
        (nf_game::aivolume::build(&lv, 0x0700_000d), ())
    };
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let dr = &sim.drones.list[idx];
        eprintln!("[sight] drone #{} skin {:#x} sight_range {} half_fov {} alert {} alert_bits {:#x} light {} vision_on {} yaw {}",
            idx, dr.skin, dr.sight_range, dr.half_fov, dr.alert, dr.alert_bits, dr.light, dr.vision_on, dr.yaw);
        let vols2 = &vols;
        let _ = vols2;
        eprintln!("[sight] volumes built: {}", vols.len());
        for (v, vol) in vols.iter().enumerate() {
            eprintln!("[sight] volume {v} centre ({:.1},{:.1},{:.1}) half ({:.1},{:.1},{:.1}) factor {} kind {} placement {}",
                vol.m[3][0], vol.m[3][1], vol.m[3][2], vol.half[0], vol.half[1], vol.half[2], vol.factor, vol.kind, vol.placement);
        }
    }
    for x in [180.0f32, 188.0, 196.0, 204.0, 212.0] {
        for z in [-268.0f32, -256.0, -244.0, -232.0] {
            for y in [0.0f32] {
                {
                    let sim = w.scene.sim.as_mut().unwrap();
                    sim.bond.pos = [x, y, z];
                    sim.bond.vel = [0.0; 3];
                }
                w.scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
                let sim = w.scene.sim.as_ref().unwrap();
                let dr = &sim.drones.list[idx];
                let vf = nf_game::aivolume::visibility_factor(&vols, [x, y, z], false);
                eprintln!("[sight] x{x:.0} z{z:.0} y{y:.0} dist {:.1} yaw_to_opp {:.2} drone_yaw {:.2} light {} vol_factor {} vis {:.3} los {}",
                    dr.dist, dr.yaw_to_opp, dr.yaw, dr.light, vf, dr.visibility(), dr.los);
            }
        }
    }
}

/// Every 0x0F oDrone placement of 0700000d with its full params (the start-state/gate/weapon
/// bytes), and the port's live state + vision bit for the three yard snipers. Assertion-free.
#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn probe_0d_odrone_params_and_states() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    {
        let img = IsoImage::open(&iso).expect("open iso");
        let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
        let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000d.bin")).expect("the level bin");
        let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
        let lv = Level::parse(&data).expect("parse");
        let mm = lv.main_map().expect("main map");
        for (i, p) in lv.maps[mm].placements.iter().enumerate() {
            if p.flags & 0xff != 0x0f { continue; }
            let t = p.translation;
            let params: Vec<String> = p.params.iter().map(|(a, b)| format!("{a}:{b:#x}")).collect();
            eprintln!("[od] #{i} @({:.2},{:.2},{:.2}) params [{}]", t[0], t[1], t[2], params.join(" "));
        }
    }
    let mut w = walker::Walker::new(nf_view::load_with_carry(&iso, "0700000d.bin", false, None).unwrap());
    w.wait_enabled(3000, "the 0d intro");
    w.wait(60);
    let sim = w.scene.sim.as_ref().unwrap();
    for (i, d) in sim.drones.list.iter().enumerate() {
        if !(150.0..=225.0).contains(&d.pos[0]) || !(-272.0..=-222.0).contains(&d.pos[2]) { continue; }
        eprintln!("[st] drone #{i} skin {:#x} weapon {:#x} pos ({:.2},{:.2},{:.2}) state {} state2 {} vision_on {} seen {} alert {:.2} present {} enabled {}",
            d.skin, d.weapon, d.pos[0], d.pos[1], d.pos[2], d.state, d.prev_state, d.vision_on, d.seen, d.alert, d.present, d.enabled);
    }
}

/// The visibility field (drone #31's `visibility()` + los) along the direct east strip from the
/// warehouse-ground drop to the ladder base (x 178..218, z -224..-244) — where is the blind
/// corridor (vis < 0.7, no LOS probe [M 0x176a28]), what floor does the walker stand on. Assertion-free.
#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn probe_0d_vis_field_east_strip() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut w = walker::Walker::new(nf_view::load_with_carry(&iso, "0700000d.bin", false, None).unwrap());
    w.wait_enabled(3000, "the 0d intro");
    w.wait(60);
    let idx = {
        let sim = w.scene.sim.as_ref().unwrap();
        sim.drones.list.iter().position(|d| d.skin == 0x0500_0087
            && (d.pos[0] - 161.1).abs() < 1.0 && (d.pos[2] + 235.1).abs() < 1.0)
            .unwrap_or_else(|| sim.drones.list.iter().position(|d| d.skin == 0x0500_0087).expect("drone 31"))
    };
    for x in [178.0f32, 182.0, 186.0, 190.0, 194.0, 198.0, 202.0, 206.0, 210.0, 214.0, 218.0] {
        for z in [-224.0f32, -227.0, -230.0, -233.0, -236.0, -240.0, -244.0] {
            // The ground spot: find the floor near the expected heights.
            let sim = w.scene.sim.as_ref().unwrap();
            let floor = sim.world.floor_below([x, 3.0, z], 8.0);
            let y = floor.unwrap_or(0.0);
            drop(sim);
            {
                let sim = w.scene.sim.as_mut().unwrap();
                sim.bond.pos = [x, y, z];
                sim.bond.vel = [0.0; 3];
            }
            w.scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
            let sim = w.scene.sim.as_ref().unwrap();
            let dr = &sim.drones.list[idx];
            eprintln!("[vis] x{x:.0} z{z:.0} floor {y:.2} dist {:.1} vis {:.3} los {}",
                dr.dist, dr.visibility(), dr.los);
        }
    }
}

/// The trench pit floor map around the pin spot (186.3, −3.4, −265.9) — the floor heights and
/// what blocks the eastward walk to the old waypoint (190, −3, −266). Assertion-free.
#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn probe_0d_trench_pit_map() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut w = walker::Walker::new(nf_view::load_with_carry(&iso, "0700000d.bin", false, None).unwrap());
    w.wait_enabled(3000, "the 0d intro");
    w.wait(60);
    let sim = w.scene.sim.as_ref().unwrap();
    eprintln!("== pit floors (floor_below from y 2) x 180..200, z -270..-258 ==");
    for x in [180.0f32, 183.0, 186.0, 189.0, 192.0, 195.0, 198.0] {
        let mut row = String::new();
        for z in [-270.0f32, -268.0, -266.0, -264.0, -262.0, -260.0, -258.0] {
            let f = sim.world.floor_below([x, 2.0, z], 8.0);
            row.push_str(&format!("({x:.0},{z:.0})={:.2} ", f.unwrap_or(f32::NAN)));
        }
        eprintln!("[pit] {row}");
    }
    eprintln!("== rays east at Bond's heights from (186.3, y, -265.9) ==");
    for y in [-3.4f32, -2.9, -2.4, -1.9, -1.4, -0.9] {
        let hit = sim.world.segment_hit([186.3, y, -265.9], [190.5, y, -266.0]);
        let hit2 = sim.world.segment_hit([186.3, y, -265.9], [191.5, y, -263.0]);
        eprintln!("[pit] y{y} east {hit:?} ne {hit2:?}");
    }
    eprintln!("== up-columns at the pin (186.3,-265.9) and east (190,-266) ==");
    for (x, z) in [(184.0f32, -265.9), (186.3, -265.9), (188.0, -265.9), (190.0, -266.0), (192.0, -266.0)] {
        let mut hits = Vec::new();
        let mut bot = -5.0f32;
        for _ in 0..8 {
            match sim.world.segment_hit([x, bot, z], [x, 4.0, z]) {
                Some((f, p)) => { let y = bot + (4.0 - bot) * f; hits.push(y); bot = y + 0.05; }
                None => break,
            }
        }
        eprintln!("[pit] col ({x},{z}) surfaces {hits:?}");
    }
}

/// The 0xF0 pickup placements of 0700000d (kind/param/position) — where the BodyArmourHeavy
/// and the sniper ammo stand. Assertion-free.
#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn probe_0d_pickups() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let img = IsoImage::open(&iso).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("0700000d.bin")).expect("the level bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lv = Level::parse(&data).expect("parse");
    let mm = lv.main_map().expect("main map");
    for (i, p) in lv.maps[mm].placements.iter().enumerate() {
        if p.flags & 0xff != 0xf0 { continue; }
        let t = p.translation;
        let target = if p.by_hash() { lv.by_hash.get(&p.hashcode).copied() }
            else if (p.index as usize) < lv.maps[mm].objects.len() { Some((mm, p.index as usize)) } else { None };
        let name = match target {
            Some((m, o)) => lv.maps[m].objects[o].name.trim().to_string(),
            None => format!("UNRESOLVED {:08x}", p.hashcode),
        };
        let params: Vec<String> = p.params.iter().map(|(a, b)| format!("{a}:{b:#x}")).collect();
        eprintln!("[pk] #{i} \"{}\" @({:.2},{:.2},{:.2}) params [{}]", name, t[0], t[1], t[2], params.join(" "));
    }
}

/// The armour pickup's life: walk to #525, print armour; take the platform drop, print armour;
/// stand at the ladder base, print armour. Assertion-free.
#[test]
#[ignore = "assertion-free probe; run with --ignored --nocapture"]
fn probe_0d_armour_timeline() {
    let iso = std::env::var("NF_ISO").expect("NF_ISO");
    let mut w = walker::Walker::new(nf_view::load_with_carry(&iso, "0700000d.bin", false, None).unwrap());
    for c in [0x1bu8, 0x1c] { w.scene.sim.as_mut().unwrap().drones.set_channel(c); }
    w.wait_enabled(3000, "the intro");
    w.wait(120);
    let a = |w: &walker::Walker, what: &str| {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[arm] {what}: health {:.1} armour {:.1} pos ({:.2},{:.2},{:.2})",
            sim.drones.health.health, sim.drones.health.armour, sim.bond.pos[0], sim.bond.pos[1], sim.bond.pos[2]);
    };
    a(&w, "after intro");
    w.walk_to(walker::Wp::new(180.6, 11.2, -208.8).r(0.9), "the BodyArmourHeavy #525");
    a(&w, "after pickup leg");
    w.wait(30);
    a(&w, "after 30 wait");
    assert!(w.try_walk_to(walker::Wp { at: [181.0, 7.5, -227.5], act: walker::Act::None, arrive: 1.5 }, "the platform drop"), "drop");
    a(&w, "after drop");
    for (at, label) in [([188.0f32, 0.5, -228.0], "strip 1"), ([196.0, 0.5, -228.5], "strip 2"), ([204.0, 0.5, -229.0], "strip 3")] {
        assert!(w.try_walk_to(walker::Wp { at, act: walker::Act::None, arrive: 2.0 }, label), "{}", label);
        a(&w, label);
    }
}
