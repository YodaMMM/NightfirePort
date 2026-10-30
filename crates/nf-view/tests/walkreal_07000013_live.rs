//! walkreal-b (716c): 07000013 (HT Tower2 C, the tower lobby → the front entrance) walked with
//! the player's own movement, START TO FINISH: spawn → the security-room door (portal 1) → the
//! panel E (ch 0x23, the gate rises) → back out → the entrance → Mission Complete. The warm chain (mission 0x07000011's parts): 07000012 WALKED (the
//! keypad burn, the console E, the shaft entrance — the shared `tower2` route, with its one
//! documented doorway stand-in) → its exit loads 0700004A, whose start #43 (4.26, −7.71, −4.76
//! [D]) sits INSIDE TouchOnce #277 (out 0x4D, y −9.87…−7.65 [D]) — Bond drops onto the touch at
//! the spawn (the game's own placement; no input writes) and "Escape through the elevator shaft"
//! completes — the carried RamSave arrives at 07000013 ([M 0x182270 keeps previous parts]) → the
//! spawn touches fire 0x28 and 0x4A ("Gain access to lobby" completes — the late AND) → the intro
//! ScriptPlayer #65 (0x060006D7 [D]) releases Bond at tick ≈351 → the REAL walk south through the
//! lobby to the front entrance → TouchOnce #156 (−8.61…−4.15, −368.81…−366.74, 65.09…67.16 [D])
//! writes ch 0x50 (the objective completes) → the success touches #73/#164/#165 write ch 0x64
//! (=100) and with every part's channel on (0x48, 0x49, 0x4D, 0x4A, 0x50) the monitor raises
//! MISSION SUCCESS [M 0x1827d8:119-123] → "Mission Complete" → LevelToEndTo 0x07300013 (the
//! driving interlude; the port logs and skips driving codes [M 0x182c60]) → the mission_end
//! latch = the level exit firing. The mission's four objectives [L ee_start_05.bin @
//! 0x2a41b8/0x2a41e8/0x2a4200 — 2+1+1 records; docs/formats/mission-data.md], asserted in
//! player order: "Activate elevator override control in security center" (ch 0x49 →
//! states[16][0] == 4, tower2's console E), "Escape through the elevator shaft" (ch 0x4D →
//! states[16][1] == 4, the 4A spawn drop), "Gain access to lobby" (ch 0x4A → states[17][0]
//! == 4, the 13 spawn touch), "Raise security gate and escape through front entrance" (ch
//! 0x50 → states[18][0] 3→4, the panel E + the entrance touch).
//!
//! THE CLOSING PIN (level-13-spawn-to-end-walk): part 0's records ride the same warm chain and
//! fold only here — 0x04000036 "Gain access to lower office floor" (ch 0x48) completes on the
//! chain's first monitor tick after 07000012's spawn touch #679 writes ch 0x48 [D; M 0x1827d8
//! case 3], and 0x04000035 "Escape from the tower" (ch 0x64 = 100 [L c07000011.rs]) — announced
//! on 07000011 and pinned announced-only by walkreal_07000011_live — is STILL announced (3) when
//! 13 loads: its writer is the whole-mission AND ch 100 [M 0x1827d8:119-123], which first fires
//! HERE, the tick the success touches #73/#164/#165 write ch 0x64. The same tick the monitor's
//! case 3 folds it 3 → 4 and the all-AND raises ch 100 → internal 6 [M 0x182c60:23-28] — no
//! channel pokes anywhere; every objective's condition is written by the game's own objects.
//!
//! The fake lift (SP #161, trigger ch 0x23) has no writer on the disc [D §4] — dead data; the
//! route out is the front entrance, not the lift.

#[path = "common/walker_b2.rs"]
mod walker;
#[path = "common/tower2.rs"]
mod tower2;

