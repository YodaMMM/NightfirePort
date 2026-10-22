//! unported-front-end-menus-3: numeric tests for the three camera ports — `Camera_Orbit`
//! 0x1bb828 ([M] via `Bond::set_cam_mode_orbit` / `Bond::orbit_camera`), `Camera_Shear`
//! 0x1bba38 and `Camera_ApplySwing` 0x1bbf40 ([M] via `scene::camera_shear_step` /
//! `scene::camera_swing_step`). Numbers only; no disc reads (nothing here is ISO-gated).
//! Every test carries a sister assertion that can fail.

use nf_game::player::Bond;
use nf_view::scene::{camera_shear_step, camera_swing_step};

const TAU: f32 = std::f32::consts::TAU;
/// The per-swing-tick phase step: 2π/(FRAME_RATE·4), FRAME_RATE = 60 [M 0x1bbf40:10-11].
const SWING_STEP: f32 = TAU / 240.0;
/// The per-shear-tick phase step: REC_FRAME_RATE = 1/60 [M 0x1bba38:26-27].
const SHEAR_STEP: f32 = 1.0 / 60.0;

/// `Camera_ApplySwing` 0x1bbf40: one tick advances the phase by 2π/240 and the deltas are
/// (zoom−1)·amount·sin(2φ / φ / 3φ) [M 0x1bbf40:10-24].
#[test]
fn swing_advances_the_phase_and_scales_the_three_sines() {
    let mut phase = 0.0;
    let (p, y, r) = camera_swing_step(&mut phase, 1.0, 2.0);
    // Phase advanced by exactly one step [M 0x1bbf40:10-11].
    assert!((phase - SWING_STEP).abs() < 1.0e-6, "phase {phase}");
    // First tick: φ = 2π/240 — pitch sin(2φ), yaw sin(φ), roll sin(3φ), all × (2−1)·1.
    let expect = |x: f32| nf_game::trig::sinf(x);
    assert!((p - expect(2.0 * phase)).abs() < 1.0e-5, "pitch {p} vs {}", expect(2.0 * phase));
    assert!((y - expect(phase)).abs() < 1.0e-5, "yaw {y}");
    assert!((r - expect(3.0 * phase)).abs() < 1.0e-5, "roll {r}");
}

/// Sister: zero amount. The `!= 0` test at [M 0x1bbf40:8] is on `param_2` = the VIEWER pointer (the
/// call site passes the weapon's swing float first and `glb_viewer[...]` second [M 0x193600:439-441]),
/// so a zero amount still advances the phase; only the deltas are zero (review 2026-10-21 — this
/// test used to pin a frozen phase).
#[test]
fn swing_with_zero_amount_still_advances_the_phase() {
    let mut phase = 0.437;
    let (p, y, r) = camera_swing_step(&mut phase, 0.0, 2.0);
    assert_eq!((p, y, r), (0.0, 0.0, 0.0));
    assert!((phase - (0.437 + SWING_STEP)).abs() < 1.0e-6, "phase {phase}");
}

/// The deltas scale with (zoom − 1) [M 0x1bbf40:12-24]: at zoom 1.0 (unzoomed) they are zero
/// while the phase still advances (the advance is gated only on the viewer).
#[test]
fn swing_at_zoom_one_yields_zero_deltas_but_still_advances() {
    let mut phase = 0.0;
    let (p, y, r) = camera_swing_step(&mut phase, 2.0, 1.0);
    assert_eq!((p, y, r), (0.0, 0.0, 0.0));
    assert!((phase - SWING_STEP).abs() < 1.0e-6, "phase {phase}");
}

/// `Camera_Shear` 0x1bba38: xs = 0.025·sin(φ+π/2), ys = 0.025·sin(φ) [M :20-24], the phase
/// steps 1/60 per frame [M :26-27].
#[test]
fn shear_first_tick_is_the_quarter_phase_amplitudes() {
    let mut phase = 0.0;
    let (xs, ys) = camera_shear_step(&mut phase);
    assert!((xs - 0.025).abs() < 1.0e-5, "xs {xs}"); // sin(π/2) = 1
    assert!(ys.abs() < 1.0e-5, "ys {ys}");           // sin(0) = 0
    assert!((phase - SHEAR_STEP).abs() < 1.0e-6, "phase {phase}");
}

/// Sister: the wrap keeps the phase in [−π, π] [M 0x1bba38:29-34] — a phase just under π
/// steps past π and comes back negative.
#[test]
fn shear_phase_wraps_into_the_minus_pi_pi_band() {
    let mut phase = std::f32::consts::PI - 1.0e-3;
    for _ in 0..3 {
        camera_shear_step(&mut phase);
        assert!(phase.abs() < std::f32::consts::PI + 1.0e-4, "phase {phase} escaped the band");
    }
    assert!(phase < 0.0, "wrapped phase should be negative, got {phase}");
}

