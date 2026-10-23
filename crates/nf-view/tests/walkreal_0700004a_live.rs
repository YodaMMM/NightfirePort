//! walkreal-c: 0700004a (the Tower-2 elevator) walked with the player's own movement — no
//! `bond.pos` writes after the spawn. The route (docs/levels/0700004a.md; the walkreal_probe_c
//! dump + the spawn-support probe): the arrival (4.26, −7.71, −4.76) is ON the lift platform
//! (the top lobby's static floor is only the strip z 0..4; the shaft opens south — the probe's
//! t0-300 tick log shows Bond grounded on the platform from t30, descending with the script's
//! key line). The walk is therefore: board (the spawn), RIDE SP #42 "SCRIPTPLAYER_Lift" (mode 1
//! [D], 0x06000571: the platform descends 150 units t90→t5040 [D keys]) with no input, the floor
//! plates firing on the way (the first plate #277's ch 0x4d reveals the objective), and at the
//! bottom WALK north off the platform into the lobby. The ride's end writes NO channel: the ride
//! holds at the −52 landing for the landing cut-scene (SP #289, pack 0780004a — its end event
//! writes ch 0x22 [D]) and the script's end is silent; "Gain access to lobby" (ch 0x4a) completes
//! on 07000013's spawn touch #153 [D; walk_07000013_live]. docs/systems/tower2-lift-ride.md.
//!
//! The shaft doors' auto-open on the lift's approach is the KNOWN GAP (§6): fix_finale_live had
//! to re-seat Bond every 30 ticks because the closed leaves scraped him off mid-ride. THIS test
//! rides with no re-seating — a scrape shows as the ride leaving the key line (the fail-first
//! for the fix). Opt-in via NF_ISO (the slim ISO is enough).

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

// level-4a-lift-3 (2026-10-23): the FRESH-BOOT live audit [L PCSX2, god mode, no input] pins the
// real end-to-end chain, and it OVERTURNS the composed hand-back:
// ① The ride: Bond ACTIVE (BLData+0x94a = 1, the normal standing-on-the-platform state [M
//    Player_Update 0x193600:78-86 — 0x94a 1 runs the full update, 0 returns frozen, >1 counts
//    down]) riding the descending platform at 0.0303 u/frame at (4.2555, y, −4.7581) [L].
// ② The stop: the lift script's pause (stream 4, event 3 p0=0 → the −1 hold [D 0x06000571]) at
//    t1611 freezes the platform with Bond's root at −54.1971 [L].
// ③ The window: the landing cut-scene 0x060006cf (SP #289, pack 0780004a [D]) plays 575 ticks
//    [D len]; its s9 event 4 at t0 = Player_Disable [M 0x1d94c0 case 4 → 0x195f70:16 sets 0x94a=0]
//    — the live "substate 0 for ~560 frames" IS the disable (0x94a 0 = the frozen player), not
//    control.
// ④ The hand-back: at the scene's t575 the s25 marker (Entity hash 0, obj_type 0x19, no parent,
//    key (1.79945, −54.12811, −3.45340) [D key 1409]) is created, s26 event 6 [1] snapshots its
//    matrix into SCRIPTINFO+0x40 [M 0x1d94c0 case 6], s27 writes ch 0x22 [D], and Script_Free's
//    bit 0x100 runs Player_Enable(glb_players, +0x40, 1) [M 0x1da880:46-49]. The live re-attach
//    pos = the marker key's RAW x/z (1.7994, −3.4534) [L] — the SP-base compose is SKIPPED for
//    the hash-0 marker class (the old composed reading (1.799, −107.662, −6.830) is refuted; the
//    port's fix in nis.rs case 1). The enable sets 0x94a = 2 → 1 (active) [M 0x195498:51].
// ⑤ The death: with no input, Bond carried/falling down the shaft dies at root −95.80 — health
//    100 → 0 in one frame, mission fail [L]. The port pins the y through the real death flow
//    (drone.rs's shaft kill, [S] mechanism — no disc writer found: dumps l3/l3b/l3c).
// ⑥ The route: walk OFF at the −52 landing — the landing's floor is at −56.0 (x −3..5, z −1..1.5
//    [L probe l3c]), 0.77 below the held platform's top; the −52 doors #110/#111 unlock on the
//    lift's t1610 ch-0x40 write [D p3=0x40; M Door_Update 0x1363c8]. Staying on the resumed lift
//    ends in ⑤ (the sister below). ch 0x4a still never fires on 0700004a [D; walk_07000013_live].

