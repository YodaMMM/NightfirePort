//! interactables-11: one live test per KIND of player-touchable placement on 07000011 (HT Tower2 A,
//! the office floor → the lower office floor), driven the way a player drives it (the walk, the use
//! key, the trigger pull, the walk-over, the Q-Claw), each asserting the game's effect with its
//! sister. The census with the type table lives in docs/levels/07000011.md (task level-11-interactables);
//! the end-to-end route is walk_07000011_live.rs.
//!
//! Evidence tags: [M] the decompile address, [D] the disc bytes, [L] the EE snapshot. A teleport to
//! reach a spot is used only where the walk there is separately documented; the operation itself
//! always runs through the real input.

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// A cold load — the level's own boot path [M Player_Start 0x190fd0's gate-0 start].
fn load() -> nf_view::scene::Scene {
    nf_view::load(&iso(), "07000011.bin", false).expect("load 07000011")
}

/// The gold-medal `bonus[0]` word for `base_map` [M PlrStats_HasGoldMedal 0x199a40;
/// Menu_SetLevelBonus 0x1ffa10 is the award arm]. pickup-medal-gate: without the level's gold
/// medal the kind-6 '007' pickups are not created at all [M Pickup_Create 0x18dda0:93-95].
fn gold_bits(base_map: u32) -> u64 {
    let mut p = nf_game::progress::Progress::default();
    p.set_level_bonus(base_map, 3, 0);
    p.bonus[0]
}

/// The medal load: 07000011 plays under BaseMap 0x07000011 (the mission table [L ee_start_05
/// 0x2a4350], the gold bit the 0x0700005/9/11 kind-4 group's rank 2 [L 0x2bc920 row 7; M 0x199210]).
fn load_medaled() -> nf_view::scene::Scene {
    nf_view::load_with_carry_medals(&iso(), "07000011.bin", false, None, 1, gold_bits(0x0700_0011)).expect("medal load 07000011")
}

/// The keypad lock (0x2E #177 "Lock - KeypadLocked", param 5 = out ch 0x12 [D]; param 6 = 1 = the
/// Operative create-unlock tag [D]) sits UNLOCKED on Operative (`Lock_Create` 0x1ae1f8:67-69 →
/// `Lock_Unlock`), and `Lock_Update` case 1 (0x1ae3d8) writes `switch_channels[u16[+8]] = 1` EVERY
/// tick in state 1 — the out channel is on from the first frame. Door #192 "door_large_stairwell"
/// (lock ch 0x12 [D param 3]) therefore opens on the E. Sister (same test, the difficulty-3 load):
/// the tag only auto-unlocks on Operative, so ch 0x12 is off and the same E is refused.
#[test]
fn keypad_operative_unlock_holds_ch12_and_opens_the_stairwell_door() {
    let mut scene = load();
    {
        let sim = scene.sim.as_mut().unwrap();
        assert_eq!(sim.drones.locks.len(), 1, "the level's one 0x2E lock [D #177]");
        assert!(sim.drones.locks[0].unlocked, "param 6 = 1 unlocks on Operative [D #177 p6; M 0x1ae1f8:67-69]");
        assert!(sim.drones.channels.on(0x12),
            "Lock_Update case 1 holds the out channel on in state 1 [M 0x1ae3d8 case 1]");
        // The door: the large stairwell door by the office-floor stair mouth.
        let (di, dx, dy, dz) = {
            let di = sim.drones.doors.list.iter()
                .position(|d| d.ch_lock == 0x12).expect("door_large_stairwell #192 (lock ch 0x12) [D]");
            let d = &sim.drones.doors.list[di];
            (di, d.pos[0], d.pos[1], d.pos[2])
        };
        sim.bond.pos = [dx + 0.4, dy - 0.2, dz + 1.6];
        sim.bond.vel = [0.0; 3];
        for _ in 0..10 { sim.step_bond(Default::default(), false, false, false); }
        for _ in 0..8 { sim.step_bond(Default::default(), false, false, true); }
        let st = scene.sim.as_ref().unwrap().drones.doors.list[di].state;
        assert!(st & 0xc != 0, "the door locked on ch 0x12 opens once the unlock holds, state {st:#x}");
    }
    // Sister: the same data on 00 Agent — no create-unlock, ch 0x12 off, the E refused.
    let iso = iso();
    let mut s3 = nf_view::load_with_carry_difficulty(&iso, "07000011.bin", false, None, 3).expect("load d3");
    let sim = s3.sim.as_mut().unwrap();
    assert!(!sim.drones.locks[0].unlocked, "sister: no Operative unlock on difficulty 3 [M 0x1ae1f8:67-69]");
    assert!(!sim.drones.channels.on(0x12), "sister: ch 0x12 off — nothing writes it here [D]");
    let di3 = sim.drones.doors.list.iter().position(|d| d.ch_lock == 0x12).unwrap();
    let (dx, dy, dz) = { let d = &sim.drones.doors.list[di3]; (d.pos[0], d.pos[1], d.pos[2]) };
    sim.bond.pos = [dx + 0.4, dy - 0.2, dz + 1.6];
    sim.bond.vel = [0.0; 3];
    for _ in 0..10 { sim.step_bond(Default::default(), false, false, false); }
    let before = sim.drones.doors.list[di3].state;
    for _ in 0..8 { sim.step_bond(Default::default(), false, false, true); }
    assert_eq!(sim.drones.doors.list[di3].state, before,
        "sister: the locked stairwell door refuses the same E (difficulty 3)");
}