use walker::{Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn walk_the_warm_chain_to_the_front_entrance_and_out() {
    let iso = iso();
    // ---- 07000012 walked (the shared route) ----
    // Agent difficulty (3): the keypad locks' param-6 tag auto-unlocks them at create on
    // Operative [M 0x1ae1f8 tail], and the route's burn leg pins the Agent+ behaviour
    // (the burn writes ch 0xa; the Operative create-unlock writes nothing [M Lock_Unlock
    // 0x1b0650's flag arm]). The block drops the scene before the next level loads (a 4 GB
    // test machine OOMs with three resident levels).
    let save = {
        let mut w = Walker::new(nf_view::load_with_carry_difficulty(&iso, "07000012.bin", false, None, 3).unwrap());
        tower2::walk_12(&mut w);
        tower2::walk_12_exit(&mut w);
        let sim = w.scene.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request,
            &sim.drones.inventory, &sim.drones.channels).with_mission(&sim.drones.mission)
    };

    // ---- 0700004A: the spawn drop onto the 0x4D touch (the game's own placement) ----
    let save = {
        let mut w = Walker::new(nf_view::load_with_carry(&iso, "0700004a.bin", false, Some(&save)).unwrap());
        w.wait_enabled(3000, "the 4A intro");
        let mut landed = false;
        for _ in 0..600 {
            w.tick(nf_game::player::Input::default(), false);
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x4d) { landed = true; break; }
        }
        assert!(landed, "4A: the start #43 drops Bond into touch #277 → ch 0x4D [D]");
        // fix-objective-banner: the monitor runs at the TOP of the tick — the objective completes the
        // tick after the channel rises.
        for _ in 0..2 { w.tick(nf_game::player::Input::default(), false); }
        assert_eq!(w.scene.sim.as_ref().unwrap().drones.mission.states[16][1], 4,
            "12's 'Escape through the elevator shaft' completes (the late AND) [M 0x1827d8]");
        let sim = w.scene.sim.as_ref().unwrap();
        nf_game::player::RamSave::capture(&sim.drones.health, sim.gun.switch_request,
            &sim.drones.inventory, &sim.drones.channels).with_mission(&sim.drones.mission)
    };

    // ---- 07000013 walked ----
    let mut w = Walker::new(nf_view::load_with_carry(&iso, "07000013.bin", false, Some(&save)).unwrap());
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x48) && sim.drones.channels.on(0x49) && sim.drones.channels.on(0x4d),
            "the previous parts' channels carry through the warm load [M 0x182270]");
    }
    w.wait_enabled(3000, "the 13 intro");
    w.wait(120);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x4a), "the spawn touch #153 fires ch 0x4A [D]");
        assert_eq!(sim.drones.mission.states[17][0], 4, "4A's 'Gain access to lobby' completes (the late AND) [M]");
        assert_eq!(sim.drones.mission.states[18][0], 3, "'Raise security gate…' announces (enable 0x4A) [L]");
        // Part 0's rows at the carried load [M 0x182270 leaves previous parts alone]: 0x04000036
        // folded on the chain (12's spawn touch #679 wrote ch 0x48 [D]); 0x04000035 still announced
        // — its writer is the whole-mission AND that first fires on THIS level [M 0x1827d8:119-123].
        assert_eq!(sim.drones.mission.states[15][1], 4,
            "11's 'Gain access to lower office floor' folded on the warm chain (ch 0x48, written by 12's spawn touch #679 [D]; M 0x1827d8 case 3)");
        assert_eq!(sim.drones.mission.states[15][0], 3,
            "0x04000035 'Escape from the tower' still announced at 13's load [L c07000011.rs; M 0x1827d8:119-123 — its AND writer first fires here]");
        assert!(!sim.drones.channels.on(100), "ch 100 is off at 13's load — the success touches are its first writer [D]");
    }

    // THE SECURITY GATE: the "Shutters2" SimpleScript #149 (out ch 0x23 [D param 3]) is the
    // entrance mouth's panel (the SS cel's glist spans x −8.54..−4.97 at z 65.02 [D the object
    // bounds] — exactly the wall the old walk pinned at). Its writer is ScriptPlayer #161
    // ("SCRIPTPLAYER_Elevators", mode 64 preload, mask 4, out ch 0x23 [D params (5,4),(6,23)]):
    // the preload's Entity op creates 0x02000A2C "SCRIPT_SHUTTER_CLOSE" — the wall panel's
    // world-baked glist at (−18.08, −368.23, 25.01) [D obj 70] — with obj+0xE4 = the SP handle
    // [M Script_EntityStart 0x1d8d78:110], so the E pick's class-0 arm reaches SP_Activate
    // [M Player_Activate 0x1a5170:121-122] and the played script writes ch 0x23
    // [M SP_Update 0x1dbda8:221-229]. The panel is the west-wall shutter display ("SCRIPT_
    // SHUTTER_CLOSE/OPEN" share its glist centre [D objs 68/69]); ch 0x23 runs the SS playhead
    // to key 1 and the gate lifts [M SS_Update 0x1a9948 + KeyFrame_Interp 0x1e1140].
    // Sister: before the press ch 0x23 is off and the SS rests at key 0.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x23), "ch 0x23 off before the panel [D]");
        let rest = sim.drones.sss.list.iter().find(|s| s.channel == 0x23)
            .map(|s| s.matrix[3][1]).expect("the Shutters2 SS on ch 0x23 [D]");
        assert!(rest.abs() < 0.05, "the gate SS rests at key 0 (y offset {rest}) [D]");
    }
    // The panel E — WALKED through the security room (716c): the lobby's west "furniture maze"
    // truly blocks every direct lane (the fake-lift doors' baked mesh at x −14.6..−12.9 /
    // z 31.3..32.9 inside the security-room glass band x −18.4..−10.6 / z 27.5..35.3, plus the
    // room's own east wall — probe13_cell's stuck A/B/C object dumps), and the panel display
    // (−18.08, −368.23, 25.01 [D obj 70]) sits INSIDE the security room's footprint. The
    // player's route is the unlocked north-west door at (−26.65, −368.36, 37.44) — the only
    // west door the sim creates, no lock channel, E-opens [M Door_Activate's toggle] — whose
    // doorway carries portal 1 (cels 21↔28 [D the portal record's celA/celB]); Bond's runtime
    // cel crosses 21→28 through the stateful relink's portal arm (Cel_ObjectLeftCel 0x1e0d58)
    // and the room's own floor (cel 28's bucket) holds him.
    w.walk_to(Wp::new(5.0, -368.4, 26.5).r(1.5), "13: the lobby east lane");
    w.walk_to(Wp::new(-6.0, -368.4, 40.0).r(1.5), "13: the lobby west mid");
    w.walk_to(Wp::new(-14.0, -368.4, 40.0).r(1.5), "13: the lobby north-west");
    w.walk_to(Wp::new(-22.0, -368.4, 39.5).r(1.5), "13: past the glass band");
    w.walk_to(Wp::new(-24.5, -368.4, 38.8).r(1.2), "13: at the security-room door");
    w.walk_to(Wp::at(-26.65, -368.36, 37.44, walker::Act::Use { use_ticks: 90 }).r(1.4), "13: E the security-room door");
    w.wait(120); // the sliding door finishes [probe: state 0x8 → 0x4 within the wait]
    w.walk_to(Wp::new(-27.0, -368.4, 37.0).r(1.0), "13: through the doorway");
    w.walk_to(Wp::new(-27.0, -368.4, 35.0).r(1.2), "13: inside the security room");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.bond.cel, Some(28), "the portal-1 cross relinks Bond into the security room's cel [M 0x1e0d58]");
    }
    w.walk_to(Wp::new(-25.5, -368.4, 32.5).r(1.2), "13: the room south-east");
    w.walk_to(Wp::new(-22.0, -368.4, 29.0).r(1.2), "13: the room south");
    w.walk_to(Wp::new(-18.5, -368.4, 26.2).r(1.1), "13: at the shutter panel");
    // The press is the real use key at the walked spot.
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(!sim.drones.channels.on(0x23), "ch 0x23 off before the panel E [D]");
    }
    let mut ch23 = false;
    for t in 0..150 {
        let sim = w.scene.sim.as_ref().unwrap();
        let desired = (-18.08f32 - sim.bond.pos[0]).atan2(25.01 - sim.bond.pos[2]);
        let err = walker::wrap_pi(desired - sim.bond.yaw);
        let input = nf_game::player::Input { turn: err.clamp(-0.05, 0.05), ..Default::default() };
        w.tick(input, t % 6 == 0);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x23) { ch23 = true; break; }
    }
    assert!(ch23, "the walked E at the shutter panel writes ch 0x23 [M 0x1dbda8:221-229; D SP #161 out]");
    // The gate lifts while Bond walks back out and south (the SS playhead runs the ~4-unit rise
    // over ~200 ticks [probe]).
    w.walk_to(Wp::new(-22.0, -368.4, 29.0).r(1.2), "13: back north");
    w.walk_to(Wp::new(-25.5, -368.4, 32.5).r(1.2), "13: back to the room's north-east");
    w.walk_to(Wp::new(-27.0, -368.4, 35.5).r(1.2), "13: back to the doorway");
    // The door auto-closes ≈150 ticks after the E [probe]: press it (only while closed — the E
    // is a toggle [M Door_Activate]) and hold forward through the doorway's west side in one
    // loop, so the crossing rides the open window.
    let di = w.scene.sim.as_ref().unwrap().drones.doors.list.iter()
        .position(|d| walker::dxz(d.pos, [-26.65, -368.36, 37.44]) < 2.0).expect("the security-room door");
    let mut out = false;
    for _ in 0..900 {
        let (door_closed, bp) = {
            let sim = w.scene.sim.as_ref().unwrap();
            let d = &sim.drones.doors.list[di];
            (d.state & 0xc == 0, sim.bond.pos)
        };
        if bp[2] > 38.4 && (-28.0..=-25.5).contains(&bp[0]) { out = true; break; }
        let dx = -27.05f32 - bp[0];
        let dz = 38.7 - bp[2];
        let desired = dx.atan2(dz);
        let err = walker::wrap_pi(desired - w.scene.sim.as_ref().unwrap().bond.yaw);
        let input = nf_game::player::Input { turn: err.clamp(-0.05, 0.05), forward: 1.0, ..Default::default() };
        w.tick(input, door_closed);
    }
    assert!(out, "13: out through the doorway while the door's open window holds [probe]");
    w.walk_to(Wp::new(-24.5, -368.4, 38.8).r(1.2), "13: out of the room");
    w.walk_to(Wp::new(-14.0, -368.4, 40.0).r(1.5), "13: the lobby north-west again");
    w.walk_to(Wp::new(-6.0, -368.2, 55.0).r(1.5), "13: the lobby south");
    w.walk_to(Wp::new(-6.2, -368.0, 66.3).r(0.6), "13: the front entrance");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.channels.on(0x50), "the entrance touch #156 writes ch 0x50 [D]");
        assert_eq!(sim.drones.mission.states[18][0], 4, "the entrance objective completes [M 0x1827d8]");
    }
    // The success touches #73/#164/#165 (ch 0x64) + the AND over all parts → MISSION SUCCESS
    // [M 0x1827d8:119-123]. The three success plates sit across the entrance mouth [D] — walk
    // the band while waiting for the latch.
    let mut success = false;
    for band in [[-5.0f32, 66.0], [-7.6, 65.8], [-6.4, 66.4], [-6.4, 65.6]] {
        for _ in 0..400 {
            w.tick(nf_game::player::Input::default(), false);
            if w.scene.sim.as_ref().unwrap().drones.mission.state == 6 { success = true; break; }
        }
        if success { break; }
        let sim = w.scene.sim.as_ref().unwrap();
        eprintln!("[013] no latch at {:?}: ch64 {} ch50 {} ch4a {} ch4d {} ch49 {} ch48 {}",
            band, sim.drones.channels.on(0x64), sim.drones.channels.on(0x50), sim.drones.channels.on(0x4d),
            sim.drones.channels.on(0x4a), sim.drones.channels.on(0x49), sim.drones.channels.on(0x48));
        let _ = w.try_walk_to(Wp::new(band[0], -368.0, band[1]).r(0.8), "13: the success band");
    }
    assert!(success, "the AND raises mission success at the entrance [M 0x1827d8]");
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x64), "the success touches wrote ch 0x64 [D]");

    // THE LEVEL EXIT: MissionState 6's own sequence — case 6's "Mission Complete" banner with
    // LevelToEndTo 0x07300013 → case 7's 240-tick hold → case 4's ResetMap_LevelToLoad
    // [M 0x182c60:85-122, 65-76]. 0x07300013 is a driving interlude code → the port's end
    // sequence arrives as the viewer's `mission_end` latch (the 04 walkreal's pattern; the
    // walker's ticks drain the layer's events through the scene's real hud path [M scene.rs]).
    // The level's own exit #72 (−12.79, −367.97, 68.25; the E-load fallback) is DEAD DATA —
    // its gate ch 0x4E has no writer on the disc [D §3] and the walked probe wedges at x ≈ −9.1
    // — so the game's real exit IS this mission-complete hand-off, and the E-arm itself is
    // covered by walk_07000013_live.
    let mut ended = false;
    for _ in 0..500u32 {
        w.tick(nf_game::player::Input::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.mission_end.is_some() { ended = true; break; }
    }
    assert!(ended, "the mission-complete sequence must land (case 6 banner → case 4 ResetMap) [M 0x182c60]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission_end, Some(0x0730_0013),
            "LevelEnd 0x07300013 → the end sequence with LevelToEndTo 0x07300013 [M 0x182c60:89-90]");
        assert_eq!(sim.drones.mission.to_end, 0x0730_0013, "LevelToEndTo [M 0x182c60:89-90]");
        assert_eq!(sim.drones.mission.state, 6, "MissionState 6 = complete [M 0x182c60:88]");
        assert_eq!(sim.drones.mission.states[18][0], 4, "the entrance objective stays complete");
        // THE CLOSING PIN: the whole-mission AND over every part's condition [M 0x1827d8:119-123]
        // folded 0x04000035 the tick the success touches wrote ch 0x64 = 100 [D] — the record
        // walkreal_07000011_live could only pin as announced.
        assert_eq!(sim.drones.mission.states[15][0], 4,
            "0x04000035 'Escape from the tower' COMPLETE: the success touches wrote ch 0x64 = 100 and the whole-mission AND folded it [M 0x1827d8:119-123 case 3; D]");
        assert!(sim.drones.channels.on(100), "the monitor's all-AND raised ch 100 [M 0x1827d8:119-123]");
        assert_eq!(sim.drones.mission.states[15][1], 4, "11's office-floor objective stays complete");
        assert_eq!(sim.drones.mission.states[16][0], 4, "12's console objective stays complete");
        assert_eq!(sim.drones.mission.states[16][1], 4, "12's shaft objective stays complete");
        assert_eq!(sim.drones.mission.states[17][0], 4, "4A's lobby objective stays complete");
    }

    // LevelToEndTo 0x07300013 (the driving interlude) is the mission end's own hand-off [M
    // 0x182c60 case 6] — latched above. The level's own exit #72 (−12.79, −367.97, 68.25; the
    // E-load fallback) sits beyond the entrance's west wall (the walked probe wedges at x ≈ −9.1
    // [D]) and is DEAD DATA anyway (gate ch 0x4E has no writer on the disc [D §3]) — the walkreal
    // route ends at the level exit; the E-arm itself is covered by walk_07000013_live.
    eprintln!("[013] walked out in {} ticks; stuck {}; falls {}", w.total_ticks, w.report.stuck_events.len(), w.report.falls.len());
}
