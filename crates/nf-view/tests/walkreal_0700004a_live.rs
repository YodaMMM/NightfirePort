//! walkreal-c: 0700004a (the Tower-2 elevator) walked with the player's own movement — no
//! `bond.pos` writes after the spawn — from the boarding through BOTH cut-scenes and out the
//! way the game exits. The route (docs/levels/0700004a.md §3-§7; the disc scripts):
//!
//! 1. The arrival spawns ON the lift platform (4.26, −7.71, −4.76 [D #1]); the reveal plate
//!    #277 fires ch 0x4d under the first steps [D].
//! 2. SP #42 "SCRIPTPLAYER_Lift" (mode 1, out 0x22 [D placement]) plays 0x06000571: the cab
//!    entity (0x20005b4) descends 150 units from key t90 to key t5040 [D keys], its sound 0x28c
//!    riding along. At t1610 its stream-5 event 0xD raises ch 0x40 [D]; at t1611 its stream-4
//!    event 3 (p0 = 0 → the negative delay) HOLDS the cab [D 06000571; M 0x1da998:17-18,112-114].
//! 3. ch 0x40 starts SP #289 (out 0x40 [D placement]) — the landing cut-scene 0x060006cf (575
//!    ticks [D length], pack 0780004a.bin): tick-0 events 0xB (the view model), 0x05 (the drone
//!    freeze) and 0x04 (PlayerDisable) [D]. Bond freezes ON the held cab.
//! 4. The scene's last tick: stream 26's event 6 snapshots the previous stream's object matrix
//!    into SCRIPTINFO+0x40 [M 0x1d94c0:125-134; Mat_Copy(src,dst) 0x1e3348] — stream 25's ot25
//!    marker, created at t575 by `control_create_object(0,0,0,0)` [M disasm 0x1d8d24] whose
//!    zero-key `Script_GetInterp` collapses onto key-pool entry 1409 = (1.80, −54.128, −3.453),
//!    Bond's freeze pose, world-space (no parent → the base Concat never runs [M 0x1d9a08:
//!    114-135]) — and stream 27's event 0xD [34, 1] raises ch 0x22 [D 060006cf].
//! 5. `Script_Free` re-enables Bond AT the snapshot [M 0x1da880:44-47, Player_Enable 0x195498 —
//!    whose mode-1 SphereIntersect only dispatches wire/creepwall, no push-out [M 0x195498:
//!    90-111]] — he stands on the still-held cab, and ch 0x22 releases the lift through SP
//!    #42's mask-1 UnPause [M 0x1dbda8:124-128]. The descent continues to the lobby.
//! 6. The parked cab sits inside the lobby TouchOnce #275 (0.89..5.17, −157.32..−155.10,
//!    −5.71..−0.99 [D]) whose ch 0x84 starts SP #276 — the arrival cut-scene 0x060006d3 (800
//!    ticks, pack 0790004a.bin; tick-0 PlayerDisable again). Its last tick: stream 4's event 6
//!    [0] snapshots its stream-3 BODY ANIM's pose (an anim object's +0x90 is its discrete frame
//!    [M 0x1da158 case 2]) and stream 9's event 0x12 calls `ResetMap_LevelToLoad(0x0720004a)`
//!    [D 060006d3 streams 4/9] — the movie code resolving 0x0720004a → 0x07100013 → 07000013
//!    [M 0x1d3560:52-96; nf_game::triggers::resolve_exit_code].
//! 7. 4a's objective "Gain access to lobby" completes on 07000013's SPAWN: plate #153 fires
//!    ch 0x4A there and the mission monitor's enable(0x4d)+complete(0x4a) AND folds the row
//!    [M 0x1827d8; D 07000013 trigger #153; docs/levels/07000013.md §2] — the late AND, the
//!    same shape as 12's objective completing on 4a's plate #277. No 0700004a-resident writer
//!    exists ([D] the triggers, the scripts, both packs).
//!
//! The scrape check models the hold: between ch 0x40 and ch 0x22 the script clock freezes, so
//! the key line pauses; a rider off the line by > 5 outside the disable window is the shaft-door
//! scrape (the fail-first for the door fix). Opt-in via NF_ISO (the slim ISO is enough).

