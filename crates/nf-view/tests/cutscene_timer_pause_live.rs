//! level-07-audit: the cut-scenes freeze the mission clock (the party's two NIS windows).
//!
//! THE GAME LAW [M]: `Script_EventHandler`'s event-4 arm is `Player_Disable(glb_players, 0)` +
//! `PlarStat_LogTimerPause(0)` [M 0x1d94c0:118-121] — the cut-scene ALSO pauses the mission
//! timer. The accumulator `PlarStat_LogUpdateElapsedTime` adds `psiGetTimeIn100ths() − +0x2C`
//! into `+0x30` (the elapsed play time) ONLY while the pause byte `+0x34` is 0 [M 0x1995a0:9-14;
//! the pause byte set by `PlarStat_LogTimerPause` 0x199520:8-11]. The NIS's free re-enables both:
//! `Script_Free` runs `Player_Enable` + `PlarStat_LogTimerUnpause(0)` (`+0x2C := the clock,
//! +0x34 := 0`) [M 0x1da880:44-48, 0x199550:8-21].
//!
//! THE LEVEL [D]: both of 07000007's cut-scenes disable the player — SP #419 (0x06000072, Drake's
//! reveal, script length 3000, stream 20 event 4 @tick 0) and SP #420 (0x06000073, the
//! control-taking scene, length 4326, stream 5 event 4 @tick 0, hand-back event 6 @4326) [D the
//! stream decode, level-07-audit]. The trigger is the real route: the hall walk-in TouchOnce #412
//! (94.27, 43.61, −45.92, out ch 0x48) fires SP #420 [D placements].
//!
//! THE PORT GAP (level-07-audit): `Request::PlayerDisable`/`PlayerEnable` never touched
//! `progress.log_timer_pause`/`log_timer_unpause` — the mission clock kept counting through both
//! scenes. Fixed in scene.rs's step_bond (this test pins it through the real input path).

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::{Act, Walker, Wp};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn elapsed(w: &Walker) -> u32 { w.scene.sim.as_ref().unwrap().drones.progress.stats[0].elapsed }

/// The real walk to the story point (the party anchors, the party_door_lock route): spawn → the
/// west aisle → the hall → E the hall door #244 → cross the walk-in plate #412 → SP #420 takes
/// control. No teleports.
fn walk_to_the_story_point(w: &mut Walker) {
    w.wait_enabled(3000, "the party start");
    w.wait(60);
    for (name, at) in [
        // drone-bump-collision-4: the -3 round's north-lane anchors (z ≈ −46.6) are WRONG —
        // the nav web's hall nodes sit in the crowd band and the lane itself wedges on the
        // hall's west wall at x ≈ 80 (the -3 report's own "What's left" item 1). The hall is
        // crossed on walkreal_07000007_live's PROVEN south-band line — the west mouth
        // (76.6, −50.7) → the floor (83.1, −50.5) → the east (93.3, −50.4) — the exact anchors
        // walkreal_07000007 walks green through the armed crowd [drone-bump-collision-3:
        // walkreal_07 2/2 on the kind-gated push]. The crowd is ARMED: enabled bots with
        // running control (the party states write no +0x4F8 collision bits [M PartyGirlInit
        // 0x1728c0 / PartyGirl 0x160140]) fill +0x154 on their own control tick [M
        // PreDroneControl 0x148db0:35-48], so DoCollision's near arm fires inside 2.0 [M
        // 0x149ec8:30-33, disasm] and NDrone2_Collision writes kind +0x464 = 0x800 [M
        // 0x140e90:95-101] → the Jointy push runs at capsule contact [M 0x1ec180:229-281;
        // drone_bump_live 2/2] — this walk neither disarms nor dodges it; it IS the line a
        // player walks, on walkreal's measured timing (the pre-hall anchors are identical to
        // walkreal's, so Bond's tick phase at the mouth matches the proven walk). The main
        // agent's live check (does the game's crowd stop Bond mid-floor?) is listed in
        // docs/reports/drone-bump-collision-3.md.
        ("west aisle south", [35.7f32, 43.6, -70.6]),
        ("west aisle north", [42.6, 43.6, -70.5]),
        ("the west wall run", [42.6, 43.6, -45.0]),
        ("the north door", [49.4, 43.6, -44.7]),
        ("the north row west", [62.7, 43.6, -44.3]),
        ("the north row east", [76.5, 43.6, -44.9]),
        ("the hall west mouth", [76.6, 43.6, -50.7]),
        ("the hall floor", [83.1, 43.6, -50.5]),
        ("the hall east", [93.3, 43.6, -50.4]),
    ] {
        w.walk_nav_to(at, name, 1.5);
    }
    w.walk_to(Wp { at: [94.6, 43.6, -47.4], act: Act::Use { use_ticks: 240 }, arrive: 0.7 }, "E the hall door");
    {
        let target = [94.3f32, 43.6, -45.9];
        for t in 0..1200u32 {
            if w.scene.sim.as_ref().unwrap().drones.channels.on(0x48) { break; }
            let sim = w.scene.sim.as_ref().unwrap();
            let p = sim.bond.pos;
            let desired = (target[0] - p[0]).atan2(target[2] - p[2]);
            let err = walker::wrap_pi(desired - sim.bond.yaw);
            let input = Input { turn: err.clamp(-0.08, 0.08), forward: if err.abs() > 1.0 { 0.0 } else { 1.0 }, ..Default::default() };
            w.tick(input, false);
            if t == 1199 { panic!("the hall walk-in plate #412 never fired ch 0x48"); }
        }
    }
    assert!(w.scene.sim.as_ref().unwrap().drones.channels.on(0x48), "the hall walk-in plate #412 fired ch 0x48 [D placements]");
}

