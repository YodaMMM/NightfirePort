//! walkreal-b: 07000013 (HT Tower2 C, the tower lobby → the front entrance) walked with the
//! player's own movement. The warm chain (mission 0x07000011's parts): 07000012 WALKED (the
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
//! driving interlude; the port logs and skips driving codes [M 0x182c60]) → E at the level's own
//! exit #72 (−12.79, −367.97, 68.25) requests 0x07300013 with its gate 0x4E off (the E arm falls
//! through [M 0x1b2318 case 10]; dead data — ch 0x4E has no writer on this disc [D]).
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
#[ignore = "the gate raise chain is proven green by gate13_live (the panel E → ch 0x23 → the SS rise → the mouth clears); this end-to-end walk is blocked twice over: (1) the walked approach to the panel — the lobby's west furniture maze wedges the walker at every probed lane (z 25, z 20-22, z 28-31; STUCK at (−8.1, 21.6), (−12.8, 30.8), (−13.9, 31.9), segment visible but the push cancels the move), and (2) the reach-only teleport that does reach the panel leaves the port's room cell west — every later leg falls (y −386 → −852) while the warm no-E load stands stable at y −368.33 for 240+ ticks (fall13_probe). Needs the cell re-resolution after a teleport, or the game's own path to the panel (through the security-room door #355, open ch 0x11d — no writer found [D]). The pre-E legs, the gate mechanism and the entrance/exit chain are pinned by walk_07000013_live, gate13_live and the probes"]
fn walk_the_warm_chain_to_the_front_entrance_and_out() {
    let iso = iso();
    // ---- 07000012 walked (the shared route) ----
    // Agent difficulty (3): the keypad locks' param-6 tag auto-unlocks them at create on
    // Operative [M 0x1ae1f8 tail], and the route's burn leg pins the Agent+ behaviour
    // (the burn writes ch 0xa; the Operative create-unlock writes nothing [M Lock_Unlock
    // 0x1b0650's flag arm]). The block drops the scene before the next level loads (the 4 GB
    // sandbox OOMs three resident levels).
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
    // The panel E — WALKED, not teleported: the port's room-cell rides the walk (a teleport
    // across the lobby leaves the scoped collision in the old cell and the floor vanishes —
    // see the report). The panel's port-side entity sphere sits at the model centre transformed
    // by the SP's stream matrix (t −1.41 x) — (−19.49, −368.23, 25.01); the game-side
    // displacement of a world-baked glist is the report's live check. The press is the real use
    // key.
    w.lenient = true;
    w.walk_to(Wp::new(-4.0, -368.5, 20.0).r(1.2), "13: off the car, south-west");
    w.walk_to(Wp::new(-10.0, -368.5, 20.8).r(1.2), "13: the south-west aisle");
    w.walk_to(Wp::new(-15.0, -368.6, 22.5).r(1.0), "13: the aisle west end");
    w.walk_to(Wp::new(-18.4, -369.0, 24.6).r(0.7), "13: at the shutter panel");
    let mut ch23 = false;
    for _ in 0..80 {
        w.tick(nf_game::player::Input::default(), true);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x23) { ch23 = true; break; }
    }
    assert!(ch23, "E at the shutter panel writes ch 0x23 [M 0x1dbda8:221-229; D SP #161 out]");
    // The gate lifts while Bond walks south (the SS playhead runs the ~4-unit rise over ~200
    // ticks [probe]).
    w.walk_to(Wp::new(5.0, -368.4, 26.5).r(1.5), "13: the lobby east lane");
    w.walk_to(Wp::new(3.0, -368.4, 45.0).r(1.5), "13: the lobby mid");
    w.walk_to(Wp::new(-6.0, -368.2, 60.0).r(1.5), "13: the lobby south");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let y = sim.drones.sss.list.iter().find(|s| s.channel == 0x23).map(|s| s.matrix[3][1]).unwrap_or(0.0);
        assert!(y > 3.0, "the gate SS has risen to key 1 by the lobby's south end (y offset {y}) [M SS_Update 0x1a9948]");
    }
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

    // LevelToEndTo 0x07300013 (the driving interlude) is the mission end's own hand-off [M
    // 0x182c60 case 6] — latched above. The level's own exit #72 (−12.79, −367.97, 68.25; the
    // E-load fallback) sits beyond the entrance's west wall (the walked probe wedges at x ≈ −9.1
    // [D]) and is DEAD DATA anyway (gate ch 0x4E has no writer on the disc [D §3]) — the walkreal
    // route ends at the mission latch; the E-arm itself is covered by walk_07000013_live.
    eprintln!("[013] walked out in {} ticks; stuck {}; falls {}", w.total_ticks, w.report.stuck_events.len(), w.report.falls.len());
}
