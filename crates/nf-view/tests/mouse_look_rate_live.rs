//! Mouse look rate vs the render rate: the same raw mouse counts per wall-clock second must turn
//! Bond (and the drawn view) by the same angle at 60 Hz and at 240 Hz (vsync at the display rate).
//!
//! The app's path, driven headless the way the winit loop runs it:
//! - each frame's raw `DeviceEvent::MouseMotion` deltas go through `nf_view::mouse_look_add` into
//!   the pending turn/look (lib.rs `device_event`);
//! - `nf_view::ticks_for_frame` schedules the frame's fixed-60 Hz ticks (the loop's arithmetic);
//! - each tick takes the WHOLE pending turn/look once (`std::mem::take`, the tick loop's Input
//!   build) into `Sim::step_bond`;
//! - the drawn camera comes from `nf_view::draw_bracket` over the tick cameras.
//!
//! The pinned rate is `MOUSE_SENSITIVITY` = 0.0025 rad per count at zoom 1 (unchanged since before
//! v0.1.22). Sister: a per-frame add of the pending delta (the delta applied on every rendered
//! frame instead of being taken once per tick) multiplies the rate by frames-per-tick — 4x at
//! 240 Hz — and the same comparison catches it.
//!
//! The ISO half (07000006) is opt-in through NF_ISO; the arithmetic half needs nothing.

use nf_view::{draw_bracket, mouse_look_add, ticks_for_frame, Camera, MOUSE_SENSITIVITY};

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// Shortest signed angle a → b.
fn wrap(d: f32) -> f32 {
    let t = std::f32::consts::TAU;
    let mut d = d % t;
    if d > std::f32::consts::PI { d -= t; }
    if d < -std::f32::consts::PI { d += t; }
    d
}

/// Counts per second fed in every run: integer counts per frame at 60, 120 and 240 Hz.
const DX_PER_SEC: i32 = 960;
const DY_PER_SEC: i32 = 240;

/// How the pending delta reaches the sim.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Feed {
    /// The app's path: taken once per tick (`std::mem::take`).
    TakePerTick,
    /// The regressed shape the sister models: the pending delta applied on every rendered frame
    /// that runs a tick AND every frame that does not (not cleared until a tick).
    AddPerFrame,
}

/// Arithmetic-only loop (no sim): returns the total turn and look the "sim" received.
fn arith_loop(fps: u32, feed: Feed) -> (f32, f32) {
    let dt = 1.0 / fps as f32;
    let (dxf, dyf) = (DX_PER_SEC / fps as i32, DY_PER_SEC / fps as i32);
    let mut anim_time = 0.0f32;
    let (mut turn, mut look) = (0.0f32, 0.0f32);
    let (mut got_t, mut got_l) = (0.0f32, 0.0f32);
    // One second of mouse, then idle frames until the pending delta drains.
    for f in 0..fps + fps / 30 + 2 {
        if f < fps { mouse_look_add(&mut turn, &mut look, dxf as f64, dyf as f64, 1.0); }
        let n = ticks_for_frame(&mut anim_time, dt, 4);
        match feed {
            Feed::TakePerTick => for _ in 0..n {
                got_t += std::mem::take(&mut turn);
                got_l += std::mem::take(&mut look);
            },
            Feed::AddPerFrame => {
                got_t += turn;
                got_l += look;
                if n > 0 { turn = 0.0; look = 0.0; }
            }
        }
    }
    (got_t, got_l)
}

#[test]
fn look_rate_is_the_same_at_60_and_240_hz() {
    let want_t = -(DX_PER_SEC as f32) * MOUSE_SENSITIVITY;
    let want_l = -(DY_PER_SEC as f32) * MOUSE_SENSITIVITY / std::f32::consts::FRAC_PI_2;
    for fps in [60, 120, 240] {
        let (t, l) = arith_loop(fps, Feed::TakePerTick);
        assert!((t - want_t).abs() < 1e-4, "{fps} Hz: turn {t} vs {want_t} (0.0025 rad/count)");
        assert!((l - want_l).abs() < 1e-4, "{fps} Hz: look {l} vs {want_l}");
    }
    // The pinned constant itself (the v0.1.23 value).
    assert_eq!(MOUSE_SENSITIVITY, 0.0025);
}