/// The control-taking scene freezes the mission clock for its whole 4326-tick window and the
/// hand-back restarts it — through the player's own walk onto the trigger.
#[test]
fn the_control_scene_freezes_the_mission_clock_until_the_hand_back() {
    let iso = iso();
    let mut w = Walker::new(nf_view::load(&iso, "07000007.bin", false).expect("load 07000007"));
    // drone-bump-collision-4: the crowd answer — step around an armed bot the way a player
    // steers (aim beside it, keep walking; the push slides Bond along the capsule). Opt-in on
    // the walker; the arming itself is untouched [M the chain in the anchors' comment].
    w.crowd_aware = true;
    w.skip_cutscenes = false; // watch the real window (the walker's default skips it)

    // Baseline: the clock runs while Bond is in control [M 0x1995a0 — no pause byte set outside
    // the pause menu / a cut-scene; Player_Init's boot re-stamp 0x1910f8:187 is one-time].
    walk_to_the_story_point(&mut w);
    {
        let (e0, e1) = (elapsed(&w), { w.wait(120); elapsed(&w) });
        assert!(e1 > e0, "the mission clock runs before the scene ({e0} → {e1} 1/100 s) [M 0x1995a0:9-14]");
    }

    // The scene takes control: PlayerDisable + the timer pause arrive with its event 4 [D stream
    // 5 @tick 0; M 0x1d94c0:118-121].
    let mut disabled_at = None;
    for _ in 0..600 {
        w.tick(Input::default(), false);
        if w.scene.sim.as_ref().unwrap().disabled { disabled_at = Some(elapsed(&w)); break; }
    }
    let frozen = disabled_at.expect("SP #420's event 4 disabled Bond right after the plate");
    // The clock is FROZEN for the whole window [M the pause byte gates 0x1995a0's accumulator].
    for _ in 0..300 { w.tick(Input::default(), false); }
    assert_eq!(elapsed(&w), frozen,
        "the cut-scene froze the mission clock ({frozen} stayed) [M 0x1d94c0:118-121 + 0x1995a0:9-14 + 0x199520]");
    eprintln!("[07 timer] frozen at {frozen} 1/100 s through 300 ticks of the scene");

    // The hand-back (script tick 4326's event 6 → Script_Free) unpauses the clock
    // [M 0x1da880:44-48]. Poll past the scene's length with margin.
    let mut resumed = None;
    for _ in 0..5500 {
        w.tick(Input::default(), false);
        if !w.scene.sim.as_ref().unwrap().disabled { resumed = Some(elapsed(&w)); break; }
    }
    let resumed = resumed.expect("the hand-back re-enabled Bond before the cap");
    w.wait(120);
    let after = elapsed(&w);
    assert!(after > resumed, "the hand-back restarted the clock ({resumed} → {after}) [M 0x1da880:44-48 + 0x199550]");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.scripts.players.iter()
            .any(|p| p.scripts.contains(&0x0600_0073) && p.state >= 3),
            "the control-taking cut-scene (0x06000073) ran through the real trigger");
        let p = sim.bond.pos;
        assert!((p[0] - 91.52).abs() < 1.0 && (p[2] + 31.09).abs() < 1.0,
            "Bond handed back at the script's marker [D stream 6 key 498 (91.520, 43.851, −31.090)], got {p:?}");
    }
    eprintln!("[07 timer] the scene froze the clock {frozen} → {resumed} and it runs again ({after}) at tick {}", w.total_ticks);
}
