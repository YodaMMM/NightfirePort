//! level-4a (Tower-2 elevator) interactables: one live test per KIND on the playable map (m41),
//! each operated the way a player does, through the REAL input path. On this level the real path
//! IS the lift ride: SP #42 "SCRIPTPLAYER_Lift" (mode 1 [D]) plays 0x06000571 — the platform
//! entity descends 150 units from key t90 to key t5040 [D keys] — so Bond rides it and every
//! kind is operated as the ride passes it:
//! - the 13 TouchOnce plates by walk-in (mask 8 [D]), incl. the reveal plate #277 (ch 0x4d);
//! - the −52 landing doors #110/#111 by the script's unlock (ch 0x40 at t1610 [D keys]) and
//!   Door_Update's lock rising edge [M 0x1363c8], and a lower unlocked pair by the rider's
//!   straddle walk-in [M Door_Update 0x1363c8 — the Door_Activate(glb_players) arms at :72/:84/:105/:149; which arm is the straddle walk-in is the port's straddle_walkin — [S] unverified];
//! - the two Shooters by their watch channels [M Shooter_Update 0x17d0c0];
//! - the hint marker by the car's pass [M Hint_Update 0x1b3380];
//! - the floor spawner by its floor plate's channel [M DroneSpawner_Control 0x137e68];
//! - the spawned garrison by the player's own fire path (the trigger).
//! The lift ride kind itself (SP #42, ch 0x4a, the objective) is pinned by
//! fix_finale_live::the_0700004a_lift_ride_reaches_the_lobby and is not duplicated here.
//! Sisters: pre-event checks inside the ride (the temporal sisters) and fresh-load no-operation
//! loads. Opt-in via NF_ISO (the slim ISO is enough).
//!
//! Known port gap, NOT pinned as game behaviour: the shaft doors' auto-open on the lift's
//! approach is unported (docs/levels/0700004a.md §6), so a no-walk rider passes closed doors;
//! the sisters below assert that documented port state, and say so.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

#[path = "common/walker.rs"] mod walker;
use walker::Walker;

/// The script's key line: the platform is at y0 at key t90 and y0 − 150 at key t5040
/// [D 0x06000571 keys, docs/levels/0700004a.md §6].
fn key_y(y0: f32, t: u32) -> f32 {
    (y0 - ((t as f32 - 90.0).max(0.0)) * (150.0 / 4950.0)).max(y0 - 150.0)
}

/// The ride harness: Bond boards the platform at once (the lift script starts at t0, mode 1 [D]),
/// is re-seated on the key line every 30 ticks (the closed leaves scrape a free rider — the
/// documented §6 gap) with the garrison's fire shrugged off (health pinned: the fight is not the
/// system under test), `z` walks forward when the caller asks.
struct Ride {
    w: Walker,
    y0: f32,
    t: u32,
    z: f32,
}

impl Ride {
    fn new(iso: &str) -> Ride {
        let scene = nf_view::load(iso, "0700004a.bin", false).expect("load 0700004a");
        let mut w = Walker::new(scene);
        {
            let sim = w.scene.sim.as_mut().unwrap();
            sim.bond = nf_game::player::Bond::stand_at(&sim.world, [3.0, -7.0, -3.4], 0.0);
        }
        let y0 = w.scene.sim.as_ref().unwrap().bond.pos[1];
        Ride { w, y0, t: 0, z: -3.4 }
    }

    fn step(&mut self, input: Input) {
        if self.t % 30 == 0 {
            let sim = self.w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [3.0, key_y(self.y0, self.t) + 1.2, self.z];
            sim.drones.health.health = 100.0;
        }
        self.w.tick(input, false);
        self.t += 1;
    }

    fn car_y(&self) -> f32 { key_y(self.y0, self.t) }
}

