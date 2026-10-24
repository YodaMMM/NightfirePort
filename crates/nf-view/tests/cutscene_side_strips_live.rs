//! cutscene-side-strips: during a script-camera frame the whole frame belongs to the cut-scene
//! camera — no draw uses the gameplay camera, and the frame's aspect is the game viewer's own
//! viewer+0x114 (the AV options' widescreen row), not a hard-coded 4:3.
//!
//! The game's evidence:
//! - the cut-scene view builds at the viewer's own aspect: `Script_UpdateCamera` 0x1db518:9
//!   multiplies the view by `viewer+0x114` (the frustum builder `Vision_Calculate_View_Fustrum`
//!   0x1e7df8:79 reads the same word) [M];
//! - `Camera_CalcViewAngles` 0x1baf70:28-32 writes 16/9 (0x3fe38e39) there with the AV options'
//!   widescreen flag `DAT_002a37c4` set, 4/3 (0x3faaaaab) otherwise [M].
//!
//! So with the option ON the game's cut-scene is a full 16/9 picture — the port's pillar-boxed
//! 4:3 centre (fix-cutscene-letterbox's option-off shape) left the sides black where the game
//! shows the widened cut-scene. The port's own gameplay widescreen treatment is the widened
//! full-window view, and the cut-scene now matches it (`cutscene_frame`'s aspect argument),
//! while the gameplay camera itself never draws during a script camera.
//!
//! Gates (all through the real input path — `nf_view::load` + `Sim::step_bond` from the repro
//! spot; the shot renders named in the report carry the pixel side):
//! 1. the draw camera the frame feeds is the SCRIPT camera — the CLI walk's rule (lib.rs: the
//!    Bond-head camera replaced by `script_camera()`), checked at live-cutscene ticks;
//! 2. the gameplay HUD has no draws while the script camera runs (`overlay_for` empty);
//! 3. the frame: option off → the 4:3 pillar-box band; option on → the full widened frame at
//!    16/9, same vertical FOV.
//!
//! Opt-in via NF_ISO.

use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The repro spot (the task's command line): the terrace over the courtyard, level 07000004.
const SPOT: ([f32; 3], f32, f32) = ([-6.12, 5.44, 36.24], -0.402, -0.005);

/// Walks the level from the repro spot and returns (scene, first live script-camera tick, the
/// script camera at that tick, Bond's head camera there) — the same walk the `--frame` shot runs.
fn walk_to_cutscene() -> Option<(nf_view::scene::Scene, u32, [f32; 3], f32, f32, f32, [f32; 3])> {
    let iso = iso()?;
    let mut scene = nf_view::load(&iso, "07000004.bin", false).expect("load 07000004");
    {
        let sim = scene.sim.as_mut().unwrap();
        let (p, yaw, pitch) = SPOT;
        sim.bond = nf_game::player::Bond::new(p, yaw);
        sim.bond.pitch = pitch;
    }
    for t in 1..=1400u32 {
        scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        if let Some((p, yaw, pitch, fov)) = sim.script_camera() {
            let h = sim.bond.head();
            return Some((scene, t, p, yaw, pitch, fov, h));
        }
    }
    panic!("07000004 never runs a script camera from the repro spot in 1400 ticks")
}

#[test]
fn the_frame_camera_is_the_script_camera_never_bonds() {
    let Some((_scene, t, p, _yaw, _pitch, _fov, head)) = walk_to_cutscene() else { eprintln!("no NF_ISO — skipping"); return };
    // The script camera is live and it is NOT Bond's head camera: the walk's draw rule (lib.rs —
    // the Bond-head camera, then the `script_camera()` replacement) feeds the frame the script
    // camera from the first live tick on. The repro's strips complaint can only come from a
    // frame where this does not hold.
    assert!((p[0] - head[0]).abs() > 1e-3 || (p[1] - head[1]).abs() > 1e-3 || (p[2] - head[2]).abs() > 1e-3,
        "tick {t}: the script camera sits on Bond's head — the frame would be the gameplay camera");
}

#[test]
fn the_gameplay_hud_has_no_draws_during_a_script_camera() {
    let Some((scene, t, _, _, _, _, _)) = walk_to_cutscene() else { eprintln!("no NF_ISO — skipping"); return };
    let frame = nf_view::overlay_for(&scene, 4.0 / 3.0);
    assert!(frame.v.is_empty(), "tick {t}: the gameplay HUD drew {} vertices during a script camera", frame.v.len());
    assert!(frame.runs.is_empty(), "tick {t}: the gameplay HUD has {} runs during a script camera", frame.runs.len());
}

/// (3) The frame. Option off (the game's default): the 4:3 band, pillar-boxed on a wide window.
/// Option on (`Sim::view_aspect` = the viewer+0x114 value [M 0x1baf70:28-32]): the cut-scene
/// camera drives the WHOLE 16:9 frame at 16/9 — the widened view at the same vertical FOV, the
/// port's gameplay widescreen treatment. On a 4:3 window the 16/9 option keeps the full frame
/// too (the game's anamorphic squeeze on a 4:3 TV — the aspect still 16/9).
#[test]
fn the_cutscene_frame_follows_the_viewers_widescreen_row() {
    // Off — fix-cutscene-letterbox's shape, unchanged.
    let (aspect, band) = nf_view::cutscene_frame(1280.0, 720.0, 4.0 / 3.0);
    assert!((aspect - 4.0 / 3.0).abs() < 1e-6);
    assert_eq!(band, [160.0, 0.0, 960.0, 720.0], "option off: the 4:3 pillar-box band");
    // On — the widened full frame (this run was the black side strips before the fix).
    let (aspect, band) = nf_view::cutscene_frame(1280.0, 720.0, 16.0 / 9.0);
    assert!((aspect - 16.0 / 9.0).abs() < 1e-6, "the cut-scene projects at the viewer's 16/9, got {aspect}");
    assert_eq!(band, [0.0, 0.0, 1280.0, 720.0], "option on: the cut-scene camera drives the whole frame");
    // The same vertical FOV: both bands own the FULL window height (the projection's f row is
    // aspect-independent — the widened frame sees more at the sides, never less at top/bottom).
    let (_, band43) = nf_view::cutscene_frame(1280.0, 720.0, 4.0 / 3.0);
    assert_eq!(band43[1], 0.0);
    assert_eq!(band43[3], 720.0);
    assert_eq!(band[1], 0.0);
    assert_eq!(band[3], 720.0);
    // On a 4:3 window the option still owns the projection (the anamorphic case): the 16/9
    // picture fits the window letter-boxed (top/bottom bands), aspect 16/9.
    let (aspect, band) = nf_view::cutscene_frame(1024.0, 768.0, 16.0 / 9.0);
    assert!((aspect - 16.0 / 9.0).abs() < 1e-6, "the projection stays the viewer's 16/9, got {aspect}");
    assert!((band[0]).abs() < 1e-3 && (band[2] - 1024.0).abs() < 1e-3, "full width, got {band:?}");
    assert!((band[1] - 96.0).abs() < 1e-2 && (band[3] - 576.0).abs() < 1e-2, "letter-boxed height, got {band:?}");
}
