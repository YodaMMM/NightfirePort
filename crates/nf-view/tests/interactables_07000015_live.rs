//! level-15 interactables: one live test per interactive KIND on 07000015 (the missile base,
//! the silo) through the real input path (`nf_view::load` + the Walker; teleports only to REACH a
//! spot — every operation itself is the real input). Evidence tags in every assert;
//! docs/levels/07000015.md §7 is the census.
#[path = "common/walker.rs"]
mod walker;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load15() -> Walker {
    // Difficulty 3 (00 Agent): the three keypad locks carry param 6 = 1 — the Operative
    // create-unlock tag [M Lock_Create 0x1ae1f8 tail: tag+0x44 != 0 && GameState+0x28 == 1 →
    // Lock_Unlock] — and the burn leg pins the Agent+ behaviour, so the tests run past the
    // auto-unlock (the same choice walkreal_07000012 made).
    let mut w = Walker::new(nf_view::load_with_carry_medals(&iso(), "07000015.bin", false, None, 3, u64::MAX /* kind-6 pickups need the gold medal [M 0x18dda0:93] */).unwrap());
    assert!(w.wait_enabled(3000, "the silo arrival"), "the intro releases Bond");
    w.wait(120);
    w
}

/// The spawn TouchOnce #814 (out 0x4b) — the objective reveal — fires on the spawn gantry, and the
/// mission announces 'Find the exit to Omega Sector' [L record 15]. Sister: the east room's alert
/// channel 0xf6 stays off standing here [D #717's volume sits at x 80.8].
#[test]
fn the_spawn_touch_reveals_the_objective() {
    let mut w = load15();
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.channels.on(0x4b), "the spawn plate #814 fires ch 0x4b [D placement params]");
    assert_eq!(sim.drones.mission.states[21][0], 3, "'Find the exit to Omega Sector' announces on the 0x4b enable [L record 15]");
    assert!(!sim.drones.channels.on(0xf6), "sister: the east room's 0xf6 alert stays off at the spawn [D]");
}

/// The room-alert TouchOnce chain, through the REAL walk east along the gantry: the plate #717
/// (80.81, 41.79, 0.08) writes 0xf6, which wakes the room's drones (params 3 = 0xf6 [D]), arms the
/// Apocalypse #710 (ch 0xf6, 2 s whiteout [D params]) and starts the spawner #718 (start ch 0xf6).
fn templates(sim: &nf_view::scene::Sim) -> impl Iterator<Item = &nf_game::drone::Drone> {
    sim.drones.list.iter().filter(|d|
        walker::dxz(d.pos, [50.10, 41.46, -0.08]) < 1.0 || walker::dxz(d.pos, [56.73, 41.45, 0.06]) < 1.0)
}

#[test]
fn the_east_room_touch_wakes_its_squad() {
    let mut w = load15();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // Sister: the level's one spawner (group 1, start ch 0xf6, mode 3, max 2 [D census params])
        // holds before the touch, and its group's templates — the two 0xf6-wake drones by their
        // spots (50.10, 41.46, −0.08)/(56.73, 41.45, 0.06) [D] — are absent from the world: the
        // spawner's drones are templates until `DroneSpawner_Control` brings them in
        // [M DroneSpawner_Init — the port's parse notes "their group's drones become templates,
        // absent until spawned"].
        assert_eq!(sim.drones.spawners.len(), 1, "the one AI_DroneSpawner placement [D census]");
        let s = &sim.drones.spawners[0];
        assert_eq!(s.start, 0xf6, "the spawner waits on ch 0xf6 [D param 4]");
        assert!(s.spawned.is_empty() && !s.active, "sister: the spawner holds before the touch [M 0x137e68]");
        assert!(templates(&sim).count() >= 2, "the group-1 templates load as records [M Drone_Create 0x136be0]");
        assert!(templates(&sim).all(|d| !d.present),
            "sister: the templates are absent from the world before the touch [M DroneSpawner_Init]");
    }
    w.walk_to(Wp::new(88.0, 41.4, -0.5).r(1.2), "the gantry west");
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(!sim.drones.channels.on(0xf6), "sister: 0xf6 off before the east walk [D]");
    w.walk_to(Wp::new(80.5, 41.4, 0.08).r(1.2), "into touch #717");
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.channels.on(0xf6), "the east room's plate #717 fires ch 0xf6 on the walk [D volume (80.81, 41.79, 0.08)]");
    let apoc = sim.drones.fx.apocalypses.iter().find(|a| a.channel == 0xf6).expect("the 0xf6 apocalypse");
    assert!(apoc.armed, "the Apocalypse #710 latches armed on ch 0xf6 [M Apocalypse_Update 0x18d4f0]");
    assert!(apoc.fade > 0.0, "the whiteout mirror runs (remaining·5/total [M 0x18d4f0])");
    // The spawner's start channel fires: the group-1 templates come in (mode 3, max 2 [D params];
    // DroneSpawner_Control 0x137e68's trigger path) and stand in the world. The spawn runs the
    // tick after the touch's tick, so wait bounded ticks for it.
    let mut came = false;
    for _ in 0..120 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.spawners[0].spawned.iter().any(|&i| sim.drones.list[i].present) { came = true; break; }
        w.tick(nf_game::player::Input::default(), false);
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(came, "the ch-0xf6 start brings the group-1 templates in [M DroneSpawner_Control 0x137e68's trigger path]");
    // Mode 3 spawns only when none of its own are alive [M 0x137e68's mode arm `living == 0`], so
    // the first tick brings ONE drone in; the second follows once the first dies (max alive 2 [D
    // param 2]). The initial-fill flag is clear (start 0xf6 was off at create [M 0x137aa0:33-45]),
    // so no immediate fill.
    assert_eq!(sim.drones.spawners[0].spawned.len(), 1, "the first template spawns (mode 3 holds the second until none is alive [M 0x137e68])");
    assert!(sim.drones.spawners[0].spawned.iter().all(|&i| sim.drones.list[i].present),
        "the spawned drone stands in the world [M 0x137aa0's live-slot array]");
    assert!(sim.drones.spawners[0].spawned.iter().all(|&i| sim.drones.list[i].switch_channel == 0xf6 && sim.drones.list[i].channel_on),
        "the spawned squad wakes on the same ch 0xf6 [D drone param 3; M the +0x134 wake read]");
}