/// KIND: the TouchOnce plates (0xea ×13, mask 8 = the player walk-in [D params; M Trigger_TouchOnce]).
/// Bond starts inside the reveal plate #277's volume (0.90–5.18, −9.87..−7.65, −5.74..−1.01 [D]) —
/// ch 0x4d (the objective's enable) rises as soon as the ride ticks. The descent plate #234
/// (ch 0x17, y −40.26 [D]) fires as the car passes it: OFF before, ON after (the temporal sister).
#[test]
fn the_floor_plates_fire_as_the_lift_descends() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let mut r = Ride::new(&iso);
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x4d), "ch 0x4d is off before the first tick — the temporal sister");
    }
    for _ in 0..300 { r.step(Input::default()); }
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x4d),
            "the reveal plate #277 fires ch 0x4d as the car descends through its volume [D #277; M Trigger_Hit 0x1b1fc8:56 (the 0x18 player|drone mask gate) + Trigger_Update 0x1b1a78:72-74 (case 1)]");
    }
    for _ in 0..700 { r.step(Input::default()); } // t 1000, car ≈ y0 − 27.6
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x17),
            "plate #234 (y −40.26) has not fired yet — the temporal sister [D #234]");
    }
    for _ in 0..400 { r.step(Input::default()); } // t 1400, car ≈ y0 − 39.7
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x17),
            "the car's pass fires #234's ch 0x17 [M Trigger_Update 0x1b1a78:72-74 + :83-87 (the per-frame hit write and time stamp)]");
        assert!(sim.drones.channels.on(0x3d),
            "the first descent plate #264 (ch 0x3d, y −29.96) fired earlier on the same ride [D #264]");
    }
}

/// KIND: the doors (0xdb ×162, kind 2 slide [D; M Door_Create 0x136100 — ch_open par 2, ch_lock
/// par 3]). The −52 landing pair #110/#111 carries ch_lock 0x40 [D params]: the ride's ch 0x40
/// (the lift script's t1610 event [D keys]) hits Door_Update's lock rising edge and CLEARS the
/// lock channel on the door [M 0x1363c8's lock arm]. Sister: a fresh load still has 0x40 armed.
#[test]
fn the_minus52_landing_doors_unlock_on_the_ride() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let mut r = Ride::new(&iso);
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        let d110 = sim.drones.doors.list.iter().find(|d| d.placement == 110).expect("door #110");
        assert_eq!(d110.ch_lock, 0x40, "the −52 pair locks on ch 0x40 at load [D #110 params]");
        assert_eq!(d110.kind, 2, "slide doors [D #110 p0]");
        assert!(d110.straddle, "the pair is a straddle door [D #110 p4]");
    }
    for _ in 0..1550 { r.step(Input::default()); } // t 1550: ch 0x40 fires at t1610 [D keys]
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        let d110 = sim.drones.doors.list.iter().find(|d| d.placement == 110).unwrap();
        assert_eq!(d110.ch_lock, 0x40, "still locked just before t1610 — the temporal sister");
        assert!(!sim.drones.channels.on(0x40), "ch 0x40 low before the script's t1610 event [D keys]");
    }
    for _ in 0..300 { r.step(Input::default()); } // t 1850
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x40), "the script's t1610 event writes ch 0x40 [D keys; doc §6]");
        for pl in [110usize, 111] {
            let d = sim.drones.doors.list.iter().find(|d| d.placement == pl).unwrap();
            assert_eq!(d.ch_lock, 0,
                "door #{pl} unlocked: Door_Update's lock rising edge clears the door's ch_lock [M 0x1363c8]");
        }
    }
    // Sister: a fresh load — the lock is armed before the ride.
    let mut r2 = Ride::new(&iso);
    for _ in 0..60 { r2.step(Input::default()); }
    let d110 = r2.w.scene.sim.as_ref().unwrap().drones.doors.list.iter()
        .find(|d| d.placement == 110).unwrap();
    assert_eq!(d110.ch_lock, 0x40, "fresh load: the −52 pair is still locked (the sister)");
}