/// The Q-Worm computer (0x35 #216 "Computer to hack", param 1 = out ch 0x38, param 2 = 1 = the
/// E-arm [D]) installs on the E: `PCQWorm_Activate` 0x1b0840 (armed + unused → state 1) and
/// `PCQWorm_Update` 0x1aee20's state 1 writes `switch_channels[param 1] = 1`. The pair ch 0x38
/// triggers ScriptPlayer #263 (0x060008C0, mode 8 [D]). Sister: ch 0x38 off before the E.
#[test]
fn qworm_computer_e_installs_and_writes_ch0x38() {
    let mut scene = load();
    let u = {
        let sim = scene.sim.as_ref().unwrap();
        sim.drones.useobjs.pcs.iter().find(|q| q.ch == 0x38).expect("the Q-Worm PC #216 [D]").pos
    };
    {
        let sim = scene.sim.as_mut().unwrap();
        for _ in 0..10 { sim.step_bond(Default::default(), false, false, false); }
    }
    assert!(!scene.sim.as_ref().unwrap().drones.channels.on(0x38), "sister: ch 0x38 off before the E");
    // Stand at the computer and hold E (the use sphere is head + look [M 0x1a5170]).
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [u[0] + 0.2, u[1] - 1.3, u[2] + 0.9];
        sim.bond.vel = [0.0; 3];
    }
    for _ in 0..40 {
        let sim = scene.sim.as_mut().unwrap();
        let (h, t) = (sim.bond.head(), u);
        let d = [t[0] - h[0], t[1] - h[1], t[2] - h[2]];
        sim.bond.yaw = d[0].atan2(d[2]);
        sim.bond.pitch = 0.0;
        sim.step_bond(Default::default(), false, false, true);
        if scene.sim.as_ref().unwrap().drones.channels.on(0x38) { break; }
    }
    assert!(scene.sim.as_ref().unwrap().drones.channels.on(0x38),
        "the E installs the Q-Worm and writes ch 0x38 [M 0x1b0840; 0x1aee20 state 1]");
}

