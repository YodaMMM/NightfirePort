//! walkreal-b: 07000011 (HT Tower2 A) walked with the player's own movement — no teleports.
//! Route (docs/levels/07000011.md + this task's surface scan): the spawn #146 (−0.40, 46.97, 25.83
//! [D]; the TouchOnce #282 spawn tile → ch 0x7E) → east across the top office floor to the
//! stairwell door (13.8, 46.8, 7.1 [D]) → E-opened — the doorway's clear band is its SOUTH half
//! (the leaf's hinge sits on the north edge z 7.1; the swung leaf blocks z 7.0-7.2 [D probe]) →
//! the stairwell's top landing (15.1, 6.4) → the stairwell DESCENT WALKED: the ring of flights
//! around the central well (the full chain in leg 2 below; the surface scan [D probe] resolved
//! what the old point-probe pass could not) → the lower floor (15.0) → the Load Point #158
//! (14.58, 16.41, −3.94 [D]) → E → level_exit 0x07000012 ([M Trigger_Activate 0x1b2318 — the E
//! arm falls through with the gate off]).
//!
//! THE EXTENSION (level-11-every-objective-3): after the 07000012 spawn stand the walk CONTINUES
//! on real input through every remaining objective of mission 0x07000011 to the mission's end —
//! the next level's load: the override console E (#731 → ch 0x49 completes 0x04000037), the lift
//! call, the shaft entrance (#683 → ch 0x1F), the gated exit #669 auto-firing into 0700004A
//! ([M Trigger_Update 0x1b1a78 case 10]), then the carried load's spawn drop into 0700004A's
//! touch #277 (ch 0x4D completes 0x04000038 — its writer lives one level ahead [D]). The route on
//! 07000012 is tower2's, shared with walkreal_07000012_live; its TWO documented stand-ins (the
//! door[19] corner pincer and the north-wall funnel — both measured, probes 12/12b/12c/12d, the
//! game passes them on the same key data) are carried over with it and listed in the report's
//! live checks. Everything else on both levels is walked input.
//!
//! THE MISSION ROWS ON THE WALK [D c07000011.rs; M 0x1827d8's machine]:
//! - 0x04000035 "Escape from the tower" (ch 0x64 = 100, the mission-success AND [M 0x1827d8:119-123])
//!   announces at load and completes only on 07000013, where the AND over the whole mission first
//!   fires [D docs/levels/07000011.md §2] — asserted announced (3) here, never complete.
//! - 0x04000036 "Gain access to lower office floor" (ch 0x48) announces at load; the writer hunt
//!   (level-11-every-objective-2, docs/reports/level-11-every-objective-2.md) found no writer on
//!   07000011 — that absence is [S] unverified (an assertion may not rest on a missing writer;
//!   the port may simply lack it), so the walk does NOT assert it: it logs the channel each leg
//!   and asserts the [M]-backed monitor states instead. The writer on 07000012 is TouchOnce
//!   #679 [D]: the spawn stands inside its tile (21.48, 0.23, 18.64)–(23.59, 1.21, 19.62 [D])
//!   and ch 0x48 fires under Bond's feet — the record asserts COMPLETE (state 4) there
//!   [M 0x1827d8 case 3].