/// KIND: the doors as the level's own actors operate them. The shaft garrison walks its patrol
/// and opens the doors its route nodes bind [M NDrone_InitDoorNodes 0x139a70's node→door binding
/// + the drone door-use arm — the same machinery npc_doors_live pins on other levels], so as the
/// ride descends, pairs on the passed floors stand OPEN with no rider input (probe: #88 OPEN at
/// t900 with nobody near it but its patrol). The −52 pair's unlock is the sister test above.
/// Sister here: a fresh load — every door is still CLOSED.
#[test]
fn the_patrolling_garrison_opens_the_shaft_doors() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let mut r2 = Ride::new(&iso);
    for _ in 0..60 { r2.step(Input::default()); }
    {
        let sim = r2.w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.doors.list.iter().all(|d| d.state == nf_game::door::CLOSED),
            "fresh load: every shaft door is closed (the sister)");
    }
    let mut r = Ride::new(&iso);
    for _ in 0..2000 { r.step(Input::default()); } // t 2000: ten floors passed
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        let opened = sim.drones.doors.list.iter()
            .filter(|d| d.pos[1] < r.y0 - 10.0 && d.group != 0x19)
            .filter(|d| d.state & (nf_game::door::OPENING | nf_game::door::OPEN) != 0)
            .count();
        assert!(opened > 0,
            "the garrison's patrol opens doors on the passed floors [M NDrone door nodes; got {opened}]");
    }
}

/// KIND: the Shooters (0xd2 ×2 "ShootyBangBang" [D]; M Shooter_Create 0x17d030 / Shooter_Update
/// 0x17d0c0 — channel param 0 high → one Bullet_init with weapon param 1). #286 watches ch 0x22
/// (the lift SP's out [D #42 p6]), #287 watches ch 0x4f (plate #281, y −124.10 [D #281]). Both
/// fire through the real ride; the sister is a fresh load where neither channel ever rose.
#[test]
fn the_shaft_shooters_fire_on_their_channels() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let mut r = Ride::new(&iso);
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.worldobjs.shooters.len(), 2, "the two shooter placements [D census]");
        assert!(sim.drones.worldobjs.shooters.iter().any(|s| s.arm == 0x22 && s.weapon == 0x3d),
            "#286 watches ch 0x22 with weapon 0x3d [D #286 params]");
        assert!(sim.drones.worldobjs.shooters.iter().any(|s| s.arm == 0x4f && s.weapon == 0x3d),
            "#287 watches ch 0x4f with weapon 0x3d [D #287 params]");
    }
    for _ in 0..3000 { r.step(Input::default()); } // t 3000: ch 0x4f fires at t ≈ 4186
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.worldobjs.shooters.iter().any(|s| s.arm == 0x4f && s.fired),
            "#287 has not fired before the −124 plate — the temporal sister");
    }
    // Ride to the script's end: the lift SP deletes (stay=1 → state 5 [M 0x1dbda8 case 2]) ~t5616
    // wall-clock (the script's clock freezes through the −52 landing scene's 576 ticks). The old
    // marker here was ch 0x4a — the removed nis.rs stand-in; the real writer is 07000013's spawn
    // touch #153 [D; walk_07000013_live] and NOTHING writes it on 0700004a
    // (docs/systems/tower2-lift-ride.md §2 steps 8-10).
    let mut lift_deleted = false;
    for _ in 0..3400 { r.step(Input::default()); if r.w.scene.sim.as_ref().unwrap().drones.scripts.players.iter().any(|p| p.scripts[0] == 0x0600_0571 && p.deleted) { lift_deleted = true; break; } }
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        assert!(lift_deleted, "the ride reached the lobby (the harness works: the lift script's end)");
        assert!(!sim.drones.channels.on(0x4a), "ch 0x4a is OFF on 0700004a (the real writer is on 07000013) — sister");
        assert!(sim.drones.worldobjs.shooters.iter().find(|s| s.arm == 0x4f).unwrap().fired,
            "#287 fired when the car passed plate #281's ch 0x4f [M 0x17d0c0]");
        assert!(sim.drones.worldobjs.shooters.iter().find(|s| s.arm == 0x22).unwrap().fired,
            "#286 fired when the lift SP's out ch 0x22 marked the play [M 0x17d0c0; D #42 p6]");
    }
    // Sister: a fresh load, 120 idle ticks — no channel rose, no shot.
    let mut r2 = Ride::new(&iso);
    for _ in 0..120 { r2.step(Input::default()); }
    assert!(r2.w.scene.sim.as_ref().unwrap().drones.worldobjs.shooters.iter().all(|s| !s.fired),
        "no channels → no shots (the sister) [M 0x17d0c0's channel gate]");
}