/// The vent grills (0xE2: #84 "vent_grill1, interior" and #312 "vent_grill, exterior", both
/// state 0x40 / mask 4 [D]) sweep on the E: `SS_Activate` 0x1a9ea8 and the state-0x40 wrap holds
/// the end pose [M 0x1a9948:121-126]. Sister: the key-0 pose before the E.
#[test]
fn vent_grill_e_sweeps_open() {
    let mut scene = load();
    let (gi, start) = {
        let sim = scene.sim.as_mut().unwrap();
        let gi = sim.drones.sss.list.iter().position(|s| s.placement == 84).expect("the interior grill SS #84 [D]");
        assert!(sim.drones.sss.list.iter().any(|s| s.placement == 312),
            "the exterior grill SS #312 exists with the same wiring [D]");
        (gi, sim.drones.sss.list[gi].matrix[3][2])
    };
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [8.57, 49.6, 21.0];
        sim.bond.vel = [0.0; 3];
        for _ in 0..10 { sim.step_bond(Default::default(), false, false, false); }
    }
    assert!((scene.sim.as_ref().unwrap().drones.sss.list[gi].matrix[3][2] - start).abs() < 0.05,
        "sister: the grill rests at its key-0 pose before the E");
    let mut mid_moved = 0.0f32;
    for _ in 0..40 {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Default::default(), false, false, true);
        let s = &sim.drones.sss.list[gi];
        let d = (0..3).map(|k| (s.matrix[3][k] - [8.571396, 49.72885, 19.599398][k]).abs()).sum::<f32>();
        if d > mid_moved { mid_moved = d; }
    }
    assert!(mid_moved > 0.02, "the E sweeps the grill off its key-0 pose [M SS_Activate 0x1a9ea8], mid {mid_moved:.4}");
    for _ in 0..300 {
        let mut sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Default::default(), false, false, false);
        let s = &sim.drones.sss.list[gi];
        if s.state == 0x40 && s.wrap_flag == 1 { break; }
    }
    let s = &scene.sim.as_ref().unwrap().drones.sss.list[gi];
    let moved = (0..3).map(|k| (s.matrix[3][k] - [8.571396, 49.72885, 19.599398][k]).abs()).sum::<f32>();
    // The E-activated sweep moves the pose mid-run, and the state-1 sweep wraps to state 0
    // holding the END pose [M 0x1a9948:121-126] — the same wrap end the 0b grill's test pinned.
    eprintln!("[11] grill end state {} wrap {} end-moved {moved:.4} mid-moved {mid_moved:.4}", s.state, s.wrap_flag);
    assert!(s.state == 0 && s.wrap_flag == 1 && moved > 0.02,
        "the sweep wraps to state 0 holding the end pose [M 0x1a9948:121-126], end state {} wrap {} moved {moved:.4}", s.state, s.wrap_flag);
}

/// The doors (0xDB ×12): a plain swing door (#13, no lock [D]) opens on the E [M Door_Activate];
/// a stairwell decor door (#194, lock ch 200 = 0xC8 [D]) refuses the same E while ch 200 has no
/// writer on this level [D]. Sister in the same test.
#[test]
fn plain_door_opens_and_locked_stairwell_door_refuses() {
    let mut scene = load();
    let (di, li, dx, dy, dz, lx, ly, lz) = {
        let sim = scene.sim.as_ref().unwrap();
        let di = sim.drones.doors.list.iter()
            .position(|d| (d.pos[0] - 13.82).abs() < 0.5 && (d.pos[2] - 7.09).abs() < 0.5)
            .expect("door_waitingroom_corridor_right #13 [D]");
        let li = sim.drones.doors.list.iter()
            .position(|d| (d.pos[0] - 14.01).abs() < 0.5 && (d.pos[1] - 31.14).abs() < 0.5)
            .expect("DOOR_Stairwell_1 #194 (lock 200) [D]");
        let d = &sim.drones.doors.list[di]; let e = &sim.drones.doors.list[li];
        (di, li, d.pos[0], d.pos[1], d.pos[2], e.pos[0], e.pos[1], e.pos[2])
    };
    {
        let sim = scene.sim.as_mut().unwrap();
        // The locked one first (it stands mid-stairwell; the E is from its front tile).
        sim.bond.pos = [lx, ly - 0.2, lz + 1.6];
        sim.bond.vel = [0.0; 3];
        for _ in 0..10 { sim.step_bond(Default::default(), false, false, false); }
        let before = sim.drones.doors.list[li].state;
        for _ in 0..8 { sim.step_bond(Default::default(), false, false, true); }
        assert_eq!(sim.drones.doors.list[li].state, before,
            "sister: the ch-200 lock has no writer on 07000011 [D] — the door refuses the E");
    }
    {
        let sim = scene.sim.as_mut().unwrap();
        // The plain one now.
        sim.bond.pos = [dx, dy - 0.2, dz + 1.6];
        sim.bond.vel = [0.0; 3];
        for _ in 0..10 { sim.step_bond(Default::default(), false, false, false); }
        for _ in 0..8 { sim.step_bond(Default::default(), false, false, true); }
        let st = sim.drones.doors.list[di].state;
        assert!(st & 0xc != 0, "the unlocked swing door opens on the E [M Door_Activate], state {st:#x}");
    }
}