/// The pickups take on the walk-over: the Pickup 007 #813 (53.40, 40.57, 71.14) sits on the
/// north band east of the locked keypad door #685, the Pickup 007 #812 (−1.15, y, −7.81) in the
/// exit hall. The route is the walkreal_07000015_live lane (GREEN end to end): the north lane
/// doors, the x34.6 spine's west plank, the NAV GRAPH's east→north→west wrap round the console
/// (map54 #145 walls the south-east approach to door[32] [D plc + the probe's floor grid]) — the
/// direct north-band leg the first run attempted wedges at the room mouths ([D the floor grid:
/// z 67 is void east of x 44]). The band is SPLIT by #685's closed leaf ([L the live door[33]
/// record: sph (46.13, 41.52, 71.18) r 1.764 — the leaf spans the N-S doorway plane]), so the
/// walk takes the level's own gate: the #711 keypad (45.89, 41.96, 69.58, out 0xf9 [D census])
/// burns through the held Q-Decryptor beam in the doorway's west pocket ([D the pocket floor
/// x 43.3..45.9, z 68.6..71.2]), the rising edge opens #685 [M Door_Update's lock path], and the
/// band's east side carries the crate. The take consumes the pickup
/// [M Pickup_Handler 0x18e650 case 6 → PlrStat_LogBondBonus 0x1999e0].
#[test]
fn the_pickups_take_on_the_walk() {
    let mut w = load15();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(walker::pickups(sim).iter().any(|(p, _, _)| walker::dxz(*p, [53.40, 40.57, 71.14]) < 1.5),
            "sister: the 007 pickup #813 still sits on its crate [D]");
        assert!(walker::pickups(sim).iter().any(|(p, _, _)| walker::dxz(*p, [-1.15, 41.3, -7.81]) < 1.5),
            "sister: the 007 pickup #812 still sits in the exit hall [D]");
    }
    w.walk_to(Wp::new(88.0, 41.4, -0.5).r(1.2), "the gantry west");
    assert!(w.follow_nav_route([34.6, 41.4, 6.0], "the silo crossing"), "the nav route crosses the gantry");
    w.walk_to(Wp::new(34.6, 41.4, 6.0).r(1.2), "the north turn");
    w.walk_to(Wp::new(34.6, 41.4, 16.0).r(1.2), "door[38] north");
    w.walk_to(Wp::new(34.6, 41.4, 28.0).r(1.2), "door[39] north");
    w.walk_to(Wp::new(29.0, 41.4, 31.0).r(1.2), "the west plank");
    w.walk_to(Wp::new(28.0, 41.4, 35.0).r(1.2), "the plank north");
    w.walk_to(Wp::new(28.5, 41.4, 39.0).r(1.2), "the plank north 2");
    w.walk_to(Wp::new(30.5, 41.4, 42.0).r(1.2), "the spine north");
    w.walk_to(Wp::new(34.6, 41.4, 46.5).r(1.2), "door[46] north");
    w.walk_to(Wp::new(34.6, 41.4, 52.0).r(1.2), "door[46] through");
    w.walk_to(Wp::new(34.6, 41.4, 65.0).r(1.2), "door[47] north");
    // The north band east: the room mouths wedge the direct line; walk the nav lane's
    // east→north wrap, then the band's own z ≈ 71.2 lane east to the 007 crate.
    w.walk_to(Wp::new(37.6, 41.4, 66.3).r(1.2), "the nav lane east");
    w.walk_to(Wp::new(41.6, 41.4, 69.2).r(1.2), "the nav lane north");
    w.walk_to(Wp::new(41.9, 41.4, 71.2).r(1.2), "the nav lane top");
    // The band is SPLIT by the locked keypad door #685: its closed leaf spans z 69.4..72.9
    // along the N-S doorway plane at x 46.13 ([L the live door[33] record: sph (46.13, 41.52,
    // 71.18) r 1.764 — the leaf's half-length along the slide axis]). The level's own gate is
    // the #711 keypad (45.89, 41.96, 69.58, out 0xf9 [D census]) on the doorway's west pocket
    // (the pocket floor 40.28 spans x 43.3..45.9, z 68.6..71.2 [D tris]) — the same burn chain
    // the_keypad_burn_unlocks_its_door proves for #683/0x95. So: walk south into the pocket,
    // burn the lock through the held Q-Decryptor beam, walk east through the opened doorway.
    w.walk_to(Wp::new(44.5, 41.4, 70.3).r(0.9), "the lock pocket");
    {
        let lock_at = [45.89f32, 41.96, 69.58];
        {
            let sim = w.scene.sim.as_ref().unwrap();
            assert!(!sim.drones.channels.on(0xf9), "sister: ch 0xf9 off before the burn [D]");
            assert!(sim.drones.locks.iter().any(|l| l.out == 0xf9 && !l.unlocked && !l.burned),
                "the #711 keypad sits locked [M Lock_Create 0x1ae1f8]");
        }
        let mut cycles = 0;
        loop {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.gun.weapon == 0x4e { break; }
            assert!(cycles < 14, "the G cycle never reached the Q-Decryptor (0x4e)");
            w.tick(nf_game::player::Input { gadget_cycle: 1, ..Default::default() }, false);
            for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
            cycles += 1;
        }
        let start = w.total_ticks;
        loop {
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.drones.channels.on(0xf9) { break; }
            assert!(w.total_ticks - start < 600, "the beam never burned the #711 keypad (ch 0xf9) [M 0x1ae3d8:194]");
            let p = sim.bond.pos;
            let eye = p[1] + sim.bond.eye - sim.bond.crouch_drop;
            let dx = lock_at[0] - p[0];
            let dz = lock_at[2] - p[2];
            let dy = lock_at[1] - eye;
            let desired = dx.atan2(dz);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let want_pitch = (dy / walker::dxz(p, lock_at).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
            let input = nf_game::player::Input {
                turn: err.clamp(-0.05, 0.05),
                look: (want_pitch - sim.bond.pitch).clamp(-0.05, 0.05),
                ..Default::default()
            };
            w.tick_fire(input);
        }
        // The rising edge opens the door [M Door_Update's lock path].
        for _ in 0..90 { w.tick(nf_game::player::Input::default(), false); }
        let sim = w.scene.sim.as_ref().unwrap();
        let (_, di) = walker::door_near(sim, [46.13, 41.5, 70.13]).expect("door #685 near the burned lock");
        assert!(sim.drones.doors.list[di].state & 0xc != 0,
            "ch 0xf9's rising edge opens door #685 [M Door_Update's lock path]");
    }
    w.walk_to(Wp::new(47.5, 41.4, 70.2).r(1.0), "door[685] through east");
    w.walk_to(Wp::new(53.40, 41.4, 71.2).r(0.8), "the 007 crate east");
    // The return west: back through the (re-openable) doorway, then the walkreal's proven wrap
    // round the console to door[32]'s north approach, the west room, door[52], the south
    // corridor, and the split east to the exit hall's 007 #812.
    w.walk_to(Wp::new(44.5, 41.4, 70.3).r(1.2), "the pocket west");
    w.walk_to(Wp::new(41.9, 41.4, 71.2).r(1.2), "the nav lane top");
    w.walk_to(Wp::new(37.6, 41.4, 76.9).r(1.2), "the nav lane west");
    w.walk_to(Wp::new(30.5, 41.4, 74.6).r(1.2), "the nav lane southwest");
    w.walk_to(Wp::new(28.7, 41.4, 71.5).r(1.0), "the doorway north side");
    w.walk_to(Wp::new(26.2, 41.4, 71.2).r(0.9), "door[32] threshold");
    w.walk_to(Wp::new(23.0, 41.4, 71.2).r(0.9), "door[32] through");
    w.walk_to(Wp::new(20.0, 41.4, 70.0).r(1.2), "door[32] west-top");
    w.walk_to(Wp::new(13.0, 41.4, 71.0).r(1.2), "the west lane");
    w.walk_to(Wp::new(11.5, 41.4, 71.2).r(0.9), "door[29] threshold");
    w.walk_to(Wp::new(8.0, 41.4, 71.2).r(1.0), "door[29] through");
    w.walk_to(Wp::new(6.0, 41.4, 68.5).r(1.0), "the southwest jog");
    w.walk_to(Wp::new(6.0, 41.4, 66.0).r(1.2), "the west room");
    w.walk_to(Wp::new(1.0, 41.4, 63.0).r(1.2), "the west room south");
    w.walk_to(Wp::new(1.0, 41.4, 61.5).r(0.8), "door[52] threshold");
    w.walk_to(Wp::new(0.5, 41.4, 59.0).r(1.0), "door[52] through");
    w.walk_to(Wp::new(0.0, 41.4, 48.0).r(1.2), "the corridor south");
    w.walk_to(Wp::new(0.0, 41.4, 47.0).r(0.8), "door[50] threshold");
    w.walk_to(Wp::new(0.0, 41.4, 44.0).r(1.0), "door[50] through");
    w.walk_to(Wp::new(5.0, 41.4, 42.0).r(1.2), "the hall east");
    w.walk_to(Wp::new(6.5, 41.4, 34.0).r(1.2), "the east bridge");
    w.walk_to(Wp::new(2.0, 41.4, 30.0).r(1.2), "the hall south");
    w.walk_to(Wp::new(0.0, 41.4, 26.0).r(0.9), "door[42] threshold");
    w.walk_to(Wp::new(0.0, 41.4, 23.0).r(1.0), "door[42] through");
    w.walk_to(Wp::new(0.0, 41.4, 11.5).r(0.9), "door[51] threshold");
    w.walk_to(Wp::new(0.0, 41.4, 8.5).r(1.0), "door[51] through");
    w.walk_to(Wp::new(5.0, 41.4, 4.0).r(1.0), "the split east");
    w.walk_to(Wp::new(5.0, 41.4, -4.0).r(1.0), "the split south");
    w.walk_to(Wp::new(1.0, 41.4, -7.0).r(1.0), "the hall mouth");
    w.walk_to(Wp::new(-1.15, 41.4, -7.81).r(0.8), "the exit hall 007");
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(!walker::pickups(sim).iter().any(|(p, _, _)| walker::dxz(*p, [53.40, 40.57, 71.14]) < 1.5),
        "the Pickup 007 #813 takes on the walk-over (the take consumes it [M 0x18e650 case 6])");
    assert!(!walker::pickups(sim).iter().any(|(p, _, _)| walker::dxz(*p, [-1.15, 41.3, -7.81]) < 1.5),
        "the Pickup 007 #812 takes on the walk-over (the take consumes it [M 0x18e650 case 6])");
}