/// `Player_SetCamMode(BL, 1)` 0x195928 case 1 [M :17-24]: cam_mode 1 + the CAM_ORBIT init
/// (max 1.5, radius 0.15, target 1.5, phase 0) and the look at the head.
#[test]
fn orbit_init_stamps_the_game_constants() {
    let mut b = Bond::new([12.0, 30.0, -4.0], 0.0);
    b.set_cam_mode_orbit();
    assert_eq!(b.cam_mode, 1);
    let o = b.orbit.as_ref().expect("orbit armed");
    assert_eq!(o.look, b.head());
    assert_eq!((o.max, o.radius, o.target, o.phase), (1.5, 0.15, 1.5, 0.0));
}

/// `Camera_Orbit` 0x1bb828 [M]: tick 1 with no sphere hit — eye = look + (sin(φ)·r,
/// +0.75, cos(φ)·r) with the pre-advance phase and radius [M :29-52], then the phase steps
/// 0.7/60 [M :45] and the radius eases toward the max (no hit → target = max [M :73-74]) by
/// (target − radius)/12 [M :79-82].
#[test]
fn orbit_first_tick_matches_the_decompile_order() {
    let mut b = Bond::new([100.0, 40.0, 50.0], 0.0);
    b.set_cam_mode_orbit();
    let (eye, look, zoom) = b.orbit_camera(None).expect("orbit eye");
    // φ = 0: sin = 0, cos = 1 — eye.x = look.x, eye.y = look.y + 0.75 (r 0.15 < 0.75 floor),
    // eye.z = look.z + 0.15.
    assert!((eye[0] - look[0]).abs() < 1.0e-5, "eye.x {}", eye[0]);
    assert!((eye[1] - (look[1] + 0.75)).abs() < 1.0e-5, "eye.y {}", eye[1]);
    assert!((eye[2] - (look[2] + 0.15)).abs() < 1.0e-5, "eye.z {}", eye[2]);
    assert_eq!(zoom, 1.0); // [M 0x18f5d8:136 — case 1 sets viewer+0x108 = 0x3f800000]
    let o = b.orbit.as_ref().unwrap();
    assert!((o.phase - 0.7 / 60.0).abs() < 1.0e-6, "phase {}", o.phase);
    let want = 0.15 + (1.5 - 0.15) * (1.0 / 60.0) * 5.0;
    assert!((o.radius - want).abs() < 1.0e-6, "radius {} vs {want}", o.radius);
}

/// Sister for the orbit arm: no orbit state (cam_mode 0) → no eye.
#[test]
fn orbit_camera_without_the_mode_arms_returns_none() {
    let mut b = Bond::new([0.0, 30.0, 0.0], 0.0);
    assert!(b.orbit_camera(None).is_none());
    assert_eq!(b.cam_mode, 0);
}

/// The radius ease never crosses the target [M 0x1bb828:79-82 — the clamp fires only when a
/// step would pass the target, so the approach is asymptotic]: easing toward the max
/// converges onto it without ever exceeding, and a wall pull (hit 0.05 → target floors at
/// 0.1 [M :76-78]) walks the radius down without passing it.
#[test]
fn orbit_radius_eases_to_the_target_without_overshoot() {
    let mut b = Bond::new([0.0, 30.0, 0.0], 0.0);
    b.set_cam_mode_orbit();
    let mut max_seen = 0.0f32;
    for _ in 0..400 {
        b.orbit_camera(None).expect("orbit eye");
        max_seen = max_seen.max(b.orbit.as_ref().unwrap().radius);
    }
    let o = b.orbit.as_ref().unwrap();
    assert!((o.radius - 1.5).abs() < 1.0e-4, "radius {} must converge on the max", o.radius);
    assert!(max_seen <= 1.5, "radius {} crossed the max", max_seen);
    // A wall at 0.05 pulls the target to the 0.1 floor.
    b.orbit_camera(Some(0.05)).expect("orbit eye");
    let o = b.orbit.as_ref().unwrap();
    assert_eq!(o.target, 0.1);
    let mut min_seen = 2.0f32;
    for _ in 0..400 {
        b.orbit_camera(Some(0.05)).expect("orbit eye");
        min_seen = min_seen.min(b.orbit.as_ref().unwrap().radius);
    }
    let o = b.orbit.as_ref().unwrap();
    assert!((o.radius - 0.1).abs() < 1.0e-4, "radius {} must converge on the floor", o.radius);
    assert!(min_seen >= 0.1, "radius {} crossed the floor", min_seen);
}
