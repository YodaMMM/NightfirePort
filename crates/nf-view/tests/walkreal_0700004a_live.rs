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
#[ignore = "level-4a-lift-2: the HAND-BACK POSITION is settled this round, every link verified in the disasm/decompile: SP_LoadScript passes the SP object's +0x30/+0x50 into Script_Load [M 0x1dbc08:37-40] which builds SCRIPTINFO+0x00 = View_RotTransMatrix(rot, pos) [M 0x1d8248:148] = SP #289's placement (0,-53.534,-3.377) rot 0 [D]; Script_Interp's case-1 GetInterp call passes the 9th (stack char) arg = (stream parent == -1) [M disasm 0x1d9a7c-0x1d9a98: lb 0x25/nor/sltiu/sw ($sp); the ABI is f12,a0,a1,a2,a3,t0,t1,t2,t3,0(sp)] and Script_GetInterp's tail runs Concat(SCRIPTINFO+0x00, out) when that flag is non-zero [M 0x1da158 tail] — so the UNPARENTED marker (s25, parent None [D]) composes the base, obj+0x90 = (1.799,-107.662,-6.830); the parented path composes it once at the chain end [M 0x1d9a08:114-135]; the event-6 snapshot reads the PREVIOUS stream's object (the handler's stream base +0x80 vs Script_Run's +0xa8 = stream[i-1], kind gate 1/2, stream->0x10 [M 0x1d94c0 case 6 + 0x1d8548], default Bond's own glb_players+0x90 when none) into SCRIPTINFO+0x40; Script_Free calls Player_Enable(glb_players, +0x40, +0xab6) on the +0xaa8 bit 0x100 [M 0x1da880:46-49]; Player_Enable applies the matrix translation VERBATIM (obj+0x30 <- m+0x30, copied to +0x40; yaw/pitch from Mat_GetDir; the velocity vectors zeroed; substate 2; the param-3 sphere scan handles only wire/creepwall cels — NO ground snap) [M 0x195498:54-89]. The port does exactly this (the sister below pins it through the real path with no re-seats). THE BLOCKER is the POST-enable world: the composed spot floats in OPEN AIR — the disc's collision has NO floor in the column x 1.8/z -6.83 (floor_below -104..-507 None [L probes l2b/l2d; the bottom -156 slab ends at z -1..-1.5]; no walls within 30 horizontally, the walkway edge 5.5 north), so the port's Bond falls forever (probe l2: airborne from t2188, y -989 at t3501, hp 100) and the shipped game's catch is unknown — the fresh-boot PCSX2 live check decides (gdbwatch glb_players+0x30 x/y/z + health + BLData+0x94a substate through t2190..2800 after the -52 landing scene: does the fall end on the resumed cab, the -108 walkway, a void recovery, or death?), and the scrape/fall-exempt logic must be reworked around whatever that shows before the ride -> landing -> lobby walk can be pinned"]
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

/// level-4a-lift-2 SISTER: the hand-back position through the REAL path — board with movement
/// input, ride with none, the landing scene plays un-skipped — and the first post-enable tick
/// places Bond at the composed marker matrix (1.799,−107.662,−6.830) [M: SP base 0x1dbc08:37-40 +
/// 0x1d8248:148 from the SP #289 placement (0,−53.534,−3.377) [D]; the unparented marker's base
/// compose inside Script_GetInterp, disasm 0x1d9a7c-0x1d9a98 + the tail Concat 0x1da158; the
/// prev-stream snapshot 0x1d94c0 case 6 + 0x1d8548; the verbatim apply 0x195498:54-72]. No
/// re-seats anywhere (fix_landing_matrix_live re-seats the ride; this one does not). The second
/// half pins the column's open air [L probes l2b/l2d]: floor_below −507 from the hand-back spot
/// is None and the bottom −156 slab ends at z −1..−1.5 — the disc's own collision, which any
/// future "catch" (a recovery script, a moving solid, a room-linked cel) must explain against.
#[test]
fn the_handback_lands_bond_at_the_composed_marker_and_the_column_is_open_air() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "0700004a.bin", false).unwrap());
    // Board: the walkreal's own settle steps (real movement input, the reveal plate underfoot).
    for k in 0..84 {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        let target = if k < 42 { [3.2, 0.0, -2.2] } else { [3.2, 0.0, -3.8] };
        let desired = (target[0] - p[0]).atan2(target[2] - p[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        w.tick(nf_game::player::Input { turn: err.clamp(-0.08, 0.08), forward: 1.0, ..Default::default() }, false);
    }
    assert!(w.scene.sim.as_ref().unwrap().bond.grounded, "Bond boards the platform");
    w.skip_cutscenes = false;
    // Ride with no input until the landing scene's end enables Bond (ch 0x22 rises the same tick,
    // [D 0780004a.bin 0x060006cf s26/s27 at t575]).
    let mut handback: Option<[f32; 3]> = None;
    for _ in 0..3000u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.channels.on(0x22) && !sim.disabled && handback.is_none() {
            handback = Some(sim.bond.pos);
            break;
        }
    }
    let p = handback.expect("the enable lands within the ride window (ch 0x22 at the scene's end)");
    assert!((p[0] - 1.799).abs() < 0.05 && (p[1] - (-107.662)).abs() < 0.5 && (p[2] - (-6.830)).abs() < 0.05,
        "the hand-back = SP #289's base composed onto the unparented marker's key [M chain], got {p:?}");
    // The column's open air [L]: nothing below within 400 units; the walkway edge is 5.5 north.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.world.floor_below([1.8, -107.0, -6.83], 400.0).is_none(),
            "the composed hand-back column is open air in the disc's collision [L l2d]");
        assert!(sim.world.floor_below([-4.34, -102.0, -0.58], 8.0).is_some(),
            "the guards' own column (x -4.34, z -0.58) has the -104 walkway [L l2b] — the floors exist, 6 units north of the hand-back");
    }
}