/// The plain E door opens; the 0xc8-keypad door refuses (ch 0xc8 has no writer on the level [D
/// census — no trigger/lock/SS writes it], the same dead-keypad family as 07000013's).
#[test]
fn plain_door_opens_and_the_dead_keypad_door_refuses() {
    let mut w = load15();
    // Door #694 (door_D8A_D11A at (−1.05, 41.59, 25.06), all params permissive [D]) — the west
    // spine's door. Teleport to reach, then the real E.
    w.scene.sim.as_mut().unwrap().bond.pos = [-1.05, 41.5, 23.4];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    let (di, before) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let (_, di) = walker::door_near(sim, [-1.05, 41.5, 24.5]).expect("door #694 near");
        (di, sim.drones.doors.list[di].state)
    };
    assert_eq!(before & 0xc, 0, "sister: the plain door starts shut [D]");
    w.use_door(di, "15: the plain west door");
    for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.doors.list[di].state & 0xc != 0, "the E opens the unlocked kind-2 door [M Doors::player_use]");
    // The dead-keypad door #686 (door_D12C_D13Cc at (60.71, 41.59, 70.14), ch_lock 0xc8 [D]).
    w.scene.sim.as_mut().unwrap().bond.pos = [60.71, 41.5, 68.5];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    let (_, di2) = {
        let sim = w.scene.sim.as_ref().unwrap();
        walker::door_near(sim, [60.71, 41.5, 69.5]).expect("door #686 near")
    };
    w.use_door(di2, "15: the 0xc8 door");
    for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    assert_eq!(sim.drones.doors.list[di2].state & 0xc, 0,
        "the 0xc8 door refuses E: its lock channel has no writer on the level [D census]");
}

