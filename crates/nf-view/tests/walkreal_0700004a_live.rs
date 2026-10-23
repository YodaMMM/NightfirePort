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
// ④ THE DESIGN: the enable (t2187, mode 1) puts Bond back at his freeze pose — 17.5 ABOVE the
//    platform that kept descending — and he FALLS ONTO IT (probe walkreal_probe_fb6i: airborne
//    t2188..2312, grounded t2313 at y −75.50, 0.4 below the key line, riding on to the lobby).
//    The ride-loop scrape check exempts the disabled window plus the fall (until 5 grounded ticks).
#[test]
#[ignore = "level-4a-lift-landing: the POST-enable handback is the blocker, now narrowed to one question. NEW EVIDENCE (2026-10-23): (a) the old fall-exempt hole let the walk PASS VACUOUSLY - Bond re-enabled at the composed marker (1.799,-107.662,-6.830) fell past the platform (its z span -5.58..-1.30 [D o0 raw bbox] misses z -6.83) into the void (y -388 at t2698 [L probe fb6 rerun], -2571 in the walk) while the exemption ran ungrounded forever; the exemption is now TIME-CAPPED (300 ticks) so the void fall trips the scrape check. (b) [D dump_060006cf_marker_stream_raw] the marker stream s25's RAW key at t575 = (1.79945, -54.12811, -3.45340), quat yaw ~15 deg - a STANDING ROOT on the HELD platform: the platform floor -55.16 [L fb6: Bond grounded -54.13 root, platform row3 -46.12, floor local -9.04] + cc 1.0347 [M 0x194df0] = -54.13 = the key y; z -3.453 = the platform z centre (-5.58..-1.30 -> -3.44); x 1.8 on the platform (0.89..5.21). (c) [M disasm 0x1d9a7c-0x1d9a98 (case 1) + 0x1d9be4-0x1d9c0c (case 6)] Script_Interp's GetInterp base-concat flag = (stream+0x25 parent == -1) sltiu/nor pair - the base composes EXACTLY ONCE either way (unparented: inside GetInterp [M 0x1da158 tail]; parented: at the chain end [M 0x1d9a08:133/:270]) - the port's interp rule is CORRECT. (d) [D all-map scan] the composed pose (1.799,-107.662,-6.830) is a VOID in all 42 maps' collision (only the LiftShaft shells z -6.0..-0.75 exist at that y; nothing at z < -6.0 below y -90). CONTRADICTION: composing the SP #289 base (0,-53.534,-3.377 [D placement]) onto the world-space key doubles the landing level; the raw key is the designer's handback ON the platform. THE OPEN QUESTION: does the game's SP #289 SCRIPTINFO+0x00 carry the placement translation when the marker interp runs (the port's assumption [Script_Load 0x1d8248 reads the SP obj+0x30/0x50])? If it reads identity/near-zero the handback = the raw key and the port's SP base seeding is the bug. Live check: fieldprobe the 060006cf SCRIPTINFO+0x00 row3 at t2180..2190 + gdbwatch Bond's y t2190..2500. The walk stays ignored until the handback is settled; then the want-line must shift by the hold ticks (575) after the resume."]
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

    // fix-final-blockers: the −52 landing starts cut-scene #289 (0x060006cf — event 0xb/5/4 at
    // frame 1612: the view model, the drone freeze, PlayerDisable [NF_SCRIPTLOG]) — the game
    // plays the landing as a NIS and re-enables Bond at its end. The walkreal-c scrape was the
    // HARNESS'S OWN SKIP: with skip_cutscenes the skip fast-forwards the cut-scene and the
    // Script_Free enable places Bond at the fast-forwarded matrix (−107.66, mid-shaft) — the
    // walkreal-c "falls to −107.66". The real player WATCHES the landing cut-scene: skip it off.
    w.skip_cutscenes = false;

    // The ride: no input while the script carries the platform down. The key line: y(t) =
    // y_board − max(0, t−90)·(150/4950) [D keys]; a rider off the line by > 5 (free fall diverges
    // in ~30 ticks) is the shaft-door scrape. The plates fire in descent order [D triggers].
    let t0 = w.total_ticks;
    let mut fired: Vec<u32> = Vec::new();
    let mut scrape: Option<(u32, f32, f32)> = None;
    let mut reground = 0u32;
    let mut fall_exempt = false;
    let mut fall_exempt_ticks = 0u32;
    for k in 0..5400u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        for ch in [0x3du32, 0x17, 0x18, 0x3f, 0x19, 0x45, 0x1a, 0x3e, 0x1b, 0x4f, 0x84] {
            if sim.drones.channels.on(ch) && !fired.contains(&ch) { fired.push(ch); }
        }
        let t = k + 60; // the board ticks count too
        let want = (y_board - ((t as f32) - 90.0).max(0.0) * (150.0 / 4950.0)).max(y_board - 150.0);
        let y = sim.bond.pos[1];
        // While the landing cut-scene has Bond disabled he legitimately freezes (the game's
        // Player_CollisionHandler early-returns [M 0x1949a8:24]) — and the enable drops him at
        // his freeze pose to fall onto the still-descending cab (the header ④). The scrape check
        // resumes only once he is riding again (5 straight grounded ticks). The exemption is
        // TIME-CAPPED at 300 ticks: the game's re-enable must ground him (the ④ fall took ~125
        // ticks); an endless ungrounded fall is the void handback — the check trips (this was
        // the vacuous pass: the old exemption ran while ungrounded forever and the walk
        // "arrived" at y −2571 in free fall).
        if sim.disabled {
            fall_exempt = true;
            fall_exempt_ticks = 0;
            reground = 0;
        } else if fall_exempt {
            fall_exempt_ticks += 1;
            if sim.bond.grounded {
                reground += 1;
                if reground >= 5 { fall_exempt = false; }
            } else {
                reground = 0;
                if fall_exempt_ticks > 300 { fall_exempt = false; }
            }
        }
        if scrape.is_none() && t > 150 && !sim.disabled && !fall_exempt && (y - want).abs() > 5.0 {
            scrape = Some((t, y, want));
            eprintln!("[walk] the ride left the key line at t{t}: y {y:.2} want {want:.2} grounded={} substate={}",
                sim.bond.grounded, sim.bond.substate);
            break;
        }
        if sim.drones.channels.on(0x4a) { break; }
    }
    let ride_ticks = w.total_ticks - t0;
    {
        let sim = w.scene.sim.as_ref().unwrap();
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
    }

    // The walk out: north off the platform into the bottom lobby (the lobby strip at the shaft's
    // south edge; the lobby doors at z 0 are the mission's "access").
    w.walk_to(walker::Wp::new(3.0, y_board - 149.5, -2.5).r(1.0), "off the platform");
    w.walk_to(walker::Wp::new(3.0, y_board - 149.5, 2.0).r(1.2), "into the lobby");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.mission.states[17][0] != 0, "the objective holds through the walk-out");
    }
}
