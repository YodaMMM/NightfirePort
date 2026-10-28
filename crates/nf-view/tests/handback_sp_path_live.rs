//! cutscene-handback-sp-path: the kind-2 (anim) previous-stream event-6 hand-back through the
//! REAL SP runtime path — `Scripts::update_player` (the game's SP_Update 0x1dbda8 control walk),
//! not the census's raw `Script::update`. The census (nis_handback_census.rs) pins the resolution
//! through a bare `Script::load`+`update` loop; the real-path divergence this task closes is that
//! the same scenes on the runtime path lost the anim object (or its pose data) before the event:
//! the pack anim sets are now merged into the runtime `Data` (the loader's 078/079/07A-000NN
//! sweep, [M LoadableLoad 0x1d69e8]) and the event-6 arm snapshots `Obj::Anim` too [M
//! 0x1d94c0:73-84, 125-134]. Each scene here plays with the player's own per-tick step
//! (`Sim::step_bond`, no skips) and asserts Bond's position at the hand-back against the census's
//! resolution — each spot standing on a floor [D the levels' static collision, the census rows]:
//!   07000002 plc#400 (intro, script 0x06000084) → (-27.50, 1.16, 32.36), floor 0.05
//!   07000002 plc#399 (mid-level, script 0x0600007f) → (-45.56, 5.92, 8.15), floor 4.75
//!   07000003 plc#475 (mid-level, script 0x06000080) → (15.09, 5.55, -12.67), floor 4.41
//! Test setup (not asserted as game behaviour): the mid-level scenes' own start gates stand
//! behind whole-objective chains (07000002: SEQ_AND(0x3B rescue, 0x3C touch) → 0x35 [D tr[033];
//! 07000003: the mask-4 activation the E-use path owns, SP_Activate 0x1dc338's only caller being
//! Player_Activate 0x1a5170 [M]) — the tests write the SP's own start condition directly (the
//! ch-0x35 out-channel bit [M 0x1dbda8 flags bit 1]; the +0x2c activated byte [M 0x1dc338 tail])
//! and let everything below it — SP_Update's case-0 load, Script_Play, the script's events, the
//! event-6 snapshot and Script_Free's Player_Enable — run for real.
use nf_game::player::{Bond, Input};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// Steps the sim until the player has been disabled by a scene and handed back (the game's
/// event-4 Player_Disable [M 0x1d94c0:118-121] then Script_Free's Player_Enable [M 0x1da880:44-48]),
/// returning the tick. `pre` ticks run first (a scene that starts later than the load).
fn play_one_scene(sim: &mut nf_view::scene::Sim, pre: u32, cap: u32) -> Option<u32> {
    let (mut ever_disabled, mut handed) = (false, None);
    for t in 0..cap {
        sim.step_bond(Input::default(), false, false, false);
        if t < pre { continue; }
        if sim.disabled { ever_disabled = true; }
        if ever_disabled && !sim.disabled && sim.script_camera().is_none() { handed = Some(t); break; }
    }
    handed
}

#[test]
fn the_intro_scene_hands_back_at_the_anim_root() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000002.bin", false).expect("load 07000002");
    let sim = scene.sim.as_mut().unwrap();
    // The level-start scene (SP #400, script 0x06000084 [D placement]): the walker's route waits
    // for its ch-0x47 write at the scene's tick 1539 [D + L NF_SCRIPTLOG], so the scene is live
    // from the load and runs ~1600 ticks.
    let t = play_one_scene(sim, 0, 12000).expect("the intro cut-scene disables and re-enables the player within 12000 ticks");
    let p = sim.bond.pos;
    // The census's 0x06000084 row: the kind-2 snapshot lands at (-27.50, 1.16, 32.36), floor 0.05
    // below [D]; the enable's `Bond::stand_at` grounds to that floor (+1.1). x/z tight, y vs the
    // census translation (floor + root agree within 0.08 there).
    assert!((p[0] - -27.50).abs() < 0.6 && (p[2] - 32.36).abs() < 0.6,
        "07000002 intro (script 0x06000084, tick {t}): Bond handed back at {p:?}, the census's kind-2 resolution is (-27.50, ?, 32.36) [M 0x1d94c0:125-134]");
    assert!((p[1] - 1.16).abs() < 0.9, "07000002 intro: hand-back y {} vs the census's 1.16 (floor 0.05 + 1.1)", p[1]);
}