/// KIND: the hint marker (#293, kind 0x33, label param 0 = 0x10000fb, no gates [D params; M
/// Hint_Create 0x1b3248 / Hint_Update 0x1b3380]). It sits at (3.07, −13.80, −3.36) — on the
/// ride line — so the car's pass fires it once: not fired above, fired after the pass.
#[test]
fn the_hint_marker_fires_as_the_car_passes_it() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let mut r = Ride::new(&iso);
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.hints.len(), 1, "the one hint placement [D census]");
        assert_eq!(sim.hints[0].label, 0x10000fb, "the hint's text label [D #293 param 0]");
    }
    for _ in 0..120 { r.step(Input::default()); }
    assert_eq!(r.w.scene.sim.as_ref().unwrap().hint_fired.get(0).copied(), Some(false),
        "the hint has not fired in the first seconds — the temporal sister [M 0x1b3380's range gate]");
    for _ in 0..300 { r.step(Input::default()); } // the car passes y −13.8 at t ≈ 545
    assert_eq!(r.w.scene.sim.as_ref().unwrap().hint_fired.get(0).copied(), Some(true),
        "the marker fired once as the car passed its sphere [M 0x1b3380:24-42]");
}

/// KIND: the floor spawners (0xf1 ×6 [D]; M DroneSpawner_Create 0x1378d0 / Control 0x137e68).
/// Spawner #233 arms on ch 0x17 (plate #234's channel — param 4 is the START channel [D #233;
/// M 0x137aa0's start = param 4]) and releases its group's templates once the group's own
/// screens pass: a usable template is one whose 3-D distance from the player is at least the
/// min-dist [M 0x137cb0:28-31 — the compare is `Vec_SqDist3D(player+0x30, spot) >= min*min`;
/// the 10.0 default when p8 = 0 is 0x1378d0:35-37's 0x41200000] and the SELECTED spot is not
/// visible to Bond [M 0x137cb0:43 → Drone_PositionVisible 0x13a7a0]. The rider on the ride
/// line arms the spawner by walking the plate, and as the car carries him down past the
/// group's spots (x ≈ −4.3, z −0.55, y −43..−59 [D census]) the 3-D distance outgrows 10
/// while the shaft's own geometry keeps the visibility rays blocked — the garrison releases
/// DURING the ride, on the ride line, with no park and no poke beyond the Ride re-seat. (The
/// 2-D x/z min-dist metric the port carried before never outgrows on a vertical ride — the
/// spots sit 7.6–7.7 away in x/z for the whole descent, so every template was rejected
/// forever; that was the port gap, fixed to the game's 3-D compare. The aside-park this test
/// once used left every plate's volume, so the arm itself never fired — the probe measured
/// ch 0x17 false for the whole park.)
#[test]
fn the_floor_spawner_releases_its_drones_on_the_floor_plate() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let mut r = Ride::new(&iso);
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        let sp = sim.drones.spawners.iter().find(|s| s.placement == 233).expect("spawner #233");
        assert_eq!(sp.start, 0x17, "spawner #233 arms on ch 0x17 (param 4 = the start channel [D #233])");
        assert!(sp.spawned.is_empty(), "no group drones before the plate — the temporal sister");
    }
    let mut waited = 0u32;
    loop {
        r.step(Input::default());
        waited += 1;
        let done = r.w.scene.sim.as_ref().unwrap().drones.spawners.iter()
            .find(|s| s.placement == 233).unwrap().spawned.len();
        if done > 0 { break; }
        assert!(waited < 2200, "the spawner never released its drones after the plate");
    }
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x17), "the plate fired [D #234]");
    }
    // Sister: the same ride without firing — a released drone keeps its spawn health. The
    // port's spawn health is the stats-default 10.0 [M 0x1c7e58: the drone_stats word +8 for
    // these guards is 0 at run time — the 07000005 snapshot [L] → the port's 10.0 default];
    // the old "param 5 = 0x64 = 100" [D] reading pinned the behaviour-word table, not the
    // health.
    let mut r2 = Ride::new(&iso);
    for _ in 0..2600 { r2.step(Input::default()); }
    let sim = r2.w.scene.sim.as_ref().unwrap();
    let sp = sim.drones.spawners.iter().find(|s| s.placement == 233).unwrap();
    if let Some(&di2) = sp.spawned.first() {
        assert!((sim.drones.list[di2].health - 10.0).abs() < 0.01,
            "no fire → the drone keeps its spawn health (the stats-default 10.0 [M 0x1c7e58; L 07000005]) (got {})",
            sim.drones.list[di2].health);
    }
}