/// The keypad burn — the census §6 open item, now proven: the Q-Decryptor (0x4e, owned by the
/// level's loadout [D probe: owned 78]) held via the real G cycle burns the KeypadLocked lock
/// #458 (out ch 0x95 [D]) through the held beam, the rising edge opens the door #683 (ch_lock
/// 0x95 [D]) — the same chain walkreal_07000012 proved on its own keypads.
#[test]
fn the_keypad_burn_unlocks_its_door() {
    let mut w = load15();
    // Reach: the lock #458 (10.29, 41.96, 33.76) on door #683's south face. Teleport to reach;
    // the burn itself is the real G cycle + the held fire.
    w.scene.sim.as_mut().unwrap().bond.pos = [10.3, 41.5, 32.4];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x95), "sister: ch 0x95 off before the burn [D]");
        assert!(sim.drones.locks.iter().any(|l| l.out == 0x95 && !l.unlocked && !l.burned),
            "the keypad lock sits locked [M Lock_Create 0x1ae1f8]");
    }
    // The real G cycle equips the Q-Decryptor.
    let mut cycles = 0;
    loop {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.gun.weapon == 0x4e { break; }
        assert!(cycles < 14, "the G cycle never reached the Q-Decryptor (0x4e)");
        w.tick(nf_game::player::Input { gadget_cycle: 1, ..Default::default() }, false);
        for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
        cycles += 1;
    }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, 0x4e, "the Q-Decryptor is held [M runs_beam]");
    // The held beam burns the lock (Lock_Update 0x1ae3d8 case 0's weapon band → the out write
    // :194-195).
    let lock_at = [10.29f32, 41.96, 33.76];
    let start = w.total_ticks;
    loop {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.channels.on(0x95) { break; }
        assert!(w.total_ticks - start < 600, "the beam never burned the keypad lock (ch 0x95) [M 0x1ae3d8:194]");
        let p = sim.bond.pos;
        let eye = p[1] + sim.bond.eye - sim.bond.crouch_drop;
        let dx = lock_at[0] - p[0];
        let dz = lock_at[2] - p[2];
        let dy = lock_at[1] - eye;
        let desired = dx.atan2(dz);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let want_pitch = (dy / walker::dxz(p, lock_at).max(0.5)).atan() / std::f32::consts::FRAC_PI_2;
        let input = nf_game::player::Input {
            turn: err.clamp(-0.05, 0.05),
            look: (want_pitch - sim.bond.pitch).clamp(-0.05, 0.05),
            ..Default::default()
        };
        w.tick_fire(input);
    }
    // The rising edge opens the door (Door_Update's lock path — walkreal-12's two-latch fix).
    for _ in 0..90 { w.tick(nf_game::player::Input::default(), false); }
    let sim = w.scene.sim.as_ref().unwrap();
    let (_, di) = walker::door_near(sim, [10.51, 41.5, 34.54]).expect("door #683 near the burned lock");
    assert!(sim.drones.doors.list[di].state & 0xc != 0,
        "ch 0x95's rising edge opens door #683 [M Door_Update's lock path]");
}

/// The silo-fall kill: the death plates (0xE4, kind 6, damage 1000, one-shot [D #794-#801/#855])
/// sit at y 38.57 under each silo shaft. The east room's plate #794 (71.14, 38.57, −0.01): the
/// walked west lane from the proven east-room spot (the touch #717 leg) crosses the (71, 0)
/// shaft's east rim — the walkreal reproduced this fall family on the z ≈ 0 lane. The damage
/// suppression lifts for the fall leg (the route's guards otherwise shoot the walk dead).
#[test]
fn the_silo_fall_dies_on_the_death_plate() {
    let mut w = load15();
    w.walk_to(Wp::new(88.0, 41.4, -0.5).r(1.2), "the gantry west");
    w.walk_to(Wp::new(80.5, 41.4, 0.08).r(1.2), "the east room");
    w.top_up_health = false;
    // Reach: the shaft's airspace over plate #794 (71.14, 38.57, −0.01) — a reach teleport (the
    // gantry rims' rail_collision stops every probed west line; the fall itself is the real
    // physics below).
    w.scene.sim.as_mut().unwrap().bond.pos = [71.0, 41.0, 0.0];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    let mut fell = false;
    for _ in 0..300 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.pos[1] < 40.0 { fell = true; break; }
        w.tick(nf_game::player::Input::default(), false);
    }
    assert!(fell, "the shaft drop never began");
    let mut hurt = false;
    for _ in 0..600 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.health.health < 100.0 || sim.drones.death.active { hurt = true; break; }
        w.tick(nf_game::player::Input::default(), false);
    }
    assert!(hurt, "the shaft fall never hurt — the death plate #794 (y 38.57, damage 1000 [D]) must catch the fall [M Hurt_Update 0x17e5d0]");
}