#[test]
fn ride_the_lift_from_the_spawn_and_walk_into_the_lobby() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700004a.bin", false).unwrap());
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] > 2.0 && p[0] < 6.0 && p[1] > -9.0 && p[1] < -6.0 && p[2] > -6.0 && p[2] < -3.0,
            "the arrival spawns at the shaft's top, on the platform (#1 [D]), got {p:?}");
    }
    // Board: settle onto the platform; the reveal plate #277 fires ch 0x4d under the first steps.
    let mut fired_plate = false;
    for k in 0..84 {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let target = if k < 42 { [3.2, 0.0, -2.2] } else { [3.2, 0.0, -3.8] };
        let desired = (target[0] - p[0]).atan2(target[2] - p[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        w.tick(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, ..Default::default() }, false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x4d) { fired_plate = true; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.bond.grounded, "Bond settles back onto the lift platform after the first steps");
        assert!(fired_plate && sim.drones.channels.on(0x4d), "the shaft's first plate #277 fires ch 0x4d under the first steps [D]");
        assert_eq!(sim.drones.mission.states[17][0], 3, "ch 0x4d reveals 'Gain access to lobby' [L record 4a]");
    }
    w.skip_cutscenes = false;

    // ② The ride with no input down to the STOP: the key line 0.0303 u/frame [D keys t90→t5040;
    //    L the live rate], the descent plates firing [D triggers], until the platform freezes at
    //    the −52 landing (root −54.1971 [L]). A rider off the line by > 5 before the stop is the
    //    shaft-door scrape (the fail-first).
    let y_board = w.scene.sim.as_ref().unwrap().bond.pos[1];
    let t0 = w.total_ticks;
    let mut fired: Vec<u32> = Vec::new();
    let mut stop_y: Option<f32> = None;
    let mut stop_t = 0u32;
    let mut frozen = 0u32;
    let mut last_y = f32::NAN;
    let mut scrape: Option<(u32, f32, f32)> = None;
    for k in 0..2400u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        for ch in [0x3du32, 0x17, 0x18, 0x3f, 0x19, 0x45, 0x1a, 0x3e, 0x1b, 0x4f] {
            if sim.drones.channels.on(ch) && !fired.contains(&ch) { fired.push(ch); }
        }
        let y = sim.bond.pos[1];
        let t = w.total_ticks - t0;
        // The key line until the pause [D keys; the ride's own board offset].
        let want = y_board - ((t + 60) as f32 - 90.0).max(0.0) * (150.0 / 4950.0);
        if !sim.disabled && t > 150 && (y - want).abs() > 5.0 && stop_y.is_none() {
            scrape = Some((t, y, want));
            break;
        }
        // The freeze detect: the root stops moving (the hold [D event 3 at t1611]).
        if (y - last_y).abs() < 0.0005 { frozen += 1; } else { frozen = 0; }
        last_y = y;
        if frozen >= 10 && stop_y.is_none() {
            stop_y = Some(y);
            stop_t = t;
            break;
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(fired.contains(&0x17) || fired.contains(&0x3d),
            "the descent plates fire during the ride ([D] floors), got {fired:?}");
        assert!(scrape.is_none(), "the ride holds the key line until the stop — left it at {:?}",
            scrape.map(|(t, y, want)| format!("t{t} y {y:.2} want {want:.2}")));
        let _ = &scrape;
        let sy = stop_y.expect("the lift reaches its hold (the pause event 3 [D 0x06000571 s4])");
        assert!((sy - (-54.1971)).abs() < 0.6,
            "the stop height = the live −54.1971 [L fresh boot], got {sy:.4} after {stop_t} ride ticks (boarded {y_board:.3})");
        assert!(sim.drones.channels.on(0x40), "the lift script's t1610 event 0x0d wrote ch 0x40 — the −52 doors unlock [D s5; M Door_Update]");
    }

    // ③+④ The window (the landing cut-scene, 575 ticks [D], the disable [M case 4]) and the
    // hand-back at the RAW marker key [L]. No input throughout — the scene plays un-skipped.
    let mut disabled_ticks = 0u32;
    let mut handback: Option<[f32; 3]> = None;
    for _ in 0..1600u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.disabled { disabled_ticks += 1; }
        else if disabled_ticks > 100 {
            // The first active tick after the disable window: the enable's own placement.
            handback = Some(sim.bond.pos);
            break;
        }
    }
    {
        let p = handback.expect("the landing scene ends with the Player_Enable hand-back [M 0x1da880:46-49]");
        assert!((disabled_ticks as i32 - 575).abs() < 120,
            "the window = the scene's 575-tick disable [D len; L ~560], got {disabled_ticks}");
        assert!((p[0] - 1.7994).abs() < 0.08 && (p[2] - (-3.4534)).abs() < 0.08,
            "the hand-back x/z = the marker key's RAW x/z [L fresh boot], got {p:?}");
        assert!(p[1] > -55.6 && p[1] < -53.6,
            "the hand-back y at the key line (−54.128 [D key 1409] + the first fall ticks), got {p:?}");
        assert!(sim_health(&w) > 0.0, "Bond is alive at the hand-back");
    }

    // ⑥ The walk off: north off the platform onto the −56 landing, then east toward the
    // unlocked −52 doors #110/#111 — real input only (walker::walk_to drives step_bond).
    w.walk_to(walker::Wp::new(2.8, -55.4, 0.8).r(1.2), "onto the -52 landing");
    w.walk_to(walker::Wp::new(4.3, -55.4, 0.6).r(1.2), "toward the unlocked -52 doors");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(sim.bond.grounded, "Bond stands on the landing after walking off");
        assert!(p[1] < -54.5 && p[1] > -57.5, "the landing floor −56.0 [L probe l3c], got ({:.2},{:.2},{:.2})", p[0], p[1], p[2]);
        assert!(p[0] > 2.4, "the walk reached east of the platform's west edge (x {:.2})", p[0]);
        assert!(sim_health(&w) > 0.0, "the walk-off survives (the shaft kill is below, at −95.8 [L])");
        assert_eq!(sim.drones.mission.states[17][0], 3, "the objective stays revealed through the walk-off [L record 4a]");
        assert!(!sim.drones.channels.on(0x4a),
            "ch 0x4a never fires on 0700004a — the writer is 07000013's spawn touch #153 [D; walk_07000013_live]");
        assert!(sim.drones.channels.on(0x40), "ch 0x40 stays on through the walk-off [D s5]");
    }
}