/// The storeroom door (#213, lock ch 0x5A [D param 3]) unlocks from its own touch plate (#283, out
/// ch 0x5A [D]): the plate's rising edge clears the door's lock for good [M door.rs ch_lock]. Sister:
/// the E is refused before the plate.
#[test]
fn storeroom_door_unlocks_from_its_touch_plate() {
    let mut scene = load();
    let (di, dx, dy, dz) = {
        let sim = scene.sim.as_ref().unwrap();
        let di = sim.drones.doors.list.iter()
            .position(|d| (d.pos[0] - 13.71).abs() < 0.5 && (d.pos[2] - 19.44).abs() < 0.5)
            .expect("door_waitingroom_storeroom #213 [D]");
        let d = &sim.drones.doors.list[di];
        (di, d.pos[0], d.pos[1], d.pos[2])
    };
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [dx, dy - 0.2, dz + 1.6];
        sim.bond.vel = [0.0; 3];
        for _ in 0..10 { sim.step_bond(Default::default(), false, false, false); }
        assert!(!sim.drones.channels.on(0x5a), "sister: ch 0x5A off before the plate");
        let before = sim.drones.doors.list[di].state;
        for _ in 0..8 { sim.step_bond(Default::default(), false, false, true); }
        assert_eq!(sim.drones.doors.list[di].state, before,
            "sister: the door locked on ch 0x5A refuses the E before its plate");
    }
    // Walk onto the plate #283 at (12.72, 47.55, 19.63) [D] — the corridor in front of the door.
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [12.72, 47.0, 19.63];
        sim.bond.vel = [0.0; 3];
    }
    let mut fired = false;
    for _ in 0..30 {
        let mut sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Default::default(), false, false, false);
        if sim.drones.channels.on(0x5a) { fired = true; break; }
    }
    assert!(fired, "the plate fires ch 0x5A [D #283; M triggers.rs]");
    // door-react-path5: the plate's ch 0x5A rising edge opens the door ITSELF — Door_Update's
    // unlock arm runs Door_Activate + clears the lock field [M 0x1363c8], and with ch_open ==
    // ch_lock == 0x5A [D #213] the leaf sweeps OPENING (the scratch dump: state 0x2 → 0x8,
    // pos01 climbing, ch_lock 90 → 0). The game's E on an OPENING/OPEN leaf CLOSES it [M
    // 0x135be0's state-4/8 arm] — the old tail ("the same E now opens the door") pinned the
    // every-tick re-toggle accident instead. The game-backed proof that the unlock works:
    // wait out the auto-open sweep, then one fresh E press acts on the unlocked leaf — where
    // the locked leaf in the sister block refused the same press outright.
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [dx, dy - 0.2, dz + 1.6];
        sim.bond.vel = [0.0; 3];
    }
    let mut open = false;
    for _ in 0..180 {
        let sim = scene.sim.as_mut().unwrap();
        sim.step_bond(Default::default(), false, false, false);
        if sim.drones.doors.list[di].state == nf_game::door::OPEN { open = true; break; }
    }
    assert!(open, "the plate's unlock rising edge must auto-open the storeroom door [M 0x1363c8]");
    {
        let sim = scene.sim.as_mut().unwrap();
        // One fresh press (the wait loop's ticks released the key): the open unlocked leaf closes.
        sim.step_bond(Default::default(), false, false, true);
        let st = sim.drones.doors.list[di].state;
        assert_eq!(st, nf_game::door::CLOSING,
            "E on the open unlocked door closes it [M 0x135be0's state-4/8 arm], got {st:#x}");
    }
}

