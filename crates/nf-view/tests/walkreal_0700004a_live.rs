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

use nf_game::player::Input;
use nf_game::world::V3;
use walker::{wrap_pi, Walker};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The aim input toward `target` from Bond's eye — real look/turn only (the satchel test's aim).
fn aim_input(w: &Walker, target: V3) -> Input {
    let sim = w.scene.sim.as_ref().unwrap();
    let b = &sim.bond;
    let eye = [b.pos[0], b.pos[1] + b.eye, b.pos[2]];
    let d = [target[0] - eye[0], target[1] - eye[1], target[2] - eye[2]];
    let want_yaw = d[0].atan2(d[2]);
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
    let want_pitch = (d[1] / len).asin() / (std::f32::consts::FRAC_PI_2);
    let yaw_err = wrap_pi(want_yaw - b.yaw);
    let pitch_err = (want_pitch - b.pitch).clamp(-0.06, 0.06);
    Input { turn: yaw_err.clamp(-0.06, 0.06), look: pitch_err, ..Default::default() }
}

// ROOT CAUSE (fix-final-blockers + fix-final-blockers2 — RESOLVED):
// ① The leaves never touch the rider: at t1615 the d50/d51 leaves span z −0.98..−0.82 and Bond
//    stands at z −3.78 — a 2.8 gap; his capsule push is (0,0,0) walkable=false (probe
//    walkreal_probe_fb5). The moving-set door-solid is NOT the scraper.
// ② The −52 landing starts CUT-SCENE #289 (hash 0x060006cf) at frame 1612 — its events at tick 0:
//    stream 4 kind 0xb (the view model, weapon 0x12), stream 8 kind 0x05 (the drone freeze),
//    stream 9 kind 0x04 (PlayerDisable) [NF_SCRIPTLOG]. The lift script itself only pulses
//    ch 0x40 (event 0xd, tick 1610) and pauses (event 3, tick 1611). The game plays the landing
//    as a NIS — Bond disabled, frozen mid-shaft (the game's Player_CollisionHandler early-returns
//    [M 0x1949a8:24]) while the platform keeps descending.
// ③ THE PORT BUG (fixed in nis.rs): the cut-scene's stream-25 marker (obj_type 0x19, no parent,
//    key (1.80,−54.128,−3.453) at t575) is the event-6 `player_at` source [M 0x1d94c0:125-134 —
//    the snapshot is taken at EVENT time; Script_Free 0x1da880 does NOT re-read the entity]. The
//    port composed the SCRIPT BASE into every unparented stream's keys; the game composes it ONLY
//    at the end of a parent chain (`Concat(SCRIPTINFO, obj+0x90)` sits INSIDE the `parent != -1`
//    branch [M 0x1d9a08:106-133]; cameras get no base at all [M 0x1d9a08:190-200 + 0x1db518]).
//    The NIS keys are WORLD-SPACE: the marker's key IS Bond's freeze pose; #289's camera key[0]
//    (2.889,−7.829,−4.583) is the boarding spot (telephoto scale 0.344 — the shot films the
//    landing from the shaft top). Identity-base SPs hid the wrong base in every earlier cut-scene.
// ④ THE HAND-BACK, RESOLVED (level-4a-lift-4 + this task): master re-enables Bond at the landing
//    scene's RAW marker key (1.79945, −54.12811, −3.45340) — the hash-0 kind-0x19 marker rule in
//    nis.rs (interp case 1; pinned by fix_landing_matrix_live). The raw key IS a standing root ON
//    the held platform (floor −55.16 + cc 1.0347 [M 0x194df0] = −54.13 = the key y; z −3.453 = the
//    platform z centre, x 1.8 on the platform [D o0 raw bbox]) — no fall. The ride-loop scrape
//    check therefore runs EVERY enabled tick with no fall exemption: an ungrounded void fall
//    (the old composed-key hand-back) diverges from the key line in ~30 ticks of free fall and
//    trips it — no vacuous pass.
// ⑤ THE HOLD: the landing cut-scene holds the lift 575 ticks — the scene's hand-back marker sits
//    at its t575 [D 0x060006cf s25], the lift script pauses (event 3) at tick 1611 [D script
//    stream; NF_SCRIPTLOG], and the fresh-boot live audit measured the stop: the lift holds
//    y −54.20 for the landing scene's disable (575 ticks) [L lift-4 audit]. The want-line
//    therefore HOLDS while Bond is disabled (the keyed descent counts ACTIVE ticks only — the
//    rate is unchanged; the line's phase shifts by the disabled ticks).
// ⑥ THE SATCHELS: the two 0xD2 shooters drop timed charges onto this ride — #286 fires on the
//    ch-0x22 rise (the landing scene's end event) and #287 on the ch-0x4f plate [D #286/#287;
//    M Shooter_Update 0x17d0c0], each a 0x3b-family bullet with a 900-tick fuse and a
//    2000-damage blast [M Bullet_init 0x12c5a0:199-211; the Explode_Create arm 0x12dd44]. The
//    walk burns both with the laser watch — 1/tick off the 100-burn counter under the beam
//    [M Bullet_DoTrails 0x12e2f0-0x12e390] — the satchel test's way; its sister pins the
//    no-defuse direction (the blast kills, ch 0x62 ≈ 1377 ticks after the hand-back [L]).
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
    // Board: settle onto the platform (the probe: grounded by t30). The reveal plate #277 is a
    // floor plate the stand-in toucher misses on a pure vertical ride (walkreal-a's documented
    // stand-in misses), so Bond walks the platform's north edge and back within the script's
    // t90 pre-descent window — the real player's first steps.
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
    let y_board = w.scene.sim.as_ref().unwrap().bond.pos[1];

    // ⑥ the laser watch into the hand: the shooters' satchels drop on this ride and the beam
    // burns them [M Bullet_DoTrails 0x12e2f0-0x12e390]. The draw completes in the t90
    // pre-descent window [M 0x1a7ef8:318-319 the loadout tail equips the watch].
    w.tick(Input { switch_to: Some(0x4e), ..Default::default() }, false);
    for _ in 0..120 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().gun.weapon == 0x4e { break; }
    }
    assert_eq!(w.scene.sim.as_ref().unwrap().gun.weapon, 0x4e, "the laser watch 0x4E is drawn [M 0x1a7ef8:318-319]");

    // fix-final-blockers: the −52 landing starts cut-scene #289 (0x060006cf — event 0xb/5/4 at
    // frame 1612: the view model, the drone freeze, PlayerDisable [NF_SCRIPTLOG]) — the game
    // plays the landing as a NIS and re-enables Bond at its end. The walkreal-c scrape was the
    // HARNESS'S OWN SKIP: with skip_cutscenes the skip fast-forwards the cut-scene and the
    // Script_Free enable places Bond at the fast-forwarded matrix (−107.66, mid-shaft) — the
    // walkreal-c "falls to −107.66". The real player WATCHES the landing cut-scene: skip it off.
    w.skip_cutscenes = false;

    // The ride: no movement input while the script carries the platform down (the only inputs
    // are the watch's aim/fire at the landed charges, ⑥). The key line: y(t) =
    // y_board − max(0, t−90−hold)·(150/4950) [D keys; ⑤ the hold]; a rider off the line by > 5
    // (free fall diverges in ~30 ticks) is the shaft-door scrape. The plates fire in descent
    // order [D triggers].
    let t0 = w.total_ticks;
    let mut fired: Vec<u32> = Vec::new();
    let mut scrape: Option<(u32, f32, f32)> = None;
    let mut hold_ticks = 0u32; // ⑤ the landing scene's hold: disabled ticks shift the key line
    let mut t84: Option<u32> = None;
    for k in 0..6800u32 {
        // ⑥ burn any landed charge while the cab descends — the beam is real held fire
        // (1/tick off the 100-burn counter [M 0x12e2f0-0x12e390]; ~100 contact ticks per
        // charge). While the landing scene has Bond disabled there is nothing to aim with.
        let disabled_now = w.scene.sim.as_ref().unwrap().disabled;
        let charge = if disabled_now { None } else {
            w.scene.sim.as_ref().unwrap().drones.bullets.iter()
                .find(|b| b.landed && !b.defused && b.burn > 0).map(|b| b.pos)
        };
        if let Some(cpos) = charge {
            let input = aim_input(&w, cpos);
            w.tick_fire(input);
        } else {
            w.tick(Default::default(), false);
        }
        let sim = w.scene.sim.as_ref().unwrap();
        for ch in [0x3du32, 0x17, 0x18, 0x3f, 0x19, 0x45, 0x1a, 0x3e, 0x1b, 0x4f, 0x84] {
            if sim.drones.channels.on(ch) && !fired.contains(&ch) { fired.push(ch); }
        }
        let t = k + 60; // the board ticks count too
        // The keyed descent [D keys] with the landing hold (⑤): the disabled ticks are NOT
        // active descent — the platform pauses (the script's event 3 [D]; the fresh-boot
        // audit's 575-tick stop [L lift-4]) — so the line's phase shifts by every disabled
        // tick and the descent resumes at the keyed rate after the hand-back.
        let want = (y_board - ((t as f32) - 90.0 - hold_ticks as f32).max(0.0) * (150.0 / 4950.0)).max(y_board - 150.0);
        let y = sim.bond.pos[1];
        if sim.disabled { hold_ticks += 1; }
        if t84.is_none() && sim.drones.channels.on(0x84) { t84 = Some(t); }
        // The scrape check runs every enabled tick (④: the raw-key hand-back grounds Bond ON
        // the platform — the old fall exemption is gone; a void fall diverges in ~30 ticks
        // and trips this check, so the walk cannot pass vacuously).
        if scrape.is_none() && t > 150 && !sim.disabled && (y - want).abs() > 5.0 {
            scrape = Some((t, y, want));
            eprintln!("[walk] the ride left the key line at t{t}: y {y:.2} want {want:.2} grounded={} substate={}",
                sim.bond.grounded, sim.bond.substate);
            break;
        }
        if sim.drones.level_exit.is_some() { break; } // the mission's LevelEnd — the ride is over
        if sim.drones.channels.on(0x4a) { break; }
    }
    let ride_ticks = w.total_ticks - t0;
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // ⑥ at least one landed satchel burned under THIS walk's beam (the game's chain: two
        // shooters, one charge each [M 0x17d0c0's obj+0xFE |= 1]; both plates fire on the ride
        // [D #281/#62] — the run burned 2), and none is left with a live fuse (the sister pins
        // the no-defuse direction — the 900-tick fuse's blast kills and ch 0x62 fails [L]).
        let burned = sim.drones.bullets.iter().filter(|b| b.defused).count();
        assert!(burned >= 1,
            "a landed satchel burned under the watch's beam [M 0x12e2f0; D #286] — burned {burned}, fired {fired:?}");
        assert!(!sim.drones.bullets.iter().any(|b| b.landed && !b.defused),
            "every landed satchel burned under the watch's beam [M 0x12e2f0] — got {:?}",
            sim.drones.bullets.iter().filter(|b| b.landed)
                .map(|b| (b.burn, b.defused)).collect::<Vec<_>>());
        assert!(fired.contains(&0x17) || fired.contains(&0x3d),
            "the descent plates fire during the ride ([D] floors), got {fired:?}");
        assert!(scrape.is_none(),
            "the ride holds the script's key line (no shaft-door scrape) — left it at t{}, y {:.2} want {:.2}, fired {fired:?}",
            scrape.map(|s| s.0).unwrap_or(0), scrape.map(|s| s.1).unwrap_or(0.0), scrape.map(|s| s.2).unwrap_or(0.0));
        assert!(!sim.drones.channels.on(0x4a),
            "ch 0x4a never fires on 0700004a — the writer is 07000013's spawn touch #153 [D; walk_07000013_live] — rode {ride_ticks} ticks, y {:.2} (boarded {y_board}), fired {fired:?}",
            sim.bond.pos[1]);
        assert_eq!(sim.drones.mission.states[17][0], 3, "the objective stays revealed through the ride's end [L record 4a]");
        assert!(sim.bond.pos[1] < y_board - 140.0, "the ride covered the shaft (y {:.2} from {y_board})", sim.bond.pos[1]);
        // The ride's end IS the level's end: the bottom plate #62 fires ch 0x84 [D #62] and the
        // mission completes (`Event::LevelEnd` → `ResetMap_LevelToLoad(LevelToEndTo)`
        // [M 0x182c60:73]; the run: 0x84 at t5488, the exit ~800 ticks later under the banner).
        assert!(t84.is_some(), "the shaft's bottom plate #62 fires ch 0x84 [D #62] — fired {fired:?}");
        assert!(sim.drones.level_exit.is_some(), "the mission's LevelEnd follows the bottom plate — the ride reaches the lobby's level end");
    }

    // The walk out (the route's historical tail): with the ride's end the level has already
    // exited (the bottom plate #62's ch 0x84 → the mission's LevelEnd), so these legs are the
    // walker's stop_on_exit no-ops — kept so a regression that un-fires the exit makes the walk
    // actually run (and its own waypoints/arrival checks apply).
    w.walk_to(walker::Wp::new(3.0, y_board - 149.5, -2.5).r(1.0), "off the platform");
    w.walk_to(walker::Wp::new(3.0, y_board - 149.5, 2.0).r(1.2), "into the lobby");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.mission.states[17][0] != 0, "the objective holds through the walk-out");
    }
}
