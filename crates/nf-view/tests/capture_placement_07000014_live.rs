//! level-14-capture-placement: after 07000014's capture cut-scene (the ScriptPlayer #382 script
//! 0x0600062F, pack 07900014.bin — the scene whose end sets the "Sabotage base security and
//! escape from room" objective banner via ch 0x48 → the 0x4a enable), the game re-enables the
//! player AT THE SCENE'S END-OF-WALK MARKER ENTITY, not where the trigger was, and not in the
//! void.
//!
//! The game's rule [M]: the scene's stream 11 event 6 (`Script_EventHandler` 0x1d94c0 case 6)
//! snapshots the PREVIOUS stream's entity matrix — stream 10's kind-0x31 hash-0 marker created at
//! t1197 — into SCRIPTINFO+0x40 (`Mat_Copy(prev obj+0x90 → si+0x40)`; `prev` = record(+0x80 +
//! 0xAB3·0x28)+0x10 gated on its kind byte ∈ {1,2}); at scene end `Script_Free` 0x1da880:44-48
//! runs `Player_Enable(glb_players, si+0x40, +0xAB6)` when +0xAA8 has the 0x100 flag, and
//! `Player_Enable` 0x195498 writes the matrix translation into the player object +0x30/+0x40.
//!
//! Where the marker stands [D 07900014.bin NIS 0600062F decode]: stream 10's key t1197 local
//! (-55.4948, 1.5464, 42.0171) — WORLD space once `Script_CameraStart` 0x1d9140 has overwritten
//! the script's base with the identity (the camera keys share the frame: stream 0 t0
//! (-51.2739, 0.5677, 39.6973) → t515 (-55.7541, 0.8911, 40.6857), the control center the walker
//! stands in — the touch #381 volume -51.97…−46.39, 0…2.07, 37.28…39.46 [D placements]). The SP
//! #382 placement compose (-51.5703, 0.0, 31.2664) threw the marker to (-107.0651, 1.5464,
//! 73.2835) — the void; the port then handed Bond back there and he fell to (-107.07, -89.41,
//! 73.28). This test pins the marker (hence the PlayerEnable matrix) at the disc's world-frame
//! key through the real trigger path.
//!
//! The capture scene that follows (SP #407, 0x06000848, pack 07A00014.bin) needs no placement:
//! its event 6 (stream 7, t949, params [0]) has NO object in the previous stream (stream 6 runs
//! only events) [D decode], so the game takes its fallback arm — `Mat_Copy(glb_players+0x90 →
//! si+0x40)` — and Bond resumes where he was [M 0x1d94c0 case 6 iVar5 == 0]. The port's
//! `player_at: None` already matches; asserted here on the same real path.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The kiko scene's marker key translation, world frame [D 07900014.bin NIS 0600062F stream 10
/// key t1197; SP #382 rotation is (0,0,0) [D placement] so the identity base leaves the key].
const MARKER: [f32; 3] = [-55.4948, 1.5464, 42.0171];

#[test]
fn the_capture_scene_hands_bond_back_at_the_scene_marker_not_the_void() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000014.bin", false).expect("load 07000014");
    let sim = scene.sim.as_mut().unwrap();
    assert_eq!(sim.level, 0x0700_0014);
    // Real path stage 1 (the walk's own opener): the vent drop fires the opening scene.
    sim.bond.pos = [40.7, 2.6, 39.0];
    let mut ch47 = false;
    for _ in 0..1700u32 {
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.channels.on(0x47) { ch47 = true; break; }
    }
    assert!(ch47, "the opening scene 0x06000833 must fire on the vent touch [D]");
    for _ in 0..900u32 { sim.step_bond(Input::default(), false, false, false); }
    // Stage 2: the capture scene through its real trigger — TouchOnce #381.
    sim.bond.pos = [-49.2, 0.9, 38.4];
    let mut ch48 = false;
    for _ in 0..1500u32 {
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.channels.on(0x48) { ch48 = true; break; }
    }
    assert!(ch48, "the capture scene 0x0600062F must fire on the touch #381 [D]");
    // Event 6 ran at t1197, two ticks before ch 0x48 (t1199) [D decode]. The game's event-6 arm
    // snapshots the previous stream's entity matrix into si+0x40 (the PlayerEnable return)
    // [M 0x1d94c0 case 6]: the port's `player_at` IS that slot. It must be the marker at the
    // disc's world-frame key.
    let kiko = sim.drones.scripts.players.iter().find(|p| p.scripts[0] == 0x0600_062f)
        .expect("the kiko scene's ScriptPlayer #382");
    let at = kiko.info[kiko.cur].as_ref().and_then(|s| s.player_at);
    let Some(at) = at else { panic!("the capture scene's event 6 must snapshot the marker entity into the return slot (prev stream 10's entity, kind 1 [M 0x1d94c0 case 6])") };
    let d = ((at[3][0] - MARKER[0]).powi(2) + (at[3][1] - MARKER[1]).powi(2) + (at[3][2] - MARKER[2]).powi(2)).sqrt();
    assert!(d < 0.5,
        "the PlayerEnable return matrix must be the scene's end-of-walk marker at the disc's \
         world-frame key {MARKER:?} [D 07900014 stream 10 key t1197] — the SP #382 placement \
         compose put it at ({:.2},{:.2},{:.2}) ({d:.1} off): the void Bond fell from",
        at[3][0], at[3][1], at[3][2]);
    // Control comes back with Bond ON his feet in the room (the sim's own bond keeps the trigger
    // spot: the view's `Bond::stand_at` applies the matrix — the headless sim's position only
    // proves the scene released him).
    let mut back = false;
    for _ in 0..900u32 { sim.step_bond(Input::default(), false, false, false); if !sim.disabled { back = true; break; } }
    assert!(back, "the scene must release Bond (Script_Free's Player_Enable [M 0x1da880])");

    // The capture scene's fallback: fire SP #407 the game's way — the SEQ_AND #445 [0x8C, 0x0C]
    // → ch 9 [D placements]. 0x0C comes from the room touch (Bond is standing in it); 0x8C is the
    // fuseboxes' AND (the sabotage itself is walk_07000014_live's stage 3 pin — this test's target
    // is the scene-end placement, so the channel is written directly).
    sim.drones.channels.set(0x8c, 1, sim.drones.frame);
    sim.bond.pos = [-33.1, 0.9, 26.8];
    let mut ch4a = false;
    for _ in 0..1200u32 {
        sim.step_bond(Input::default(), false, false, false);
        if sim.drones.channels.on(0x4a) { ch4a = true; break; }
    }
    assert!(ch4a, "the escape cut-scene 0x06000848 must write ch 0x4A [D]");
    let escape = sim.drones.scripts.players.iter().find(|p| p.scripts[0] == 0x0600_0848)
        .expect("the escape scene's ScriptPlayer #407");
    let escape_at = escape.info[escape.cur].as_ref().and_then(|s| s.player_at);
    assert!(matches!(escape_at, None),
        "the escape scene's event 6 has no previous-stream object [D 07A00014 stream 7; prev \
         stream 6 runs events only] — the game's fallback keeps the player where he was \
         [M 0x1d94c0 case 6], so the return slot stays empty, got {escape_at:?}");
}