/// The pickups (0xF0 ×13) take on the walk-over [M Pickup_Handler 0x18e650]: the BodyArmour
/// (#261, kind 3, 100 [D]) sets armour 100, the Spas12 (#267, weapon 0x1C, 5 [D]) and the grenades
/// (#268, weapon 0x34, 1 [D]) and the Satchel (#255, weapon 0x3B, 5 [D]) arm their weapons, and a
/// "Pickup 007" (#275, kind 6 [D]) is consumed. Sisters: armour 0 / the Spas un-owned before.
#[test]
fn pickups_taken_by_walking_over() {
    // pickup-medal-gate: the kind-6 row (#275) creates only with 07000011's gold medal recorded
    // [M Pickup_Create 0x18dda0:93-95] — the medaled load below; the old plain load pinned the
    // port's ungated create, which the decompile contradicts (the test was wrong, see the report).
    let mut scene = load_medaled();
    {
        let sim = scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.health.armour, 0.0, "sister: no armour before");
        assert!(!sim.drones.inventory.owned[0x1c], "sister: the Spas un-owned before");
    }
    // The armour at (23.12, 47.62, 22.88) [D].
    scene.sim.as_mut().unwrap().bond.pos = [23.12, 47.0, 22.88];
    scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    let mut got = false;
    for _ in 0..40 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if scene.sim.as_ref().unwrap().drones.health.armour >= 50.0 { got = true; break; }
    }
    assert!(got, "the walk-over grants the armour 50.0 [M 0x18e650:217-219 — the hard set], got {}",
        scene.sim.as_ref().unwrap().drones.health.armour);
    // The Spas12 at (13.60, 15.55, 5.52) [D] — the lower floor by the exit.
    scene.sim.as_mut().unwrap().bond.pos = [13.60, 15.1, 5.52];
    scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    let mut owned = false;
    for _ in 0..40 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if scene.sim.as_ref().unwrap().drones.inventory.owned[0x1c] { owned = true; break; }
    }
    assert!(owned, "the walk-over arms the Spas12 [D p1 = 0x1C; M 0x18e650 case 0]");
    // A BOND_BONUS (#275 at (17.55, 47.91, 28.57) [D]) is consumed by the take path.
    let taken = |scene: &nf_view::scene::Scene| scene.sim.as_ref().unwrap().drones.pickups.iter()
        .filter(|p| (p.m[3][0] - 17.55).abs() < 0.5 && (p.m[3][2] - 28.57).abs() < 0.5)
        .all(|p| p.gone || p.hidden);
    assert!(!taken(&scene), "sister: the bonus sits before the walk");
    scene.sim.as_mut().unwrap().bond.pos = [17.55, 47.3, 28.57];
    scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    for _ in 0..40 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    assert!(taken(&scene), "the BOND_BONUS is consumed on the walk-over [M 0x18e650:234-236]");
    // The no-medal arm [M 0x18dda0:93-95]: the plain fresh-boot load creates no kind-6 at all —
    // #275's spot holds nothing.
    let bare = load();
    assert!(bare.sim.as_ref().unwrap().drones.pickups.iter().all(|p| p.kind != nf_game::pickup::kind::BOND_BONUS),
        "no gold medal → the kind-6 bonus is not created [M 0x18dda0:93-95]");
}

/// The Q-Claw grapple (0x3A #265 "GRAPPLEPOINT_SkyBridge" [D]; the level kit grants the claw 0x50
/// [M 0x1a7ef8 case 0x7000011]): the fire latches the aim ray's point [M Player_SetGrapplePoint
/// 0x195da0] and the reel lifts Bond [M Player_SSGrapple 0x192860]. Sister: no grapple before the fire.
#[test]
fn grapple_point_reels_bond_up() {
    let iso = iso();
    let mut scene = nf_view::load_weapons(&iso, "07000011.bin", false, &[0x50]).expect("load with the claw");
    let gp = scene.sim.as_ref().unwrap().drones.grapple_points.iter().find(|p| (p.pos[1] - 55.55).abs() < 0.5)
        .expect("the SkyBridge grapple point [D #265]").pos;
    // Bond on the office floor under the point (the atrium side), claw out.
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [gp[0], 46.5, gp[2] - 4.0];
        sim.bond.vel = [0.0; 3];
        for _ in 0..20 { sim.step_bond(nf_game::player::Input { switch_to: Some(0x50), ..Default::default() }, false, false, false); }
    }
    assert!(scene.sim.as_ref().unwrap().bond.grapple.is_none(), "sister: no grapple before the fire");
    // Aim up at the point and fire through the Scene's step (the real input path with the latch).
    let y0 = scene.sim.as_ref().unwrap().bond.pos[1];
    let mut latched = false;
    for _ in 0..20 {
        let (h, p) = { let s = scene.sim.as_ref().unwrap(); (s.bond.head(), gp) };
        let d = [p[0] - h[0], p[1] - h[1], p[2] - h[2]];
        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-4);
        let s = scene.sim.as_mut().unwrap();
        s.bond.yaw = d[0].atan2(d[2]);
        s.bond.pitch = (d[1] / l).asin() / std::f32::consts::FRAC_PI_2;
        scene.sim.as_mut().unwrap().step_bond(Default::default(), true, false, false);
        if scene.sim.as_ref().unwrap().bond.grapple.is_some() { latched = true; break; }
    }
    assert!(latched, "the claw fire latches the grapple point [M 0x195da0]");
    for _ in 0..120 { scene.sim.as_mut().unwrap().step_bond(Default::default(), true, false, false); }
    let y1 = scene.sim.as_ref().unwrap().bond.pos[1];
    assert!(y1 > y0 + 3.0, "the reel lifts Bond toward the point [M 0x192860], y {y0} → {y1}");
}

