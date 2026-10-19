//! level-13 gate: the front entrance's security gate raises through the real E at the shutter
//! panel, and the risen gate stops blocking the capsule band at the entrance mouth.
//!
//! The gate is the "Shutters2" SimpleScript #149 (trigger ch 0x23 [D param 3]) whose cel spans
//! the entrance mouth (x −8.54..−4.97 at z 65.02 [D the object's glist bounds]) — the wall the
//! walkreal route pinned at. Its writer is ScriptPlayer #161 ("SCRIPTPLAYER_Elevators", mode 64
//! preload, mask 4, out ch 0x23 [D params (5,4),(6,23)]): the preload's Entity op creates
//! 0x02000A2C "SCRIPT_SHUTTER_CLOSE" — the wall panel's world-baked glist at (−18.08, −368.23,
//! 25.01) [D obj 70; the OPEN twin 0x02000A2B arrives at the script's t=16] — carrying obj+0xE4 =
//! the SP handle [M Script_EntityStart 0x1d8d78:110], so the E pick's class-0 arm reaches
//! SP_Activate [M Player_Activate 0x1a5170:121-122] and the played script writes ch 0x23
//! [M SP_Update 0x1dbda8:221-229]. ch 0x23 runs the SS playhead to key 1 [M SS_Update 0x1a9948 +
//! KeyFrame_Interp 0x1e1140] and the posed slab's collision follows [M the set_dynamic pass].
//!
//! The E stands at the port's entity sphere (the model centre transformed by the SP's stream
//! matrix, t −1.41 x — the game-side displacement of a world-baked glist is the report's live
//! check). The reach-only teleport crosses a room cell, so the collision response is asserted
//! with UNSCOPED segment queries (segment_hit_scoped(None, …)) instead of a walk — the walkreal
//! route's cell-stickiness blocker is documented in the report.

#[path = "common/walker_b2.rs"]
mod walker;

use walker::Walker;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

#[test]
fn the_security_gate_raises_on_the_panel_e_and_stops_blocking_the_mouth() {
    let mut w = Walker::new(nf_view::load(&iso(), "07000013.bin", false).expect("load 07000013"));
    w.wait_enabled(3000, "the intro");
    w.wait(30);
    // The capsule-height band across the mouth: the DOWN gate blocks it.
    let seg = |w: &Walker| w.scene.sim.as_ref().unwrap().world
        .segment_hit_scoped(None, [-6.2f32, -368.0, 58.0], [-6.2, -368.0, 67.0]);
    {
        let hit = seg(&w);
        assert!(hit.is_some(), "the down gate blocks the capsule band at the mouth [D the SS cel at z 65.02]");
        let (t, _n) = hit.unwrap();
        let z = 58.0 + t * 9.0;
        assert!(z > 64.0 && z < 66.5, "the block is the gate slab (z ≈ 65), got {z:.2}");
        assert!(!w.scene.sim.as_ref().unwrap().drones.channels.on(0x23), "ch 0x23 off before the panel [D]");
    }
    // The E at the panel (the reach-only teleport; the press is the real use key).
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = [-19.49, -369.1, 23.9];
        sim.bond.yaw = 0.0;
        sim.bond.pitch = 0.0;
        sim.bond.vel = [0.0; 3];
    }
    let mut ch23 = false;
    for _ in 0..80 {
        w.tick(nf_game::player::Input::default(), true);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x23) { ch23 = true; break; }
    }
    assert!(ch23, "E at the shutter panel writes ch 0x23 [M 0x1dbda8:221-229; D SP #161 out]");
    // The playhead runs the panel up (~4 units over ~200 ticks [probe]).
    let mut risen = false;
    for _ in 0..400 {
        w.tick(nf_game::player::Input::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        let y = sim.drones.sss.list.iter().find(|s| s.channel == 0x23).map(|s| s.matrix[3][1]).unwrap_or(0.0);
        if y > 3.0 { risen = true; break; }
    }
    assert!(risen, "the gate SS animates to key 1 once ch 0x23 is on [M SS_Update 0x1a9948]");
    // The risen gate stops blocking the capsule band.
    for _ in 0..120 { w.tick(nf_game::player::Input::default(), false); }
    assert!(seg(&w).is_none(), "the risen gate clears the capsule band at the mouth [M the posed slab's collision follows]");
    // Sister: a band ABOVE the risen panel's new bottom still... the panel (y −369.4..−364.3 at
    // rest, risen ~4) now blocks y −365.4..−360.3 — the raised band reads as solid.
    let hi = w.scene.sim.as_ref().unwrap().world
        .segment_hit_scoped(None, [-6.2f32, -362.5, 58.0], [-6.2, -362.5, 67.0]);
    assert!(hi.is_some(), "the risen slab still blocks its raised band [M set_dynamic's posed geo]");
}