fn sim_health(w: &Walker) -> f32 { w.scene.sim.as_ref().unwrap().drones.health.health }

/// level-4a-lift-3 SISTER: staying on the lift through the hand-back ends in the shaft death,
/// like the game [L fresh boot, god mode, no input: the re-attach at the marker key's x/z, then
/// the death at root −95.80 — health 100 → 0 in ONE frame, mission fail]. No input anywhere
/// after the board; the same real path as the walk above through the enable.
#[test]
fn staying_on_the_lift_ends_in_the_shaft_death_like_the_game() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700004a.bin", false).unwrap());
    for k in 0..84 {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let target = if k < 42 { [3.2, 0.0, -2.2] } else { [3.2, 0.0, -3.8] };
        let desired = (target[0] - p[0]).atan2(target[2] - p[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        w.tick(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, ..Default::default() }, false);
    }
    w.skip_cutscenes = false;
    // Ride to the stop, through the disable window, to the hand-back (no input).
    let mut handback = false;
    let mut disabled_seen = false;
    for _ in 0..4200u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.disabled { disabled_seen = true; }
        if disabled_seen && !sim.disabled && sim.bond.pos[1] > -60.0 { handback = true; break; }
    }
    assert!(handback, "the hand-back arrives (the enable after the disable window)");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] - 1.7994).abs() < 0.08 && (p[2] - (-3.4534)).abs() < 0.08,
            "the hand-back x/z = the marker key's RAW x/z [L], got {p:?}");
        assert!(sim_health(&w) > 99.0, "Bond boards the hand-back at full health, got {}", sim_health(&w));
    }
    // ⑤ No input: the carry/fall down the shaft to the death [L]. The health must hit 0 with
    // Bond's root at the live −95.80 (± the tick granularity), then the death flow runs.
    let mut death_y: Option<f32> = None;
    for _ in 0..2600u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim_health(&w) <= 0.0 {
            death_y = Some(sim.bond.pos[1]);
            break;
        }
    }
    {
        let dy = death_y.expect("the shaft kill fires (health 100 → 0) [L]");
        assert!((-97.5..=-94.0).contains(&dy),
            "the death y = the live −95.80 [L fresh boot], got {dy:.3}");
        // The death flow: ch 0x62 rises [M 0x190bb0:32] and Mission_Update takes the ch-0x62 arm
        // (internal 3, the red fade, the fail flow to 0x07000048 [M 0x182c60:35-42]).
        let mut failed = false;
        for _ in 0..900u32 {
            w.tick(Default::default(), false);
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.drones.channels.on(0x62) && sim.drones.mission.internal >= 3 { failed = true; break; }
        }
        assert!(failed, "the death channel 0x62 → Mission_Update's ch-0x62 arm [M 0x182c60:21,35-42]");
    }
}