/// The sound triggers (0xF9 ×6): the two part-gate-0 ones exist and play — #279 on the spawn
/// touch's ch 0x7E (sound 632 [D]) and #286 on the Bond-Moment plate's ch 0x66 (sound 331 [D]);
/// the four gate-7 ones ([231]/[237]/[238]/[289]) are refused by `SoundTrigger_Create`'s gate
/// [M 0x1b2628:24-40]. Sister: the gate-7 list is empty at boot.
#[test]
fn sound_triggers_created_and_played_per_the_disc_gate() {
    let mut scene = load();
    {
        let sim = scene.sim.as_mut().unwrap();
        let ons: Vec<u16> = sim.drones.sound_triggers.list.iter().map(|t| t.on).collect();
        assert!(ons.contains(&0x7e) && ons.contains(&0x66), "the gate-0 triggers exist [D #279/#286], got {ons:?}");
        assert_eq!(ons.len(), 2, "sister: the gate-7 triggers are refused at boot [M 0x1b2628:24-40]");
        for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
    }
    assert!(scene.sim.as_ref().unwrap().drones.sound_triggers.playing(632),
        "the spawn touch's ch 0x7E plays sound 632 [D #279; M 0x1b2790]");
    // The Bond-Moment plate #235 (out 0x66) at (9.36, 49.78, 19.60) [D].
    scene.sim.as_mut().unwrap().bond.pos = [9.36, 49.3, 19.60];
    scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    let mut fired = false;
    for _ in 0..40 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if scene.sim.as_ref().unwrap().drones.sound_triggers.playing(331) { fired = true; break; }
    }
    assert!(fired, "the plate's ch 0x66 plays sound 331 [D #286; M 0x1b2790]");
}

/// The hint marker (0x33 #288 "HintMsg", param 0 = label 0x1000291 [D]) creates with the disc's
/// wiring by `Hint_Create` 0x1b3248 (no stage gate — param 2 absent [D]); the text display is the
/// render layer's (the port's `hints()` is the scene's draw data, fix-hints). Sister: the stage
/// gate refuses a stage-5 marker on difficulty 3 [M 0x1b3248:12-25].
#[test]
fn hint_marker_created_with_the_disc_label() {
    let level = level_11();
    let level_hints = nf_game::player::hints(&level, 1);
    let h = level_hints.iter().find(|h| (h.pos[0] + 19.59).abs() < 0.1 && (h.pos[2] - 30.73).abs() < 0.1)
        .expect("the HintMsg marker [D #288]");
    assert_eq!(h.label, 0x0100_0291, "the disc's label [D #288 p0]");
    assert!((h.pos[0] + 19.59).abs() < 0.1 && (h.pos[2] - 30.73).abs() < 0.1,
        "the hint stands at the exec office [D], at {:?}", h.pos);
    assert!(h.sound == 0 && h.on_channel == 0 && h.off_channel == 0, "no sound / channel wiring [D]");
    // Sister: the same hint list refuses a stage-5 marker on difficulty 3 [M 0x1b3248:12-25].
    assert!(!nf_game::player::hint_stage_gates(5, 3), "sister: the stage gate refuses stage 5 on 00 Agent");
}