/// The shooter + the sound trigger, both armed by the same room touch #720 (out 0xfb [D]): the
/// turret "ShootyBangBang" #719 fires its one Bullet_init (weapon 0x34 [D]) and the oSoundTrigger
/// #780 plays sound 1402 [D id 0x57a].
#[test]
fn the_shooter_fires_and_the_sound_plays_on_the_room_touch() {
    let mut w = load15();
    w.scene.sim.as_mut().unwrap().bond.pos = [35.5, 41.5, 47.5];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0xfb), "sister: 0xfb off before the walk [D]");
        assert!(!sim.drones.worldobjs.shooters[0].fired, "sister: the turret has not fired [M Shooter_Update 0x17d0c0]");
    }
    w.walk_to(Wp::new(35.68, 41.4, 50.5).r(1.0), "into touch #720");
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.channels.on(0xfb), "the room plate #720 fires ch 0xfb on the walk [D]");
    assert!(sim.drones.worldobjs.shooters[0].fired,
        "the turret fires its one shot on the arm edge [M Shooter_Update 0x17d0c0]");
    assert!(sim.drones.sound_triggers.playing(1402),
        "the sound trigger #780 plays 1402 while ch 0xfb holds [M SoundTrigger_Update 0x1b2790]");
}

/// The mines (0xFE, the port's new Mine list): walking into the doorway laser fence at z ≈ 18
/// (the floor beams #384/#385 span x 32.9..38.3 at belt height [D probe]) explodes the mine —
/// the blast records, the camera shakes, the mine deletes [M Mine_Update 0x17c930].
#[test]
fn the_mine_explodes_when_bond_walks_into_its_beam() {
    let mut w = load15();
    w.scene.sim.as_mut().unwrap().bond.pos = [35.5, 41.5, 15.0];
    w.scene.sim.as_mut().unwrap().bond.yaw = 0.0; // face +z, into the z-18 doorway fence
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.mines.len() == 9, "the nine Tripbomb_3rd cels create [D census; M Mine_Create 0x17ce28]");
        assert!(sim.drones.mines.iter().all(|m| !m.exploded), "sister: no mine has exploded standing here");
        assert!(sim.drones.explosions.is_empty(), "sister: no blast recorded");
    }
    // Walk north through the doorway fence; capture the blast on the boom tick (the explosions
    // list drains each tick — the view consumes it).
    let mut boom = None;
    let mut boom_blasts = 0usize;
    let mut boom_shake = 0.0f32;
    for _ in 0..400 {
        let sim = w.scene.sim.as_ref().unwrap();
        if let Some(m) = sim.drones.mines.iter().position(|m| m.exploded) {
            boom = Some(m);
            boom_blasts = sim.drones.explosions.len();
            boom_shake = sim.drones.shake;
            break;
        }
        let input = nf_game::player::Input { forward: 1.0, ..Default::default() };
        w.tick(input, false);
    }
    let boom = boom.expect("walking the z-18 doorway never exploded a mine — the beam contact must fire [M 0x17c930's hit-list arm]");
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(boom_shake > 0.0, "Camera_Shake(param1) fires on the explode tick [M 0x17c930's Camera_Shake arm] (the explosions list drains at the tick's end — the view consumes it, the shake survives)");
    let m = &sim.drones.mines[boom];
    assert!((m.radius - 10.0).abs() < 0.01 && (m.damage - 80.0).abs() < 0.01,
        "the blast carries the disc's radius 10 / damage 80 [D p1/p0]");
}

/// explode-debris-spawn: the mine's blast is a persistent object now, and it spawns NO debris.
/// The game chain: `Mine_Update` 0x17c930 inlines `Explode_Create` and stores the u16 debris
/// kind 0 [M 0x17c930:128], so `Explode_Update`'s debris arm returns BEFORE its Rand_Rand(5)
/// draw on every tick [M 0x17c350:25-27] — mine blasts never spawn `Debris_CreateEx` particles
/// and never touch the shared RNG stream. Same for the oil barrels: `Destroy_Smash`'s 12th arg
/// is an explicit 0 [M 0x12bcf8 `sw $zero,($sp)`]. The blast object itself lives while its
/// script plays: the standard blast script's `SCRIPTINFO+0xAB0` u16 is the lifetime [M 0x17cd18].
#[test]
fn the_mine_blast_is_a_persistent_object_that_spawns_no_debris() {
    let mut w = load15();
    w.scene.sim.as_mut().unwrap().bond.pos = [35.5, 41.5, 15.0];
    w.scene.sim.as_mut().unwrap().bond.yaw = 0.0;
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    let mut boom_tick = None;
    for _ in 0..400 {
        if w.scene.sim.as_ref().unwrap().drones.mines.iter().any(|m| m.exploded) { boom_tick = Some(true); break; }
        let input = nf_game::player::Input { forward: 1.0, ..Default::default() };
        w.tick(input, false);
    }
    assert!(boom_tick.is_some(), "the mine must explode for the blast object to exist");
    // The blast object: kind 0, the standard blast script, the script's 165-tick lifetime.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.blasts.len(), 1, "one persistent blast object [M 0x17cbd0's class-0x0F]");
        let b = &sim.drones.blasts[0];
        assert_eq!(b.debris, 0, "Mine_Update's inline store is kind 0 [M 0x17c930:128]");
        assert_eq!(b.script_hash, 0x0600_0052, "the mine's p5 script [D p5; M 0x17c930's Script_Load]");
        assert_eq!(b.lifetime, 165, "the blast script's +0xAB0 u16 [M 0x17cd18]");
        assert_eq!(sim.drones.blast_debris.len(), 0, "no debris at birth [M 0x17c350:25]");
        assert_eq!((b.radius - 10.0).abs() < 0.01, true, "the disc's radius [D p1]");
    }
    // Across the whole blast life (script end ≈ 165 ticks) plus margin: still no debris, and the
    // record dies when the script ends. The sister below fails if the arm ever drew — a passing
    // modulo tick with kind 0 would leave 3 particles for at least 90 ticks [M 0x1ac808:29-31].
    let mut alive_ticks = 0u32;
    for _ in 0..260 {
        let sim = w.scene.sim.as_ref().unwrap();
        if !sim.drones.blasts.is_empty() { alive_ticks += 1; }
        assert!(sim.drones.blast_debris.is_empty(),
            "a kind-0 blast must never spawn debris [M 0x17c930:128 + 0x17c350:25]");
        let input = nf_game::player::Input { forward: 0.0, ..Default::default() };
        w.tick(input, false);
    }
    assert!(w.scene.sim.as_ref().unwrap().drones.blasts.is_empty(), "the blast record is gone once the script ends [M 0x17c394-0x17c3a0]");
    assert!(alive_ticks >= 100, "the blast persisted {alive_ticks} ticks, far past the 41-tick debris window");
}