/// Sister: the per-frame add makes 240 Hz turn ~4x as far as 60 Hz — the comparison above fails it.
#[test]
fn sister_per_frame_add_scales_with_the_frame_rate() {
    let (t60, _) = arith_loop(60, Feed::AddPerFrame);
    let (t240, _) = arith_loop(240, Feed::AddPerFrame);
    let want = -(DX_PER_SEC as f32) * MOUSE_SENSITIVITY;
    assert!((t60 - want).abs() < 1e-4, "at 60 Hz every frame is a tick, both feeds agree: {t60}");
    let ratio = t240 / t60;
    assert!(ratio > 2.0, "the per-frame add must over-turn at 240 Hz (ratio {ratio})");
    assert!((t240 - want).abs() > 1.0, "the take-per-tick tolerance must reject it: {t240} vs {want}");
}

/// The real sim: one second of the same mouse counts at 60 and at 240 Hz through step_bond and
/// the camera bracket. Bond's yaw/pitch and the drawn camera's yaw/pitch must move by the same
/// angle at both rates, equal to counts × 0.0025.
fn sim_loop(iso: &str, fps: u32) -> (f32, f32, f32, f32) {
    let mut scene = nf_view::load(iso, "07000006.bin", false).expect("load 07000006");
    for _ in 0..7200 {
        if scene.sim.as_ref().map_or(false, |s| !s.disabled) { break; }
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
    }
    // Settle on the ground so the start pose is still.
    for _ in 0..30 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let sim = scene.sim.as_mut().unwrap();
    assert!(!sim.disabled, "07000006 never enabled");
    let (yaw0, pitch0) = (sim.bond.yaw, sim.bond.pitch);
    let cam_of = |s: &nf_view::scene::Sim| {
        let h = s.bond.head();
        Camera { pos: glam::Vec3::new(h[0], h[1], h[2]), yaw: s.bond.yaw, pitch: s.bond.pitch * std::f32::consts::FRAC_PI_2, fov: 1.0 }
    };
    let dt = 1.0 / fps as f32;
    let (dxf, dyf) = (DX_PER_SEC / fps as i32, DY_PER_SEC / fps as i32);
    let (mut anim_time, mut turn, mut look) = (0.0f32, 0.0f32, 0.0f32);
    let (mut cam_prev, mut cam_cur): (Option<Camera>, Option<Camera>) = (None, None);
    let mut cam = cam_of(sim);
    let mut drawn = cam;
    for f in 0..fps + fps / 4 {
        if f < fps { mouse_look_add(&mut turn, &mut look, dxf as f64, dyf as f64, sim.zoom); }
        let n = ticks_for_frame(&mut anim_time, dt, 4);
        for _ in 0..n {
            let input = nf_game::player::Input { turn: std::mem::take(&mut turn), look: std::mem::take(&mut look), ..Default::default() };
            sim.step_bond(input, false, false, false);
            cam = cam_of(sim);
        }
        drawn = draw_bracket(&mut cam_prev, &mut cam_cur, cam, n > 0, anim_time.fract(), true).0;
    }
    (wrap(sim.bond.yaw - yaw0), (sim.bond.pitch - pitch0) * std::f32::consts::FRAC_PI_2, wrap(drawn.yaw - yaw0), drawn.pitch - pitch0 * std::f32::consts::FRAC_PI_2)
}

#[test]
fn bond_and_the_drawn_view_turn_the_same_at_60_and_240_hz() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let a = sim_loop(&iso, 60);
    let b = sim_loop(&iso, 240);
    let want_yaw = -(DX_PER_SEC as f32) * MOUSE_SENSITIVITY;
    let want_pitch = -(DY_PER_SEC as f32) * MOUSE_SENSITIVITY;
    eprintln!("60 Hz: yaw {:.5} pitch {:.5} drawn yaw {:.5} drawn pitch {:.5}", a.0, a.1, a.2, a.3);
    eprintln!("240 Hz: yaw {:.5} pitch {:.5} drawn yaw {:.5} drawn pitch {:.5}", b.0, b.1, b.2, b.3);
    for (hz, r) in [(60, a), (240, b)] {
        assert!((r.0 - want_yaw).abs() < 1e-3, "{hz} Hz: Bond's yaw moved {} (want {want_yaw})", r.0);
        assert!((r.1 - want_pitch).abs() < 1e-3, "{hz} Hz: Bond's pitch moved {} rad (want {want_pitch})", r.1);
        assert!((r.2 - want_yaw).abs() < 1e-3, "{hz} Hz: the drawn view's yaw moved {} (want {want_yaw})", r.2);
        assert!((r.3 - want_pitch).abs() < 1e-3, "{hz} Hz: the drawn view's pitch moved {} (want {want_pitch})", r.3);
    }
    assert!((a.0 - b.0).abs() < 1e-3 && (a.2 - b.2).abs() < 1e-3, "60 vs 240 Hz must turn the same: {a:?} vs {b:?}");
}