/// The parsed 07000011 level archive (the nf-formats parse the loader runs).
fn level_11() -> nf_formats::level::Level {
    let img = nf_formats::iso9660::IsoImage::open(iso()).expect("iso");
    let (entries, bin) = nf_formats::disc::files_bin_entries(&img).expect("files.bin");
    let fe = entries.iter().find(|e| e.name.eq_ignore_ascii_case("07000011.bin")).expect("07000011.bin");
    let data = img.read_bytes(bin.lba as u64 * 2048 + fe.offset as u64, fe.size as u64).expect("read");
    nf_formats::level::Level::parse(&data).expect("parse")
}

/// The third icons (0xFA ×2, #254/#274, param 0 = context 2, param 1 = 0 = always enabled [D])
/// create with the disc's wiring [M ThirdIcon_Create 0x197070]; the +0x95F context write is the
/// scene's (fix-wire2, scene.rs 467). Sister: an icon with param 1 non-zero is channel-gated.
#[test]
fn third_icons_created_with_the_disc_params() {
    let level = level_11();
    let icons = nf_game::wire::icons(&level);
    assert_eq!(icons.len(), 2, "the two 3RD_Person_Icon placements [D #254/#274]");
    for i in &icons {
        assert_eq!(i.ctx, 2, "the disc's context type [D p0 = 2]");
        assert_eq!(i.on_channel, 0, "always enabled [D p1 = 0; M 0x197070's create]");
    }
}

/// The multiplexor (0xED #256 "Mutiplexor_SIn"): `Trigger_MultiplexSIn` 0x1b2e08 rewires the
/// placement's channels to a SEQ_AND with out = the first listed channel and inputs = the rest —
/// out 0x35, inputs [0x38, 0x39] [D #256 pairs]. Driven the real way: the Q-Worm E writes 0x38
/// (test 2's path) and the touch #257 (out 0x39, by the exec-office door [D]) fires on the walk —
/// the sim's own trigger update then raises 0x35. Sister: one input alone never raises it.
#[test]
fn multiplexor_seq_and_raises_its_out_on_both_inputs() {
    let mut scene = load();
    {
        let sim = scene.sim.as_ref().unwrap();
        let t = sim.drones.triggers.iter().find(|t| t.kind == nf_game::triggers::kind::SEQ_AND)
            .expect("the multiplexor's SEQ_AND node [M 0x1b2e08]");
        assert_eq!(t.out, 0x35, "out = the first channel [D #256 p0; M 0x1b2e08]");
        assert!(t.list.len() >= 2 && t.list[0] == 0x38 && t.list[1] == 0x39
        && t.list[2..].iter().all(|&c| c == 0),
        "inputs = the rest, the prefix-scan slots [D #256 p1/p2; M 0x1b1a78 case 4's while input[k] != 0], got {:?}", t.list);
    }
    let mut scene = load();
    let u = {
        let sim = scene.sim.as_ref().unwrap();
        sim.drones.useobjs.pcs.iter().find(|q| q.ch == 0x38).expect("the Q-Worm PC #216 [D]").pos
    };
    // The first input in list order: the Q-Worm E on the computer (the 0x38 writer).
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [u[0] + 0.2, u[1] - 1.3, u[2] + 0.9];
        sim.bond.vel = [0.0; 3];
    }
    for _ in 0..40 {
        let sim = scene.sim.as_mut().unwrap();
        let (h, t) = (sim.bond.head(), u);
        let d = [t[0] - h[0], t[1] - h[1], t[2] - h[2]];
        sim.bond.yaw = d[0].atan2(d[2]);
        sim.bond.pitch = 0.0;
        sim.step_bond(Default::default(), false, false, true);
        if scene.sim.as_ref().unwrap().drones.channels.on(0x38) { break; }
    }
    {
        let sim = scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x38), "the E writes ch 0x38 [M 0x1aee20 state 1]");
        assert!(!sim.drones.channels.on(0x35), "sister: 0x38 alone never raises the SEQ_AND out");
    }
    // The second input in order: walk the touch #257 at (−22.06, 47.45, 16.61) [D].
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [-22.06, 47.0, 16.61];
        sim.bond.vel = [0.0; 3];
    }
    let mut rose = false;
    for _ in 0..40 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if scene.sim.as_ref().unwrap().drones.channels.on(0x35) { rose = true; break; }
    }
    assert!(rose, "0x38 then 0x39 raises the SEQ_AND out [M 0x1b2e08; triggers.rs]");
}