/// The ladder: the east ladder #382-52 (84.53, 31.73, −2.29 [D]) is DEAD in the game's own pick
/// admission — the grab cannot fire from ANY spot, and the port matches. The chain [M]:
/// - `Ladder_Create` 0x1966c0 links the ladder only through `build_LinkToRoom` 0x1c9680 →
///   `build_FindCel` 0x1c8cb0, whose candidate test is the room cel's entity-params bounds box
///   (`parsemap_block_entity_params` 0x1d1508 fills the celglist slot's box from the entity
///   params — slot+0x20 min / slot+0x30 max, the same records the port's MapObject.min/max
///   parse; 0x1c8cb0:103-119 keeps cels whose box holds the point, 0 cels → miss).
/// - The ladder's world-sphere centre (84.529, 31.733, −2.288 — MapObject.centre (0,0,0) through
///   the placement [M Control_BuildWorldSph 0x133070]) sits inside NO room cel's box: the east
///   room's box spans y 39.28..45.14 (the centre y 31.73 is 7.5 below its floor) and every shaft
///   cel's box ends at x 81.87 (the ladder x 84.53) [D the 07000015 entity params]. FindCel
///   misses → `build_LinkToRoom` 0x1c9680 links nothing and leaves obj+0x20 = 0
///   [M control_create_object 0x1344e0 memsets it; the miss path adds no ForcedList entry], and
///   `control_handle_cel_change` 0x133288:63-67's retry runs the same `build_FindCel` on the same
///   centre — the same miss forever (class 0x2D ≠ 2, so not even the Drone_FellOutMap arm).
/// - `Controls_StraddleTest` 0x134778 [M disasm 0x1347b8-0x1347d0: a2 = `obj+0x20`, delay-slot
///   `sw $a2, 0x80($sp)` — the BFS start] then runs from cel 0, so `Collide_StraddleCels`
///   0x1eac98 returns just the start cel (0x1eac98:19 the `start == 0` BFS skip, :82-84 appends
///   the start cel — count 1 < 2, disasm 0x1347d8 `sltiu`) and the count<2 arm (0x13482c-0x134860:
///   `sw $zero,($s0)` / `sw $zero,4($s0)` / `0xf0 &= ~0x10008000`) does NOT add the ladder to the
///   ForcedList 0x270590. The 0x20000000 forced-class arm of `build_LinkToRoom` 0x1c9680:61-68
///   never ran for a ladder either (the flag is clear).
/// - `Collide_Pick` 0x1eb320's candidates are the straddled cels' object chains (:125-134, run
///   ONLY when the straddle count is non-zero: :128 `if (auStack_f0[0] != 0)` — disasm 0x1eb410
///   `lhu $v0, 0xb0($sp)` / 0x1eb414 `beql $v0, $zero, 0x1eba38`) plus the ForcedList loop
///   (LAB_001eba38: head = *(0x270590), `Collide_PickObj` per node, walk +4). An EMPTY straddle
///   set goes STRAIGHT to the ForcedList — the empty set admits nothing by itself. The old
///   master reading ("the pick admits the unlinked ladder only from an empty straddle set",
///   citing 0x1eb320:125-134) quoted these lines backwards: a non-empty straddle set is the
///   REQUIREMENT for the cel-chain candidates, and the unlinked ladder is on NO cel's chain and
///   not on the ForcedList, so `Collide_PickObj` never runs for it, `Player_DealWithObjHit`
///   0x194458 never sees class 0x2D, and `Player_CollLadder` 0x196118 never fires. Ladders #900
///   (84.53, 31.73, 69.07) and #901 (−13.34, 31.73, 73.33) miss FindCel the same way [D the same
///   boxes] — no ladder on this level is grabbable.
///
/// The walk below is real input to the closest possible approach: across the east room's south
/// ledge (its top strip y ≈ 40.5 spans z −2.13..−1.8 [D floor tris]) onto the south walkway
/// (y ≈ 40.4, z −2.86..−2.13, x 82..90.7 [D]) and west INTO the ladder plane (x 84.529, the
/// 2-tri z-degenerate mesh [D]). Bond's pre-push-out spot penetrates the plane's pick reach
/// (the sister pins the mesh actually reaches), yet the climb never engages — the admission
/// chain, not the geometry, is the blocker. The old assertion (the walk-in grabs [M
/// Player_CollLadder]) contradicted this chain: the TEST was wrong, fixed here (rounds 2+3).
#[test]
fn the_dead_ladder_never_grabs_at_point_blank() {
    let mut w = load15();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let world = &sim.world;
        // The premise, per ladder [D entity params; M the box test 0x1c8cb0]: FindCel misses at
        // every ladder's world-sphere centre, so the create-time link (the port resolves the
        // same call at load, scene.rs `lad.cell = world.find_cel(lad.centre)`) leaves them all
        // unlinked — the game's obj+0x20 = 0 state.
        for (name, c) in [("#382", [84.529f32, 31.733, -2.288]), ("#900", [84.529, 31.733, 69.066]), ("#901", [-13.342, 31.733, 73.332])] {
            assert_eq!(world.find_cel(c), None, "no room cel's entity-params box holds ladder {name}'s centre [D boxes; M 0x1c8cb0]");
        }
        assert!(sim.ladders.iter().all(|l| l.cell.is_none()),
            "every ladder loads unlinked — the game's FindCel miss at create [M 0x1c9680]");
    }
    // Reach: the south walkway east of the ladder plane (the walkway is real floor — cel 123,
    // y ≈ 40.35 [D floor tris] — but the room's south wall slides the walker east off the direct
    // line, the same wall round-1's probes reported; the teleport only REACHES the spot, the
    // file's standing convention). The grab attempt itself is the real walk.
    w.scene.sim.as_mut().unwrap().bond.pos = [85.2, 40.5, -2.4];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.substate != nf_game::player::SS_CLIMB, "sister: not climbing before the approach");
    }
    // West into the ladder plane, facing WEST — the grab-favourable facing
    // (|AngleDiff(−π/2, ladder yaw π/2)| = π > π/2 [M 0x196118:20]), so the angle gate would
    // pass if the pick ever ran.
    w.scene.sim.as_mut().unwrap().bond.yaw = -std::f32::consts::FRAC_PI_2;
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.pos[1] > 40.2 && sim.bond.pos[1] < 40.7,
            "sister: Bond stands on the south walkway (y {:?}, the floor ≈ 40.35 [D floor tris])", sim.bond.pos[1]);
    }
    let mut climb_ticks = 0;
    let mut min_x = f32::MAX;
    for _ in 0..70 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.substate == nf_game::player::SS_CLIMB { climb_ticks += 1; }
        min_x = min_x.min(sim.bond.pre_collide[0]);
        w.tick(nf_game::player::Input { forward: 1.0, ..Default::default() }, false);
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(climb_ticks == 0, "the walk into the ladder plane never engages the climb — the game's pick admission excludes it (unlinked + not forced [M 0x1c9680/0x134778/0x1eb320])");
    assert!(min_x < 84.6, "the walk crossed the ladder plane's x 84.529 (min pre-push x {min_x}) — the plane is no movement wall [M the movement pass tests the cel world, Player_Collision 0x194df0] and Bond's capsule passed THROUGH the pick's reach, yet no climb: the admission chain, not the geometry, is the blocker");
}