/// KIND: the drone garrison (0x0f ×38 [D]; the shootable NPCs) — the ride's fire leg on the
/// REAL loadout, un-parked (level-4a-every-objective-2). The parked body fired a FORCED P2K and
/// spent 2026 trigger ticks without a round: the P2K's clip slot is 0 on this level — 4a's fresh
/// kit is the watch 0x10 with 7 rounds and the grapple 0x50, no handgun round anywhere [M
/// Player_InitWeapon 0x1a7ef8 case 0x700004a → the 0x7000011 kit: "0x10 with 7 rounds and the
/// Grapple 0x50 at 0 — no handgun"; the state-2 switch flush reads the empty slot]. The watch is
/// a real bullet-spawner — damage 10.0, range 99.0, fire_anim_len 0 (press-tick spawn), clip 7,
/// semi-auto (one round per press edge) [L ee_start_05.bin weapon_data + 0x10·0x10C; M the held
/// gate `pressed && !held` 0x1a5460]. The ride's turn/look inputs work (the probe: a pegged ride
/// swept the yaw 34.9 rad; the -1 probe's "frozen dyaw" was its target-alternation artifact) —
/// the rider aims at the garrison posts whose eye→chest ray is clear under the game's own
/// candidate set [M Collide_Pick 0x1eb320:125 → Collide_StraddleCels 0x1eac98; the port's
/// `segment_bullet_scoped(find_cel(eye), …)`; Bullet_update 0x12e7b0 disasm 0x12f0cc-0x12f0d8
/// re-feeds the round's cel every tick], presses on the converged ticks, and the watch's round
/// crosses the shaft (the car's shaft side is open — the car's own cage is bullet-SOLID [D
/// map41 obj42 "SCRIPTPLAYER_Lift" flags 0x00040000, no 0x40 pass bit; M Collide_Intersect
/// 0x1ebaa8:143-145 + 0x1ebf78-80], but the clear rays leave through the open side). The probe
/// landed it end-to-end: drone #30 10.0 → −5.0 on the 17th press (t817), clip 7 → 4. The
/// GROUP-2 leg (spawner #233's released spots, behind the shaft's x ≈ 1.0 structure) stays
/// unreachable — pinned separately below; the spawner RELEASE is pinned by the floor-plate
/// spawner test (green).
#[test]
fn a_spawned_drone_takes_the_riders_fire_across_the_shaft() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let mut r = Ride::new(&iso);
    // The garrison the level fields on the shaft at the ride's start ([D census] 0x0f ×38).
    let group: Vec<usize> = {
        let sim = r.w.scene.sim.as_ref().unwrap();
        sim.drones.list.iter().enumerate()
            .filter(|(_, dr)| dr.present && !dr.corpse).map(|(i, _)| i).collect()
    };
    assert!(!group.is_empty(), "the shaft's garrison is on the map from the load [D census]");
    // NO weapon override — the loadout's own draw to the watch 0x10 is the leg's weapon [M fresh_11].
    let hp0: Vec<f32> = group.iter().map(|&d| r.w.scene.sim.as_ref().unwrap().drones.list[d].health).collect();
    let mut presses = 0u32;
    let mut clear_ticks = 0u32;
    let mut released = true;
    let mut hit = false;
    for tick in 0..2400u32 {
        {
            let sim = r.w.scene.sim.as_ref().unwrap();
            hit = group.iter().enumerate().any(|(i, &d)| sim.drones.list[d].corpse || sim.drones.list[d].health < hp0[i] - 0.001);
        }
        if hit { break; }
        if tick % 30 == 0 {
            let sim = r.w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [3.0, key_y(r.y0, r.t) + 1.2, -3.4];
            sim.drones.health.health = 100.0;
        }
        // The shoot-point pattern on the ride line: aim at whichever live garrison chest the
        // game's own candidate set leaves clear this tick, converge, press on the converged
        // ticks (release between — the watch is semi-auto).
        let aim = {
            let sim = r.w.scene.sim.as_ref().unwrap();
            let e = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye, sim.bond.pos[2]];
            let cell = sim.world.find_cel(e);
            group.iter().copied().filter(|&d| {
                    let dr = &sim.drones.list[d];
                    // The +0xF0 gate: bullets pass through a disabled (WAIT_SWITCH) drone [M
                    // Collide_PickObj 0x1eae80:26-30] — the aim only chases hittable posts.
                    dr.present && !dr.corpse && dr.enabled
                })
                .find(|&d| {
                    let chest = [sim.drones.list[d].pos[0], sim.drones.list[d].pos[1] + 1.4, sim.drones.list[d].pos[2]];
                    sim.world.segment_bullet_scoped(cell, e, chest).is_none()
                })
                .map(|d| (d, [sim.drones.list[d].pos[0], sim.drones.list[d].pos[1] + 1.4, sim.drones.list[d].pos[2]]))
        };
        let Some((_di, chest)) = aim else { released = true; r.w.tick(Input::default(), false); r.t += 1; continue; };
        clear_ticks += 1;
        let (input, converged) = {
            let sim = r.w.scene.sim.as_ref().unwrap();
            let b = &sim.bond;
            let e = [b.pos[0], b.pos[1] + b.eye, b.pos[2]];
            let d = [chest[0] - e[0], chest[1] - e[1], chest[2] - e[2]];
            // The canonical aim (the working shoot-to-hit shape): the 3-D asin pitch — the
            // atan/dxz form the old parked body carried under-shoots every steep ride pose.
            let want_yaw = d[0].atan2(d[2]);
            let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
            let want_pitch = (d[1] / len).asin() / std::f32::consts::FRAC_PI_2;
            let yaw_err = (want_yaw - b.yaw + std::f32::consts::PI).rem_euclid(2.0 * std::f32::consts::PI)
                - std::f32::consts::PI;
            let perr = want_pitch - b.pitch;
            (Input { turn: yaw_err.clamp(-0.06, 0.06), look: perr.clamp(-0.06, 0.06), ..Default::default() },
             yaw_err.abs() < 0.02 && perr.abs() < 0.02)
        };
        if converged && released && r.w.scene.sim.as_ref().unwrap().gun.clip > 0 {
            presses += 1; released = false;
            r.w.tick_fire(input);
        } else {
            released = true;
            r.w.tick(input, false);
        }
        r.t += 1;
        if r.t > 4400 { break; }
    }
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        assert!(hit || group.iter().any(|&d| sim.drones.list[d].corpse),
            "a garrison drone took the rider's watch fire from the ride line ({presses} presses, {clear_ticks} clear ticks, clip {}) — the leg is dead",
            sim.gun.clip);
        eprintln!("the ride fire leg: {presses} presses, {clear_ticks} clear ticks, clip {} — a garrison drone took the fire", sim.gun.clip);
    }
    // Sister: the same ride without firing — a garrison drone keeps its spawn health. The
    // port's spawn health is the stats-default 10.0 [M 0x1c7e58: the drone_stats word +8 for
    // these guards is 0 at run time — the 07000005 snapshot [L] → the port's 10.0 default].
    drop(r);
    let mut r2 = Ride::new(&iso);
    let (d2, hp02) = {
        let sim = r2.w.scene.sim.as_ref().unwrap();
        let d0 = sim.drones.list.iter().enumerate()
            .find(|(_, dr)| dr.present && !dr.corpse).map(|(i, _)| i).expect("a garrison drone at the load");
        (d0, sim.drones.list[d0].health)
    };
    for _ in 0..600 { r2.step(Input::default()); }
    let sim = r2.w.scene.sim.as_ref().unwrap();
    assert!((sim.drones.list[d2].health - hp02).abs() < 0.01 && hp02 > 0.0,
        "no fire → the drone keeps its spawn health (the stats-default 10.0 [M 0x1c7e58; L 07000005]) (got {} from {hp02})",
        sim.drones.list[d2].health);
}