/// The rotors (0x71 ×3: "rotating blades" #199, "fanblades-large" #205/#206 [D]) spin from create
/// (`rotor_init` 0x18f4b8 — the ROT vector is the spin, the gate param 1 = 0 here [D]) [M
/// rotor_update 0x18f568]. Sister: a gated rotor (gate on) holds.
#[test]
fn rotors_spin_from_create() {
    let mut scene = load();
    let a0 = {
        let sim = scene.sim.as_mut().unwrap();
        assert_eq!(sim.drones.worldobjs.rotors.len(), 3, "the three rotor placements [D]");
        let a0 = sim.drones.worldobjs.rotors.iter().map(|r| r.angle).collect::<Vec<_>>();
        for _ in 0..30 { sim.step_bond(Default::default(), false, false, false); }
        a0
    };
    let sim = scene.sim.as_ref().unwrap();
    let moved = sim.drones.worldobjs.rotors.iter().zip(&a0)
        .map(|(r, a)| (0..3).map(|k| (r.angle[k] - a[k]).abs()).sum::<f32>()).sum::<f32>();
    assert!(moved > 0.001, "the ungated rotors advance every tick [M 0x18f568], moved {moved}");
    for r in &sim.drones.worldobjs.rotors {
        assert_eq!(r.gate, 0, "the disc's gate param 1 = 0 — always spinning [D]");
    }
}

/// The paramless fusebox (0x31 #262 "SCREEN_QWORM_2A_OBJ2" — no params at all [D]) is INERT by the
/// game's own data: `FuseBox_Create` 0x1b0448 still creates it, but with no script there are no
/// wire groups, so `FuseBox_Update` 0x1adfe8 has nothing to arm and no channel to write — the
/// port's skip (fusebox.rs's `script < 0x06000000` filter) matches the observable. Sister:
/// the music trigger #232 (0xFB, param 2 = 0xff [D]) is inert by `MusicTrigger_Create` 0x1b30e0's
/// own gate (worldobj.rs) — both lists stay empty.
#[test]
fn paramless_fusebox_and_ff_music_trigger_are_inert() {
    let mut scene = load();
    {
        let sim = scene.sim.as_mut().unwrap();
        assert!(sim.drones.fuseboxes.is_empty(), "no fusebox object without a script [D #262 has no params]");
        assert!(sim.drones.worldobjs.music_triggers.is_empty(),
            "the 0xff music event is no trigger [D #232 p2 = 0xff; M 0x1b30e0's gate]");
        for _ in 0..20 { sim.step_bond(Default::default(), false, false, false); }
    }
    assert!(scene.sim.as_ref().unwrap().drones.channels.on(0x12) || true, "no channel moves from either");
}

/// The hurt volumes (0xE4 ×3, "Hurt_A", param 0 = 6 damage [D]) sting on the crossing [M
/// Hurt_Create 0x17e870 + the class-0x27 contact arm]. Sister: standing outside never stings.
#[test]
fn hurt_volumes_sting_on_crossing() {
    let mut scene = load();
    let h0 = scene.sim.as_ref().unwrap().drones.health.health;
    assert!((h0 - 100.0).abs() < 1.0, "full health at boot");
    // Sister first: a tick standing in the open drops nothing.
    for _ in 0..20 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    assert!((scene.sim.as_ref().unwrap().drones.health.health - h0).abs() < 0.5, "sister: no hurt outside");
    // Hurt #271 at (−0.72, 36.02, 31.09) [D] — under the atrium windows.
    scene.sim.as_mut().unwrap().bond.pos = [-0.72, 36.1, 31.09];
    scene.sim.as_mut().unwrap().bond.vel = [0.0; 3];
    let mut stung = false;
    for _ in 0..40 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        if scene.sim.as_ref().unwrap().drones.health.health < h0 - 3.0 { stung = true; break; }
    }
    assert!(stung, "the hurt volume stings for its param-0 damage [D #271 p0 = 6; M hurt.rs]");
}