/// Sister (can fail): the exclusion is the ADMISSION, not the geometry. At the point-blank stop
/// the ladder's own pick WOULD hit — the default walk arm's capsule against the ladder's 0x2E
/// triangles [M Collide_PickObj 0x1eae80 ×1.5 sphere + the mesh refine 0x109da0 strict < r, the
/// port's `capsule_hits_default`] — so a change that relinks the ladders (bigger boxes, a
/// ForcedList port) flips the main test too, and a change to the ladder's parsed mesh or the
/// pick math fails here.
#[test]
fn sister_the_dead_ladder_mesh_reaches_the_point_blank_stander() {
    let mut w = load15();
    w.scene.sim.as_mut().unwrap().bond.pos = [85.08, 40.4, -2.4];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.scene.sim.as_mut().unwrap().bond.yaw = -std::f32::consts::FRAC_PI_2;
    w.wait(10);
    let sim = w.scene.sim.as_ref().unwrap();
    let lad = &sim.ladders[0];
    assert!((lad.centre[0] - 84.529).abs() < 0.01 && (lad.centre[1] - 31.733).abs() < 0.01,
        "sister: ladder #382 is the east ladder at the probed spot [D placement]");
    assert!(lad.capsule_hits_default(sim.bond.pre_collide, nf_game::player::RADIUS, sim.bond.cc),
        "the walk arm's pick DOES reach the point-blank stander [D the 2-tri plane; M 0x1eae80/0x109da0] — the blocker is the admission chain, not the geometry");
}

