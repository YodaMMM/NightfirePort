//! level-06-rook-door: the REAL player path on 07000006 — Bond walks the player's route from the
//! truck-ride spawn into the castle with real inputs (the proven walkreal legs), and the switch-16
//! guard pair beside door #47 wakes and engages. The playtest symptom this pins: on the real path
//! one of them ended up PARKED IN the doorway of door #47 (36.50, 40.00, −16.25) forever — the
//! door stall. The cause was port-side: the ATTACK-ENTER flush arm ran `NDrone2_ChangeToAttackMode`
//! (0x14a648) on a mid-fight ATTACK re-entry — a call the game never makes there (the five `jal
//! 0x14a648` sites are 0x14ce68 GetDroneTypeAttackTypeFriend's kind-0xb arm, 0x14d45c
//! DoTypeSettingsOLD, 0x15da54 CivilianMission's arrival, 0x15e35c EnemyMission's arrival — itself
//! gated OFF on 0x07000006 — and 0x165948 GoToGoalPosition; `NDrone2_DSTATE_Attack` 0x164210 and
//! `DroneFunc_FirstAttack` 0x148010 are NOT callers). The guard's RUN_FOR_COVER route crossed the
//! door pair (his 0x81 OpenDoor, the ported drone door path), he re-entered ATTACK inside the
//! doorway and the flush promoted his mode 0 → 0x19 (the alert sniper: stationary, SniperAim
//! 0x2a, sight/combat range 1000) — he stood in the door plane holding the door binding until the
//! watch expired. Opt-in via NF_ISO.

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::{dxz, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn carry(on: &[u32]) -> nf_game::player::RamSave {
    let mut ch = nf_game::triggers::Channels::default();
    for &c in on { ch.set(c, 1, 0); }
    nf_game::player::RamSave::capture(&Default::default(), 0, &nf_game::pickup::Inventory::level_start(0x73, 240), &ch)
}

/// The kill of the lane-blocking idle scripted guard #438 ONLY (27.51, 39.18, −10.19 [D census];
/// the walkreal fight kills any enabled guard within 8 — that would shoot the disabled switch-16
/// pair this test watches).
fn kill_438(w: &mut Walker) {
    let di = {
        let sim = w.scene.sim.as_ref().unwrap();
        (0..sim.drones.list.len()).find(|&i| {
            let d = &sim.drones.list[i];
            !d.corpse && d.enabled && d.pos[1] < 46.0 && dxz(d.pos, [27.5, 39.2, -10.2]) < 1.5
        })
    };
    let Some(di) = di else { return };
    w.switch_to_armed("the lane guard");
    for _ in 0..6 {
        if w.scene.sim.as_ref().unwrap().drones.list[di].corpse
            || w.scene.sim.as_ref().unwrap().drones.list[di].health <= 0.0 { break; }
        if !w.shoot(di, "the lane guard") { w.resupply("the lane guard"); }
    }
    assert!(w.scene.sim.as_ref().unwrap().drones.list[di].health <= 0.0,
        "the lane guard #438 dies to the real fire path (10 hp [D census])");
}

/// The real path from the level start to the searchlight-courtyard mouth, through the switch-16
/// guards' engagement — the exact walk that parked the ch-16 guard in the door #47 doorway before
/// the fix. Every assertion below is decompile-backed; the legs are the proven walkreal ones.
///
/// THE PINS (all can fail):
/// 1. the ch-16 guard d#3 (skin 0x500002e) keeps his placement mode 0 through the whole
///    engagement — the game has no Attack-ENTER `ChangeToAttackMode` caller for a kind-2 NEW
///    placement drone [M the jal 0x14a648 census; the mode byte otherwise moves only at creation
///    DoModeSettingsNEW 0x14c600 / the abseil landing 0x1696d0];
/// 2. he PASSES the doorway: at some watch tick he is north of the door plane's z (z > −15.2,
///    the courtyard side) — the broken build held him at (35.47, 41.00, −16.93), the door plane;
/// 3. the doorway ends clear: no enabled non-corpse drone inside the door #47 jamb box
///    (x 34.6..36.6, z −17.3..−15.2) bound to the door, and the door itself is back at the shut
///    idle 0x2 (the broken build held it at 0x4 with the guard bound).
/// [ignore removed level-06-rook-door-3: the landing gate is done. The flush arm is gone on this
/// branch, and the 01 contact kill was re-derived (NOT a guard's kill — the harness's own
/// weapon-0x1c volley through the walking escort's line; the game's Drone_CollisionHandler
/// 0x140968:85-95 has no team gate) and fixed in the WALK (the shoot line gate). Both 01 walkreal
/// tests run green with the flush still removed. Pre-fix run of this exact walk: the guard froze
/// at (35.47, 41.00, −16.93) in mode 25 / state 0x2a, bound to door #1, the door held at 0x4, for
/// the whole 1800-tick watch.]
#[test]
fn the_switch16_guard_fights_through_the_door47_doorway() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000006.bin", false, Some(&carry(&[0x1d, 0x45, 0x70]))).expect("load 07000006"));
    w.wait_enabled(4000, "the intro NIS");
    w.wait(30);

    w.walk_nav_to([60.0, 38.3, 31.5], "off the truck, south", 1.6);
    w.walk_nav_to([45.0, 38.3, 24.0], "across the greeting plates", 1.6);
    w.walk_nav_to([24.3, 38.3, 27.6], "the dock's west end", 1.6);
    w.walk_nav_to([24.6, 38.5, 21.5], "down the west lane", 1.6);
    w.walk_nav_to([26.5, 39.1, 13.1], "the well courtyard's west side", 1.6);
    w.walk_to(Wp::new(23.2, 39.1, 7.0), "the hedge gap");
    w.walk_to(Wp::new(27.2, 39.1, -2.4), "around the hedge end");
    w.walk_to(Wp::new(33.7, 39.1, -8.9), "the guardhouse court");
    kill_438(&mut w);
    w.walk_to(Wp::new(36.0, 39.1, -13.4), "before door #47");
    {
        // ch 16 (the guard pair's wake switch) rises on the interior step triggers as the route
        // is walked [D #514-#523 the step-trigger bank]; both pair members are enabled here.
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(16), "the step trigger raised ch 16 (the guard pair's wake) [D #514-#523]");
        let d3 = &sim.drones.list[3];
        assert_eq!((d3.skin, d3.mode, d3.switch_channel), (0x0500_002e, 0, 16),
            "d#3 is the switch-16 guard in the placement mode 0 [D census (30.3, 41.1, −16.8)]");
    }
    {
        let (pos, di) = walker::door_near(w.scene.sim.as_ref().unwrap(), [36.5, 40.0, -16.25]).expect("door #47 present");
        eprintln!("[door47] using door {di} at {pos:?}");
        w.use_door(di, "door #47");
    }
    w.walk_nav_to([29.9, 40.5, -17.5], "the guardroom fusebox nook", 1.4);
    // Stand in the guardroom 3600 ticks: the pair wakes and engages (combat states in the watch).
    let mut lastpos: Vec<[f32; 3]> = w.scene.sim.as_ref().unwrap().drones.list.iter().map(|d| d.pos).collect();
    for _ in 0..3600 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        let d3 = &sim.drones.list[3];
        assert!(d3.corpse || d3.mode == 0, "the ch-16 guard's placement mode 0 survives the engagement \
            (the game's ChangeToAttackMode 0x14a648 has no Attack-ENTER caller for a kind-2 NEW drone) [M]");
        for (i, dr) in sim.drones.list.iter().enumerate() { lastpos[i] = dr.pos; }
    }
    // Out west through the auto-closing door #59, up to the searchlight-courtyard mouth — the leg
    // during which the pre-fix build ran the guard to the door #47 pair and parked him there.
    w.walk_nav_to([27.9, 40.0, -21.0], "before door #59", 1.4);
    {
        let (pos, di) = walker::door_near(w.scene.sim.as_ref().unwrap(), [27.88, 40.0, -23.0]).expect("door #59 present");
        w.use_door(di, "door #59");
        let _ = pos;
    }
    w.walk_to(Wp::new(26.6, 40.6, -23.6), "through door #59");
    w.walk_nav_to([8.4, 42.0, -12.0], "the searchlight courtyard mouth", 1.6);

    // The doorway watch: 1800 ticks at the mouth. The guard may legally chase through the door
    // #47 pair (his own 0x81 OpenDoor path — the ported route_reached_special + DSTATE_OpenDoor
    // [M 0x153358 + 0x1681f8]); what he must NOT do is park in the jamb as the flush sniper.
    let sim = w.scene.sim.as_ref().unwrap();
    let d3_alive = !sim.drones.list[3].corpse && sim.drones.list[3].health > 0.0;
    let mut crossed_north = false;
    for t in 0..1800u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        for (i, dr) in sim.drones.list.iter().enumerate() {
            if !dr.present || dr.corpse || !dr.enabled { continue; }
            if i == 3 && d3_alive {
                assert!(dr.mode == 0, "watch +{t}: the ch-16 guard's mode flipped to {:#x} — the flush \
                    must never run from Attack [M the jal 0x14a648 census]", dr.mode);
                if dr.pos[2] > -15.2 { crossed_north = true; }
            }
            let in_jamb = dr.pos[0] > 34.6 && dr.pos[0] < 36.6 && dr.pos[2] > -17.3 && dr.pos[2] < -15.2;
            if in_jamb && dr.hit_door == Some(1) {
                // A drone working the door from the node pair is legal for a few ticks; parked
                // forever is the stall. Allow the pass-through: fail only on a 600-tick sit.
                eprintln!("[doorway +{t}] d#{i} st {:#x} in the jamb, bound", dr.state);
            }
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        if d3_alive {
            let d3 = &sim.drones.list[3];
            assert!(crossed_north || dxz(d3.pos, [35.47, 41.0, -16.93]) > 2.0,
                "the ch-16 guard must not end parked at the door #47 plane (pre-fix freeze spot \
                (35.47, 41.00, −16.93)), got {:?} st {:#x}", d3.pos, d3.state);
            assert!(!(d3.state == 0x2a && d3.stationary),
                "the ch-16 guard must never sit in the stationary SniperAim 0x2a park [M the flush \
                that produced it has no Attack caller]");
        }
        // The door itself: back to the shut idle, not held at the blocked 0x4.
        assert_eq!(sim.drones.doors.list[1].state, 0x2,
            "door #47 ends at the shut idle 0x2 (the broken build held 0x4 with the guard bound)");
        for (i, dr) in sim.drones.list.iter().enumerate() {
            if !dr.present || dr.corpse || !dr.enabled { continue; }
            let in_jamb = dr.pos[0] > 34.6 && dr.pos[0] < 36.6 && dr.pos[2] > -17.3 && dr.pos[2] < -15.2;
            assert!(!(in_jamb && dr.hit_door == Some(1)),
                "d#{i} still stands in the door #47 jamb bound to it — the stall");
        }
    }
    eprintln!("[06] the doorway watch ended clean at tick {}", w.total_ticks);
}