#[path = "common/walker_b2.rs"]
mod walker;
#[path = "common/tower2.rs"]
mod tower2;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn walk_spawn_to_the_lower_floor_load_point() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000011.bin", false, None).expect("load"));
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let p = sim.bond.pos;
        assert!((p[0] + 0.40).abs() < 1.5 && (p[1] - 46.97).abs() < 2.0 && (p[2] - 25.83).abs() < 1.5,
            "the spawn #146 [D], got {p:?}");
    }
    w.wait_enabled(3000, "the intro");
    w.wait(120);
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x7e),
        "the spawn tile TouchOnce #282 fires ch 0x7E [D]");
    // The mission rows after the first monitor ticks [D c07000011.rs; M 0x182270 init + 0x1827d8's
    // case-0 announce (record.level <= GameState → state 3; both records' enable is 0)]:
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission.states[15][0], 3,
            "obj 0x04000035 'Escape from the tower' announced at load [D c07000011.rs; M 0x1827d8:41-54]");
        assert_eq!(sim.drones.mission.states[15][1], 3,
            "obj 0x04000036 'Gain access to lower office floor' announced at load [D c07000011.rs; M 0x1827d8:41-54]");
    }
    // The stairwell's guards (0x11/0x13/0x14/0x36/0x37 wake on the flight plates [D §5]) shoot
    // during the descent; the walk pins the route, not the firefight — the AI runs for real.
    w.top_up_health = true;

    // Leg 1: east across the top floor to the stairwell door (the nav web [D paths 0/1] has no
    // cross-floor route — the direct chain), then through the doorway's south band — walked.
    w.stop_on_exit = false;
    w.follow_nav_route([12.7, 47.4, 6.4], "across the top floor to the stairwell door");
    w.walk_to(Wp::new(16.0, 47.4, 6.4).r(0.9), "through the doorway (the south band)");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[011] post-leg1: exit {:?} ch100 {} ch62 {} ch61 {} ch60 {} ch63 {} pos {:?}",
            sim.drones.level_exit, sim.drones.channels.on(100), sim.drones.channels.on(0x62),
            sim.drones.channels.on(0x61), sim.drones.channels.on(0x60), sim.drones.channels.on(0x63), sim.bond.pos);
        assert!(!sim.drones.channels.on(0x62), "no death on the walked top floor");
    }
    // Leg 2 — THE STAIRWELL DESCENT, WALKED (the old teleport stand-in is gone). The stairwell is
    // a ring of flights around a central well (x 17.5-21, z −1..3 stays open at every level [D
    // probe]); the descent walks its rim, alternating long flights (the west strip x 14.2-16.6,
    // z −3.5..5.5) with the quarter-turn connectors [D surface scan, this task]:
    //   office floor (the seam slopes onto the flight at x 10.5-13, z −4.2..−5)
    //   → north flight (z −5.5..−4.5, x 12.5→19: 46.8→44.4) → NE landing 43.1 (x 21-24, z −5..−2)
    //   → east flight (x 22-24, z −3→4.4: 43.1→39.1) → SE landing 39.1 (x 19-25, z 4.4-6.4)
    //   → south run (z 6.0-6.4, x 21→14.6: 39.1→35.2) → SW landing 35.2 (x 14.2-16.6, z 4.4-6.5)
    //   → west flight C (x 14.2-16.6, z 5.5→−2.5: 35.2→31.1) → north strip 31.1 (x 14-23)
    //   → east connector (x 23, z −3→4.4: 31.1→27.1) → south strip 27.1 (x 14.6-23, z 3.5-5.5)
    //   → west flight B (27.1→23.0) → north strip → east connector (23.0→18.9) → south strip
    //   → west flight A (18.9→15.0) → the lower floor (15.0, flat to the Load Point).
    let band = |w: &Walker, lo: f32, hi: f32, what: &str| {
        let y = w.scene.sim.as_ref().unwrap().bond.pos[1];
        assert!(y > lo && y < hi, "{what}: y {y} not in ({lo}, {hi}) — the descent left the flights");
    };
    w.walk_to(Wp::new(12.0, 47.4, 6.6).r(0.9), "the ledge west");
    // The well strip (z 1.5-4.5) splits the top level; the crossing is the west bridge at
    // x ≈ 1-1.5 (the office floor is continuous z −2..7 at x 1.0 [D probe]; the nav's own
    // office→balcony link n12 (2.1, 47.2, 6.6) → n45 (2.2, 47.2, −0.3) rides the same bridge
    // [D nav path 1]).
    w.walk_to(Wp::new(2.2, 47.4, 6.6).r(0.9), "the ledge's west end (n12)");
    w.walk_to(Wp::new(1.2, 47.4, 3.0).r(0.6), "the west bridge south");
    w.walk_to(Wp::new(1.4, 47.4, -1.0).r(0.8), "onto the north balcony (n45)");
    w.walk_to(Wp::new(5.0, 47.4, -1.8).r(0.9), "the balcony east (n46/n47)");
    w.walk_to(Wp::new(9.0, 47.4, -2.2).r(0.9), "the north strip east");
    w.walk_to(Wp::new(9.6, 47.4, -4.4).r(0.8), "the strip west of the parapet");
    // The well's parapets at chest height [D ray-fan]: the diagonal W1 (9.2,−5.8)→(12.75,−3.5) and
    // the north-east pair W2 (11.5,−1.9)→(12.7,−3.4) meet at (12.7,−3.45) — the office floor's
    // tongue ends in their pocket; the descent crosses at W1's south-west end (below).
    w.walk_to(Wp::new(10.7, 47.3, -4.1).r(0.4), "the gap's north approach");
    // THE STAIRWELL DOOR #192 IS THE CROSSING (this session's fix of the old 2 m pocket stand-in):
    // the well's parapets W1/W2 are the stairwell's own walls, and the disc puts the E
    // door_large_stairwell #192 (10.22, 46.75, −5.21 [D], lock ch 0x12) IN that wall line — the
    // doorway is the opening; there is no walk-around (the parapets are continuous — W1
    // (9.2,−5.8)→(12.75,−3.5), the probes hit its 0x03/0x15 faces at body height [D] — and the
    // game's own nav graph has no link across the pocket either: the office-floor component ends
    // at n45/n46/n47, the stair chain starts fresh at n94 [D nav BFS]). The keypad lock #177 is
    // create-unlocked on Operative [D #177 p6 = 1; M 0x1ae1f8:67-69] and Lock_Update case 1
    // (0x1ae3d8) holds ch 0x12 on, so the door answers E from level start.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x12),
            "the keypad's Operative unlock holds ch 0x12 [D #177 p6; M 0x1ae3d8 case 1]");
    }
    w.walk_to(Wp::new(10.22, 47.4, -3.6).r(0.9), "the stairwell door's front tile");
    let d192 = w.scene.sim.as_ref().unwrap().drones.doors.list.iter()
        .position(|d| (d.pos[0] - 10.22).abs() < 0.5 && (d.pos[2] + 5.21).abs() < 0.5)
        .expect("door_large_stairwell #192 [D]");
    w.use_door(d192, "the stairwell door (the ch-0x12 unlock)");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let st = sim.drones.doors.list[d192].state;
        assert!(st & 0xc != 0, "the stairwell door opens on the walk's E, state {st:#x}");
    }
    w.wait(45); // the swing finishes before the doorway walk (the auto-close is 360 ticks [M Door_Update])
    w.walk_to(Wp::new(11.0, 47.4, -5.5).r(0.9), "through the stairwell doorway");
    w.walk_to(Wp::new(13.0, 46.9, -6.2).r(0.9), "the top flight east");
    w.walk_to(Wp::new(19.5, 44.6, -5.2).r(1.0), "the north flight down (46.8→44.4)");
    w.walk_to(Wp::new(22.3, 43.6, -3.2).r(1.0), "the north-east landing 43.1");
    band(&w, 42.6, 44.6, "the NE landing");
    w.walk_to(Wp::new(23.0, 39.5, 3.6).r(1.0), "the east flight down (43.1→39.1)");
    w.walk_to(Wp::new(20.5, 39.5, 5.8).r(1.0), "the south-east landing 39.1");
    band(&w, 38.6, 40.5, "the SE landing");
    w.walk_to(Wp::new(15.2, 35.6, 6.0).r(1.0), "the south run down (39.1→35.2)");
    w.walk_to(Wp::new(14.8, 35.4, 5.0).r(0.9), "the south-west landing 35.2");
    band(&w, 34.6, 36.6, "the SW landing");
    w.walk_to(Wp::new(14.8, 31.4, -2.6).r(1.0), "the west flight C down (35.2→31.1)");
    band(&w, 30.4, 32.5, "flight C's bottom");
    w.walk_to(Wp::new(22.4, 31.3, -3.0).r(1.0), "the north strip 31.1 east");
    w.walk_to(Wp::new(23.0, 27.4, 3.8).r(1.0), "the east connector down (31.1→27.1)");
    band(&w, 26.4, 29.3, "connector 1's bottom");
    w.walk_to(Wp::new(15.0, 27.4, 4.8).r(1.0), "the south strip 27.1 west");
    w.walk_to(Wp::new(14.8, 23.3, -2.6).r(1.0), "the west flight B down (27.1→23.0)");
    band(&w, 22.4, 24.8, "flight B's bottom");
    w.walk_to(Wp::new(22.4, 23.2, -3.0).r(1.0), "the north strip 23.0 east");
    w.walk_to(Wp::new(23.0, 19.3, 3.8).r(1.0), "the east connector down (23.0→18.9)");
    band(&w, 18.2, 21.1, "connector 2's bottom");
    w.walk_to(Wp::new(15.0, 19.2, 4.8).r(1.0), "the south strip 18.9 west");
    w.walk_to(Wp::new(14.8, 15.4, -2.6).r(1.0), "the west flight A down (18.9→15.0)");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let y = sim.bond.pos[1];
        assert!(y < 17.2, "the descent never reached the lower floor (y {y}) [D 15.0 probe + the 1.09 standing offset]");
        assert!(!sim.drones.channels.on(0x7c), "the descent never crossed the atrium plates (ch 0x7C) [D §4]");
        // [S] unverified — NOT a pass/fail assert: "no 0x48 writer on this level" rests on a
        // missing writer, and the port may simply lack it (level-11-every-objective-2). The
        // mission-state asserts below are the [M]-backed pins.
        eprintln!("[011] [S] unverified: ch 0x48 off at the stairwell bottom (expected: no lower-floor writer on this level)", );
        assert!(!sim.drones.channels.on(100), "no mission success on this level [D §2; M 0x1827d8:119-123]");
        assert_eq!(sim.drones.mission.states[15], [3, 3],
            "both records still announced at the stairwell bottom [M 0x1827d8 case 3]");
    }
    eprintln!("[011] the stairwell descent WALKED at +{} ticks", w.total_ticks);
    // Leg 3: the Load Point #158 and the E press ([M 0x1a5170 case 0x1d → 0x1b2318]).
    w.walk_to(Wp::new(14.6, 17.0, -2.8).r(0.9), "the Load Point");
    let mut loaded = false;
    for _ in 0..60 {
        w.tick(nf_game::player::Input::default(), true);
        if w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { loaded = true; break; }
    }
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.level_exit, Some(0x0700_0012),
        "E at the Load Point #158 loads 07000012 [M 0x1b2318]");
    assert!(loaded, "the exit fired");
    // The exit's end-of-level mission rows: neither record completed here — their writers live on
    // 07000012 (#679's spawn tile) and on 07000013 (the whole-mission AND) [D §2].
    {
        let sim = w.scene.sim.as_ref().unwrap();
        // [S] unverified — NOT a pass/fail assert: "no 0x48 writer exists" rests on a missing
        // writer, and the port may simply lack it (level-11-every-objective-2). The hunt is in
        // docs/reports/level-11-every-objective-2.md.
        eprintln!("[011] [S] unverified: ch 0x48 never fired on 07000011 (expected: no writer exists)", );
        assert!(!sim.drones.channels.on(100), "no mission success on this level [D §2; M 0x1827d8:119-123]");
        assert_eq!(sim.drones.mission.states[15], [3, 3],
            "both records announced (not complete) at the exit [M 0x1827d8 case 3]");
    }
    // THE CHAIN INTO THE NEXT LEVEL (real input all the way): the exit's own next level loads
    // [M the E arm → ResetMap_LevelToLoad 0x1b24b8], and 07000012's spawn #0 (22.49, 0.19, 19.16
    // [D]) stands inside TouchOnce #679's tile (21.48, 0.23, 18.64)–(23.59, 1.21, 19.62) [D] —
    // ch 0x48 fires under Bond's feet and the monitor completes the carried "Gain access to
    // lower office floor" [M 0x1827d8 case 3 → state 4]. "Escape from the tower" stays
    // announced: its writer is the whole-mission AND, which first fires on 07000013 [D §2].
    // (level-11-every-objective-3: the load is the difficulty-3 front-end path — the exact call
    // walkreal_07000012_live calibrates tower2's route on; the spawn pins here are
    // difficulty-independent [M the 0x1827d8 announce + #679's contact write].)
    let scene2 = nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 3)
        .expect("07000012 loads after the exit");
    let mut w2 = Walker::new(scene2);
    w2.wait_enabled(3000, "07000012's intro");
    w2.wait(90);
    {
        let sim = w2.scene.sim.as_ref().unwrap();
        assert_eq!(sim.level, 0x0700_0012, "the exit's next level [D #158 next]");
        assert!(sim.drones.channels.on(0x48),
            "#679's spawn tile fires ch 0x48 [D (21.48, 0.23, 18.64)–(23.59, 1.21, 19.62)]");
        assert_eq!(sim.drones.mission.states[15][1], 4,
            "'Gain access to lower office floor' COMPLETE on the next level's spawn [M 0x1827d8 case 3]");
        assert_eq!(sim.drones.mission.states[15][0], 3,
            "'Escape from the tower' still announced (completes on 07000013) [D §2]");
    }
    // THE EXTENSION (level-11-every-objective-3): every remaining objective on real input, to the
    // mission's end — the next level's load. The ch 0x48 writer is TouchOnce #679 (pinned above;
    // the port has it — triggers.rs). Next, 0x04000037 "Activate elevator override control in
    // security center" completes on ch 0x49 at the console E (#731 [D 07000012.md §2; M
    // Trigger_Activate 0x1b2318 type 1]); the lift call #699 (ch 0x58) and the shaft entrance
    // (#683 holds ch 0x1F) arm the gated exit #669, whose auto-fire [M Trigger_Update 0x1b1a78
    // case 10] loads 0700004A — the next level's load. The route is tower2's (shared with
    // walkreal_07000012_live; its two measured stand-ins carried over — see the header);
    // tower2's opening block re-pins the spawn states on this same walker.
    tower2::walk_12(&mut w2);
    tower2::walk_12_exit(&mut w2);
    // The mission's last reachable objective: 0x04000038 "Escape through the elevator shaft"
    // (ch 0x4D) has NO writer on 07000012 [D 07000012.md §2] — its writer is TouchOnce #277 on
    // 0700004A (the shaft's first plate, y −9.87…−7.65 [D 0700004a.md §2]). The game carries the
    // RamSave through the real load [M Player_RamLoad 0x1957a8]; 4A's start #43 (4.26, −7.71,
    // −4.76 [D]) drops Bond INTO #277's volume, ch 0x4D fires, and the monitor completes the
    // record the next frame (it runs before the object handler [M 0x182c60 from 0x1c9790:17]).
    // 0x04000035 "Escape from the tower" stays announced: its writer is the whole-mission AND
    // (ch 100 [M 0x1827d8:119-123]), which first fires on 07000013 [D §2] — the mission's
    // remaining part, beyond this walk's endpoint.
    let save = {
        let sim = w2.scene.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request,
            &sim.drones.inventory, &sim.drones.channels).with_mission(&sim.drones.mission)
    };
    let mut w3 = Walker::new(nf_view::load_with_carry(&iso, "0700004a.bin", false, Some(&save))
        .expect("0700004A loads after 07000012's exit"));
    w3.wait_enabled(3000, "the 4A intro");
    let mut landed = false;
    for _ in 0..600 {
        w3.tick(nf_game::player::Input::default(), false);
        if w3.scene.sim.as_ref().unwrap().drones.channels.on(0x4d) { landed = true; break; }
    }
    assert!(landed, "4A: the start #43 drops Bond into touch #277 → ch 0x4D [D]");
    for _ in 0..30 { w3.tick(nf_game::player::Input::default(), false); }
    {
        let sim = w3.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission.states[15][1], 4,
            "row 15: 'Gain access to lower office floor' stays complete (the RamSave carries it) [M 0x182270]");
        assert_eq!(sim.drones.mission.states[16][1], 4,
            "'Escape through the elevator shaft' completes through the real exit load (the late AND) [M 0x1827d8]");
        assert_eq!(sim.drones.mission.states[16][0], 4,
            "'Activate elevator override control' stays complete [M 0x182270 keeps previous parts]");
        assert_eq!(sim.drones.mission.states[15][0], 3,
            "'Escape from the tower' still announced (its writer is 07000013's whole-mission AND) [D §2]");
    }
    eprintln!("[011] walked out in {} ticks; stuck {}; falls {}", w.total_ticks, w.report.stuck_events.len(), w.report.falls.len());
}
