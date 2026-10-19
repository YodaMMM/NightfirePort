//! level-4a (Tower-2 elevator) interactables: one live test per KIND on the playable map (m41),
//! each operated the way a player does, through the REAL input path. On this level the real path
//! IS the lift ride: SP #42 "SCRIPTPLAYER_Lift" (mode 1 [D]) plays 0x06000571 — the platform
//! entity descends 150 units from key t90 to key t5040 [D keys] — so Bond rides it and every
//! kind is operated as the ride passes it:
//! - the 13 TouchOnce plates by walk-in (mask 8 [D]), incl. the reveal plate #277 (ch 0x4d);
//! - the −52 landing doors #110/#111 by the script's unlock (ch 0x40 at t1610 [D keys]) and
//!   Door_Update's lock rising edge [M 0x1363c8], and a lower unlocked pair by the rider's
//!   straddle walk-in [M Door_Update's straddle arm, ported door.rs straddle_walkin];
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
            "the reveal plate #277 fires ch 0x4d as the car descends through its volume [D #277; M mask 8 walk-in]");
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
            "the car's pass fires #234's ch 0x17 [M Trigger_TouchOnce through the ride]");
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
    for _ in 0..2400 { r.step(Input::default()); } // ride to the end (0x4a)
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x4a), "the ride reached the lobby (the harness works)");
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
/// Spawner #233 arms on ch 0x17 (plate #234's channel, param 4 [D #233]) and releases its group's
/// templates when the car passes the −40 plate: no group-0x17 drones before, spawned drones after.
#[test]
fn the_floor_spawner_releases_its_drones_on_the_floor_plate() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let mut r = Ride::new(&iso);
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        let sp = sim.drones.spawners.iter().find(|s| s.placement == 233).expect("spawner #233");
        assert_eq!(sp.start, 0x17, "spawner #233 arms on ch 0x17 (param 4 = the start channel [D #233])");
    }
    for _ in 0..1000 { r.step(Input::default()); } // t 1000, car ≈ −27.6, before the plate
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        let sp = sim.drones.spawners.iter().find(|s| s.placement == 233).unwrap();
        assert!(sp.spawned.is_empty(), "no group drones before the plate — the temporal sister");
    }
    for _ in 0..500 { r.step(Input::default()); } // t 1500
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x17), "the plate fired [D #234]");
        let sp = sim.drones.spawners.iter().find(|s| s.placement == 233).unwrap();
        assert!(!sp.spawned.is_empty(),
            "the spawner released its group's drones on the floor channel [M 0x137e68]");
    }
}

/// KIND: the drone garrison (0x0f ×38 [D]; the shootable NPCs). One of spawner #233's spawned
/// drones (level with the car as it passes its floor) takes the player's own fire across the
/// shaft: aim + trigger through `Sim::step_bond`, health drops or it dies. Sister: the same
/// ride without firing leaves the drone unwounded (its param-5 health byte is 0x64 = 100 [D]).
#[test]
fn a_spawned_drone_takes_the_riders_fire_across_the_shaft() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set"); return };
    let mut r = Ride::new(&iso);
    for _ in 0..1500 { r.step(Input::default()); } // the plate fired at t ≈ 1420
    let (di, hp0) = {
        let sim = r.w.scene.sim.as_ref().unwrap();
        let sp = sim.drones.spawners.iter().find(|s| s.placement == 233).unwrap();
        let di = *sp.spawned.first().expect("the group's first drone spawned");
        (di, sim.drones.list[di].health)
    };
    // Fire across the shaft while the car carries him level with the drone (re-seat keeps the
    // key line; the aim tracks the LIVE drone each tick — the shoot-point pattern: turn/look
    // onto the target, trigger held).
    let mut hit = false;
    for tick in 0..2400u32 {
        {
            let sim = r.w.scene.sim.as_ref().unwrap();
            if sim.drones.list[di].corpse || sim.drones.list[di].health < hp0 { hit = true; }
        }
        if hit { break; }
        if tick % 30 == 0 {
            let sim = r.w.scene.sim.as_mut().unwrap();
            sim.bond.pos = [3.0, key_y(r.y0, r.t) + 1.2, -3.4];
            sim.drones.health.health = 100.0;
        }
        let (bp, eye, crouch, dyaw, dpitch, at_now) = {
            let sim = r.w.scene.sim.as_ref().unwrap();
            (sim.bond.pos, sim.bond.eye, sim.bond.crouch_drop, sim.bond.yaw, sim.bond.pitch,
             sim.drones.list[di].pos)
        };
        let dx = at_now[0] - bp[0];
        let dz = at_now[2] - bp[2];
        let dy = at_now[1] - (bp[1] + eye - crouch);
        let desired = dx.atan2(dz);
        let err = ((desired - dyaw + std::f32::consts::PI).rem_euclid(2.0 * std::f32::consts::PI)
            - std::f32::consts::PI) as f32;
        let dxz = (dx * dx + dz * dz).sqrt().max(0.5);
        let want_pitch = (dy / dxz).atan() / std::f32::consts::FRAC_PI_2;
        let perr = want_pitch - dpitch;
        let input = Input { turn: err.clamp(-0.08, 0.08), look: perr.clamp(-0.08, 0.08), ..Default::default() };
        r.w.tick_fire(input);
        r.t += 1;
        if r.t > 4400 { break; }
    }
    {
        let sim = r.w.scene.sim.as_ref().unwrap();
        assert!(hit || sim.drones.list[di].corpse || sim.drones.list[di].health < hp0,
            "the spawned drone took the rider's fire across the shaft (health {} from {hp0})",
            sim.drones.list[di].health);
    }
    // Sister: the same ride without firing — the drone is unwounded at its 0x64 spawn health.
    let mut r2 = Ride::new(&iso);
    for _ in 0..2000 { r2.step(Input::default()); }
    let sim = r2.w.scene.sim.as_ref().unwrap();
    let sp = sim.drones.spawners.iter().find(|s| s.placement == 233).unwrap();
    if let Some(&di2) = sp.spawned.first() {
        assert!((sim.drones.list[di2].health - 100.0).abs() < 0.01,
            "no fire → the drone keeps its 0x64 spawn health [D drone param 5] (got {})",
            sim.drones.list[di2].health);
    }
}