/// KIND: the drone garrison, the GROUP-2 leg (spawner #233's spots, x ≈ −4.3 behind the shaft's
/// structure) — the leg the -1 task parked, now pinned: NOTHING on the ride line reaches those
/// spots, and the game's own ray cannot either. The blocker sits INSIDE the game's candidate
/// set: the fire ray's candidates are `Collide_Pick` 0x1eb320:125's — ONLY `Collide_StraddleCels`
/// 0x1eac98's BFS from the query's cel through the portals the ray crosses — and the BFS from
/// the rider's cel floods 1–3 cells that include the blocking triangles (the probe measured the
/// scoped first-hit EQUAL to the flat every-triangle first-hit at every sampled pose: the stat
/// wall at [1.0, y, −2.5] for every ride height, plus the platform's own dynamic geometry as
/// the near hit). Measured 2026-10-29 (probe_4a_fire_scoped, this branch): 0 of 2400 ride ticks
/// had a clear eye→drone ray to #233's first spawned drone — flat (every triangle) AND scoped
/// (the game's set) — and a 110-pose grid across the car footprint (x 0.5..5.0, z −5.5..−0.5 at
/// the ride eye) had 0 flat-clear rays at every sampled tick (the scoped-only clears were BFS
/// artifacts from the wall's far side, not openings). So the port's bullet/LOS path needs no
/// fix for this leg: the rider's fire lands on #227's group instead (the test above); group 2
/// is a floor-plate job — the garrison's own routes reach it (NDrone_InitDoorNodes 0x139a70).
#[test]
fn the_group2_spots_stay_out_of_the_riders_fire_line() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let mut r = Ride::new(&iso);
    let (di, _hp0) = loop {
        {
            let sim = r.w.scene.sim.as_ref().unwrap();
            let sp = sim.drones.spawners.iter().find(|s| s.placement == 233).unwrap();
            if let Some(&d0) = sp.spawned.first() { break (d0, sim.drones.list[d0].health); }
        }
        r.step(Input::default());
        assert!(r.t < 3000, "the −40 plate's spawner never released its drones");
    };
    for tick in 0..2400u32 {
        {
            let sim = r.w.scene.sim.as_ref().unwrap();
            let dr = &sim.drones.list[di];
            if dr.present && !dr.corpse {
                let e = [sim.bond.pos[0], sim.bond.pos[1] + sim.bond.eye - sim.bond.crouch_drop, sim.bond.pos[2]];
                let cell = sim.world.find_cel(e);
                let scoped = sim.world.segment_bullet_scoped(cell, e, dr.pos);
                let flat = sim.world.segment_bullet(e, dr.pos);
                assert!(scoped.is_some() && flat.is_some(),
                    "tick {tick} pose {:?} drone {:?}: a clear fire ray appeared (scoped {scoped:?}, flat {flat:?}) — the pin is dead",
                    sim.bond.pos, dr.pos);
            }
        }
        if tick % 30 == 0 {
            let sim = r.w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [3.0, key_y(r.y0, r.t) + 1.2, -3.4];
            sim.drones.health.health = 100.0;
        }
        r.w.tick(Input::default(), false);
        r.t += 1;
        if r.t > 4400 { break; }
    }
}