#[path = "common/walker.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn ride_the_lift_through_both_cutscenes_and_exit_to_07000013() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700004a.bin", false).unwrap());
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] > 2.0 && p[0] < 6.0 && p[1] > -9.0 && p[1] < -6.0 && p[2] > -6.0 && p[2] < -3.0,
            "the arrival spawns at the shaft's top, on the platform (#1 [D]), got {p:?}");
    }
    // Board: settle onto the platform (grounded by t30). The reveal plate #277 is a floor plate
    // the stand-in toucher misses on a pure vertical ride (walkreal-a's documented stand-in
    // misses), so Bond walks the platform's north edge and back within the script's t90
    // pre-descent window — the real player's first steps.
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

    // The ride: no input while the script carries the platform down. The landing cut-scene and
    // the arrival cut-scene PLAY (skip off — the real player watches; a skip fast-forwards the
    // enable snapshot and the exit event, the walkreal-c "falls to −107.66" harness artifact).
    w.skip_cutscenes = false;
    let t0 = w.total_ticks;
    let mut fired: Vec<u32> = Vec::new();
    let mut scrape: Option<(u32, f32, f32)> = None;
    let mut freeze_start: Option<u32> = None;
    let mut ch22_tick: Option<u32> = None;
    let mut disabled_ticks = 0u32;
    let mut reground = 0u32;
    let mut fall_exempt = false;
    for k in 0..6600u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        for ch in [0x3du32, 0x17, 0x18, 0x3f, 0x19, 0x45, 0x1a, 0x3e, 0x1b, 0x4f, 0x40, 0x22, 0x84] {
            if sim.drones.channels.on(ch) && !fired.contains(&ch) { fired.push(ch); }
        }
        // The landing scene's own window: its tick-0 event 4 disables Bond [D 060006cf stream 9]
        // and its last tick raises ch 0x22 [D stream 27]. (ch 0x40 itself is shared with a
        // top-lobby door's open channel — it rises long before t1610 — so the freeze tracks the
        // DISABLE, not the channel.)
        if sim.disabled && freeze_start.is_none() && ch22_tick.is_none() { freeze_start = Some(k); }
        if ch22_tick.is_none() && sim.drones.channels.on(0x22) { ch22_tick = Some(k); }
        if sim.disabled { disabled_ticks += 1; }
        let t = k + 84; // the board ticks count too
        // The script clock freezes from the landing scene's first tick until its ch 0x22: the cab
        // holds Bond's freeze pose through the scene [D 06000571 ev03[0] + 060006cf ev0d[34,1];
        // M 0x1dbda8:124-128].
        let hold = match (freeze_start, ch22_tick) {
            (Some(a), Some(b)) => (b - a) as f32,
            (Some(a), None) => (k - a) as f32, // still holding: the clock is frozen now
            _ => 0.0,
        };
        let want = (y_board - ((t as f32) - 90.0 - hold).max(0.0) * (150.0 / 4950.0)).max(y_board - 150.0);
        let y = sim.bond.pos[1];
        // While a cut-scene has Bond disabled he legitimately freezes (the game's
        // Player_CollisionHandler early-returns [M 0x1949a8:24]); the check resumes once he is
        // riding again (5 straight grounded ticks).
        if sim.disabled {
            fall_exempt = true;
            reground = 0;
        } else if fall_exempt {
            if sim.bond.grounded { reground += 1; if reground >= 5 { fall_exempt = false; } } else { reground = 0; }
        }
        if scrape.is_none() && t > 150 && !sim.disabled && !fall_exempt && (y - want).abs() > 5.0 {
            scrape = Some((t, y, want));
            eprintln!("[walk] the ride left the key line at t{t}: y {y:.2} want {want:.2} grounded={} substate={}",
                sim.bond.grounded, sim.bond.substate);
            break;
        }
        if sim.drones.channels.on(0x84) { break; }
    }
    let ride_ticks = w.total_ticks - t0;
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let a = freeze_start.expect("the landing scene disabled Bond (its tick-0 event 4 [D 060006cf stream 9])");
        let b = ch22_tick.expect("ch 0x22 fired — the landing scene's end [D 060006cf stream 27]");
        assert!(fired.contains(&0x17) || fired.contains(&0x3d),
            "the descent plates fire during the ride ([D] floors), got {fired:?}");
        assert!(fired.contains(&0x40), "the lift's t1610 event 0xD raised ch 0x40 — #289's start channel [D 06000571 stream 5]");
        let hold = b - a;
        assert!((573..=580).contains(&hold),
            "the cab holds through the landing scene: disable→ch 0x22 gap {hold} ≈ #289's 575-tick length [D 060006cf length]");
        assert!(disabled_ticks >= hold - 4,
            "Bond stays disabled the whole landing scene (its tick-0 event 4 [D 060006cf stream 9]) — {disabled_ticks} of {hold}");
        assert!(scrape.is_none(),
            "the ride holds the script's key line through the hold and both scenes (no shaft-door scrape) — left it at t{}, y {:.2} want {:.2}, fired {fired:?}",
            scrape.map(|s| s.0).unwrap_or(0), scrape.map(|s| s.1).unwrap_or(0.0), scrape.map(|s| s.2).unwrap_or(0.0));
        assert!(sim.drones.channels.on(0x84),
            "the parked cab sits inside the lobby touch #275 → ch 0x84 [D trigger #275] — rode {ride_ticks} ticks, y {:.2} (boarded {y_board}), fired {fired:?}",
            sim.bond.pos[1]);
        assert!(sim.bond.pos[1] < y_board - 140.0, "the ride covered the shaft (y {:.2} from {y_board})", sim.bond.pos[1]);
        // Sister: the objective-complete channel never fires on this level — the completion is
        // 07000013's late AND (plate #153 [M 0x1827d8; D #153]).
        assert!(!sim.drones.channels.on(0x4a), "ch 0x4a has no writer on 0700004a [D scans incl. both packs]");
    }

    // The arrival scene 060006d3 (started by ch 0x84): 800 ticks [D length], Bond disabled by its
    // tick-0 event 4, ending with the event-6 [0] body-anim snapshot and the event 0x12 exit.
    let mut arrival_disabled = false;
    for _ in 0..1400u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().disabled { arrival_disabled = true; }
        if w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { break; }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(arrival_disabled, "the arrival scene disabled Bond (060006d3's tick-0 event 4 [D stream 9])");
        assert_eq!(sim.drones.level_exit, Some(0x0720_004a),
            "the arrival scene's last-tick event 0x12 exits to the 0x0720004a movie code [D 060006d3 stream 9 ev12]");
        assert_eq!(nf_game::triggers::resolve_exit_code(0x0720_004a), 0x0700_0013,
            "the movie code resolves to 07000013 (0x0720004a → 0x07100013 → masked) [M 0x1d3560:52-96]");
    }

    // The exit the way the game does: the RamSave + mission carry through the load into
    // 07000013 (the movie plays between them in the game — the main agent's draw scope).
    let save = {
        let sim = w.scene.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request,
            &sim.drones.inventory, &sim.drones.channels).with_mission(&sim.drones.mission)
    };
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000013.bin", false, Some(&save)).unwrap());
    w.wait_enabled(3000, "the 013 intro (SP #65, mode 1 [D])");
    // The spawn tile's TouchOnce #153 fires ch 0x4A [D] — 4a's objective-complete: the monitor's
    // AND with the carried 0x4d folds "Gain access to lobby" on the NEXT level, the late AND
    // [M 0x1827d8].
    for _ in 0..60 { w.tick(nf_game::player::Input::default(), false); }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!(p[0] > 0.0 && p[0] < 4.5 && p[1] > -371.0 && p[1] < -367.0 && p[2] > 8.0 && p[2] < 13.0,
            "07000013's start #9 spawns at the tower base lobby [D], got {p:?}");
        assert!(sim.drones.channels.on(0x4a), "the spawn plate #153 fired ch 0x4A [D 07000013 trigger #153]");
        assert_eq!(sim.drones.mission.states[17][0], 4,
            "4a's 'Gain access to lobby' completes through the late AND on 07000013's spawn [M 0x1827d8]");
        // 12's "Escape through the elevator shaft" (row 1, complete ch 0x4d) folded HERE too —
        // its writer is 4a's plate #277, which this walk's spawn step fired [D census §2].
        // (Row 0 stays 3 in a FRESH 4a load: its complete ch 0x48 is 07000012's own writer,
        // only ON through the 12→4a carry that walkreal_07000012 pins.)
        assert_eq!(sim.drones.mission.states[16][1], 4,
            "12's 'Escape through the elevator shaft' completed through 4a's plate #277 (the late AND) [M 0x1827d8]");
    }
}