/// KIND: destroyables 0xE1, the external-kill arm — `Destroy_Create` 0x12c318 stores placement
/// param 1 into tag+0x10 (disasm 0x12c3b8-0x12c3bc: `lhu 0x30($s0)` → `sh 0x10($a0)`) and
/// `Destroy_Update` 0x12c4f8's second arm smashes the tag while that channel is ON (the word then
/// clears, :32; the port's `Props::channel_kills`). The east room's BoxSatellite by its spot
/// (69.52, 43.41, 10.26) [D census] carries param 1 = 0xf6 — the SAME channel the room touch #717
/// writes — so the real walk into #717 (the proven east-room leg) smashes it, no shot fired.
/// Sisters: the tag sits alive with the parsed kill channel before the walk, and the OTHER two
/// satellites (params 1 = 0xed/0xec — channels nothing fires on this leg) stay alive.
#[test]
fn the_room_touch_smashes_its_satellite_through_the_kill_channel() {
    let mut w = load15();
    let sat = |sim: &nf_view::scene::Sim| sim.drones.props.destroy.iter()
        .find(|d| walker::dxz(d.pos, [69.52, 43.41, 10.26]) < 1.0).cloned().expect("the east-room BoxSatellite [D census]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let s = sat(&sim);
        assert!(!s.dead, "sister: the satellite stands before the walk");
        assert_eq!(s.kill_channel, 0xf6, "its tag+0x10 kill channel is the room-alert ch 0xf6 [D param 1; M 0x12c318:36]");
        assert!(sim.drones.props.destroy.iter()
            .filter(|d| walker::dxz(d.pos, [33.91, 39.98, 72.95]) < 1.0 || walker::dxz(d.pos, [-8.04, 48.34, 1.81]) < 1.0)
            .all(|d| !d.dead), "sister: the 0xed/0xec satellites stand too [D census]");
    }
    // The real walk east into touch #717 (the same two legs the_silo_fall and the_east_room use).
    w.walk_to(Wp::new(88.0, 41.4, -0.5).r(1.2), "the gantry west");
    w.walk_to(Wp::new(80.5, 41.4, 0.08).r(1.2), "into touch #717");
    // The channel arm runs in the drones' tick; wait bounded ticks for the smash.
    let mut smashed = false;
    for _ in 0..120 {
        if sat(&w.scene.sim.as_ref().unwrap()).dead { smashed = true; break; }
        w.tick(nf_game::player::Input::default(), false);
    }
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(smashed, "the room touch's ch 0xf6 smashes the satellite through tag+0x10 [M Destroy_Update 0x12c4f8:30-35]");
    assert!(sim.drones.props.destroy.iter()
        .filter(|d| walker::dxz(d.pos, [33.91, 39.98, 72.95]) < 1.0 || walker::dxz(d.pos, [-8.04, 48.34, 1.81]) < 1.0)
        .all(|d| !d.dead), "sister: the 0xed/0xec satellites survive — their channels never fired [D params]");
}

/// KIND: destroyables 0xE1, the bullet arm + the death channel — `Destroy_CollisionHandler`
/// 0x12bd48 subtracts every admitted class-5 contact's damage from tag+0x04 (placement param 3)
/// and the death writes `switch_channels[tag+0x12] = 1` (placement param 0 [M 0x12c318:35, disasm
/// 0x12c3c0-0x12c3c4]) and smashes. The great-hall barrel by its spot (0.65, 40.30, 28.99) [D
/// census] carries health 10 (param 3), death channel 0xfd (param 0 — no other writer or reader
/// of 0xfd on the level [D census]) and the bullet mask param 4 = 0xf8 → tag+0x14 = ~0xf8 = 0xff07
/// [M 0x12c40c-0x12c420]: the handhelds' weapon word 0x0008 ANDs the mask to 0 so the shot is
/// ADMITTED ([L ee_start_05 weapon_data+0x10]; the same filter the 07000014 windows pin runs,
/// opposite data), while the laser 0x0002 / the watch 0x0100 would drop. Reach: the hall-south
/// lane the walked route proves ((2.0, 41.4, 30.0) — the pickups/walkreal leg); the teleport only
/// REACHES it, the shot itself is the real trigger path. Sister: the barrel stands at health 10
/// with ch 0xfd off before the shot.
#[test]
fn the_oil_barrel_dies_to_the_gunshot_and_writes_its_death_channel() {
    let mut w = load15();
    w.top_up_health = true; // the death blast (damage 5.0 = param 5 500 ×0.01 [M 0x12c318:48]) may splash
    let barrel = |sim: &nf_view::scene::Sim| sim.drones.props.destroy.iter()
        .find(|d| d.channel == 0xfd).cloned().expect("the ch-0xfd barrel [D census — the level's only p0 = 0xfd tag]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let b = barrel(&sim);
        assert!(!b.dead && (b.health - 10.0).abs() < 0.5, "sister: the barrel stands at health 10 [D param 3]");
        assert!(walker::dxz(b.pos, [0.65, 40.30, 28.99]) < 1.0, "sister: it is the great-hall tag [D census translation]");
        assert!(!sim.drones.channels.on(0xfd), "sister: ch 0xfd is off before the shot (the initial state)");
    }
    // Reach: the proven hall-south lane spot (the walkreal/pickups route's hall south leg).
    w.scene.sim.as_mut().unwrap().bond.pos = [2.0, 41.5, 30.0];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    let target = {
        let b = barrel(&w.scene.sim.as_ref().unwrap());
        [(b.lo[0] + b.hi[0]) / 2.0, (b.lo[1] + b.hi[1]) / 2.0, (b.lo[2] + b.hi[2]) / 2.0]
    };
    let killed = w.shoot_point(target, "15: the great-hall barrel", &|sim: &nf_view::scene::Sim| {
        sim.drones.props.destroy.iter().find(|d| d.channel == 0xfd)
            .map(|d| d.dead).unwrap_or(false) || sim.drones.channels.on(0xfd)
    });
    assert!(killed, "the shot kills the 10-health barrel [M 0x12bd48:44-48 every admitted contact]");
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(barrel(&sim).dead, "the leaf is dead");
    assert!(sim.drones.channels.on(0xfd), "the death writes switch_channels[0xfd] [M 0x12bd48:35; D param 0]");
}

/// Every placement's model resolves on this level — zero missing hashes/indices of the 968
/// placements across all 55 maps [D parse].
#[test]
fn every_placement_resolves_its_model() {
    let img = nf_formats::iso9660::IsoImage::open(&iso()).expect("open iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("FILES.BIN");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000015.bin")).expect("07000015.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    let lv = nf_formats::level::Level::parse(&data).expect("parse");
    let mut unresolved = 0;
    for m in &lv.maps {
        for p in &m.placements {
            let ok = if p.by_hash() { lv.by_hash.contains_key(&p.hashcode) }
                else { (p.index as usize) < m.objects.len() };
            if !ok { unresolved += 1; }
        }
    }
    assert_eq!(unresolved, 0, "every placement's model resolves [D the 07000015 parse]");
}
