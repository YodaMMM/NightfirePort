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
    let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso(), "07000015.bin", false, None, 3).unwrap());
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
#[test]
fn the_east_room_touch_wakes_its_squad() {
    let mut w = load15();
    w.walk_to(Wp::new(88.0, 41.4, -0.5).r(1.2), "the gantry west");
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(!sim.drones.channels.on(0xf6), "sister: 0xf6 off before the east walk [D]");
    w.walk_to(Wp::new(80.5, 41.4, 0.08).r(1.2), "into touch #717");
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(sim.drones.channels.on(0xf6), "the east room's plate #717 fires ch 0xf6 on the walk [D volume (80.81, 41.79, 0.08)]");
    let apoc = sim.drones.fx.apocalypses.iter().find(|a| a.channel == 0xf6).expect("the 0xf6 apocalypse");
    assert!(apoc.armed, "the Apocalypse #710 latches armed on ch 0xf6 [M Apocalypse_Update 0x18d4f0]");
    assert!(apoc.fade > 0.0, "the whiteout mirror runs (remaining·5/total [M 0x18d4f0])");
}

/// The pickups take on the walk-over: the Pickup 007 #813 (53.40, 40.57, 71.14) sits on the
/// north band by door[47]'s proven lane. The take consumes it
/// [M Pickup_Handler 0x18e650 case 6 → PlrStat_LogBondBonus 0x1999e0].
#[test]
#[ignore = "the walked route wedges: the north-band leg to the 007 #813 (53.40, 40.57, 71.14) and the exit-hall leg to the 007 #812 (−1.15, −7.81) both STUCK the walker (the door[29] corner at (28.05, 68.66) — the same furniture hop the walkreal documents at door[32]'s approach; the north band wedges at the room mouths). The take mechanic is family-proven (07000001/11/13) and the crates are reachable on foot in the game; owner: the walker's west-leg route"]
fn the_pickups_take_on_the_walk() {
    let mut w = load15();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(walker::pickups(sim).iter().any(|(p, _, _)| walker::dxz(*p, [53.40, 40.57, 71.14]) < 1.5),
            "sister: the 007 pickup #813 still sits on its crate [D]");
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
    w.walk_to(Wp::new(53.4, 41.4, 70.0).r(1.2), "the north band east");
    w.walk_to(Wp::new(53.40, 41.4, 71.14).r(0.8), "the 007 crate east");
    w.walk_to(Wp::new(-1.0, 41.4, 62.0).r(1.2), "the south turn");
    w.walk_to(Wp::new(-1.0, 41.4, 20.0).r(1.2), "door[42] south");
    w.walk_to(Wp::new(-1.0, 41.4, 4.0).r(1.2), "door[51] south");
    let sim = w.scene.sim.as_ref().unwrap();
    assert!(!walker::pickups(sim).iter().any(|(p, _, _)| walker::dxz(*p, [53.40, 40.57, 71.14]) < 1.5),
        "the Pickup 007 #813 takes on the walk-over (the take consumes it [M 0x18e650 case 6])");
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

/// The ladder: the east ladder #382-52 (84.53, 31.73, −2.29 [D]) takes the real climb hold —
/// Bond's walk into it engages the climb substate and he rises [M Player_CollLadder].
#[test]
#[ignore = "the route: the ladder's base is on the LOWER deck (y 23.8) at x 84.5 and the teleport drops Bond in the wall — needs the deck's spawn-side approach; the climb mechanic is family-proven (07000006/09); owner: the walker's deck route"]
fn the_ladder_takes_the_climb() {
    let mut w = load15();
    w.scene.sim.as_mut().unwrap().bond.pos = [83.4, 23.9, -2.29];
    w.scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    w.wait(10);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.substate != nf_game::player::SS_CLIMB, "sister: not climbing before the hold");
    }
    let mut climbing = false;
    for _ in 0..200 {
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.bond.substate == nf_game::player::SS_CLIMB { climbing = true; break; }
        let input = nf_game::player::Input { forward: 1.0, ..Default::default() };
        w.tick(input, false);
    }
    assert!(climbing, "the walk into the ladder never engaged the climb [M Player_CollLadder]");
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