#[test]
fn the_midlevel_scene_hands_back_at_the_anim_root() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000002.bin", false).expect("load 07000002");
    let sim = scene.sim.as_mut().unwrap();
    // Let the level's own intro scene finish first (it holds the same disabled window).
    if play_one_scene(sim, 0, 12000).is_none() { eprintln!("[02] no intro scene ran; continuing"); }
    // SP #399 (script 0x0600007f [D placement], mask 1, out 0x35 [D]): drive the chain's ROOTS —
    // AND #397's inputs (the captor deaths 0x29/0x2A/0x88 + the rescue 0x49 [D tr[017]]) poked,
    // and 0x3C left to its REAL writer: the repeating bathhouse touch #495 (out 0x3C [D audit],
    // the geisha-1 zone (−45.4, 5.74, 11.7) the walkreal route crosses) — Bond stands in it. The
    // real mux machinery then writes the chain itself: AND #397 → 0x3B, SEQ #496 (tr[033]) → 0x35,
    // the SP's own start channel [M Trigger_Update 0x1b1a78's AND case 3 / SEQ_AND case 4 re-write
    // their out channels from the inputs every tick — a direct ch-0x35 poke is cleared before the
    // SP's update reads it]. The death/rescue pokes stand in for the whole-objective route the
    // walkreal tests play (test setup, not asserted as game behaviour); the touch, both muxes and
    // everything from the SEQ's 0x35 write on is the real path.
    sim.bond = Bond::new([-45.4, 5.74, 11.7], 0.0);
    let frame = sim.drones.frame;
    for c in [0x29, 0x2a, 0x88, 0x49] { sim.drones.channels.set(c, 1, frame); }
    let t = play_one_scene(sim, 0, 12000).expect("SP #399 plays and hands back within 12000 ticks of the chain poke");
    let p = sim.bond.pos;
    // The census's 0x0600007f row: (-45.56, 5.92, 8.15), floor 4.75 below [D].
    assert!((p[0] - -45.56).abs() < 0.6 && (p[2] - 8.15).abs() < 0.6,
        "07000002 SP #399 (script 0x0600007f, tick {t}): Bond handed back at {p:?}, the census's kind-2 resolution is (-45.56, ?, 8.15) [M 0x1d94c0:125-134]");
    assert!((p[1] - 5.92).abs() < 0.9, "SP #399: hand-back y {} vs the census's 5.92 (floor 4.75 + 1.1)", p[1]);
}

#[test]
fn the_dragon_wedge_scene_hands_back_at_the_anim_root() {
    let iso = iso();
    let mut scene = nf_view::load(&iso, "07000003.bin", false).expect("load 07000003");
    let sim = scene.sim.as_mut().unwrap();
    if play_one_scene(sim, 0, 12000).is_none() { eprintln!("[03] no intro scene ran; continuing"); }
    // SP #475 (script 0x06000080 [D audit]; the runtime parse: mask 1, out 0x36 — the audit's
    // "mask 4" line disagrees with the port's param read, the parse wins): write its out channel.
    // The writers are the two TouchOnce plates #416/#417 (out 0x36, lock p7=0x3A [D audit]) — a
    // TOUCH_ONCE writes its out only on hit and never clears it, so the poke stands in for the
    // plate touch (test setup); the SP start below it is the real path [M 0x1dbda8 flags bit 1].
    sim.drones.channels.set(0x36, 1, sim.drones.frame);
    let t = play_one_scene(sim, 0, 12000).expect("SP #475 plays and hands back within 12000 ticks of the ch-0x36 write");
    let p = sim.bond.pos;
    // The census's 0x06000080 row: (15.09, 5.55, -12.67), floor 4.41 below [D].
    assert!((p[0] - 15.09).abs() < 0.6 && (p[2] - -12.67).abs() < 0.6,
        "07000003 SP #475 (script 0x06000080, tick {t}): Bond handed back at {p:?}, the census's kind-2 resolution is (15.09, ?, -12.67) [M 0x1d94c0:125-134]");
    assert!((p[1] - 5.55).abs() < 0.9, "SP #475: hand-back y {} vs the census's 5.55 (floor 4.41 + 1.1)", p[1]);
}
