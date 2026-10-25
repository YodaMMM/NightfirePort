//! fix-interp-camera-bracket: the camera and the palette must interpolate between the SAME two
//! ticks, so the drawn camera must advance by one quarter tick on every presented frame at 240 Hz.
//!
//! The defect this pins (measured in the headless captures, `out/cap_lift_*`): `draw_frame` took
//! `draw_cam` from the bracket as it stood BEFORE the palette rebuild, then the rebuild advanced
//! `cam_prev`/`cam_cur`, then the palette was blended from the NEW pair. On every frame that ran a
//! tick the camera was therefore one whole tick behind the world. Drawn eye y on 0700004a's lift
//! (one tick = -0.0303 u) was -22.72629, -22.76416, -22.77174, -22.77932, -22.75659 ... i.e.
//! 1.25, 0.25, 0.25 then a 0.75-tick BACKWARD jump every fourth frame.
//!
//! `draw_bracket` owns both halves of the bracket (advance, then sample) so this test can drive the
//! production rule without a window or a GPU. No ISO.

use glam::Vec3;
use nf_view::{draw_bracket, Camera};

fn cam(y: f32) -> Camera { Camera { pos: Vec3::new(0.0, y, 0.0), yaw: 0.0, pitch: 0.0, fov: 1.0 } }

/// 8 ticks of a uniform -0.25 u/tick motion, four presented frames per tick at 240 Hz; returns the
/// drawn eye y per frame and the frame's (tick index, fract) so the content time can be checked.
fn run() -> Vec<(f32, usize, f32)> {
    let (mut prev, mut cur) = (None, None);
    let mut out = Vec::new();
    for k in 0..8usize {
        for f in [0.0f32, 0.25, 0.5, 0.75] {
            // The frame that runs the tick has fract 0 and rebuilds the bracket; k = 0 primes it.
            let advanced = f == 0.0 && k > 0;
            let (c, alpha) = draw_bracket(&mut prev, &mut cur, cam(-0.25 * k as f32), advanced, f, true);
            out.push((c.pos.y, k, alpha));
        }
    }
    out
}

#[test]
fn the_drawn_camera_advances_a_quarter_tick_every_frame() {
    let frames = run();
    let steps: Vec<f32> = frames.windows(2).map(|w| w[1].0 - w[0].0).collect();
    // The first tick primes the bracket (no previous tick), so start after it.
    let bad: Vec<f32> = steps[4..].iter().cloned().filter(|d| (d + 0.0625).abs() > 1e-6).collect();
    assert!(bad.is_empty(), "every presented frame must advance the eye by 1/4 tick (-0.0625 u); \
        offending steps {bad:?} of {steps:?}");
}

#[test]
fn the_camera_sits_at_the_palettes_content_time() {
    // The palette is a per-tick array, so the frame's palette content time is (last tick - 1) + fract
    // for the bracket the frame rebuilt. The drawn camera must be at the same time.
    for &(y, k, fract) in run().iter().skip(4) {
        let expect = -0.25 * (k as f32 - 1.0 + fract);
        assert!((y - expect).abs() < 1e-6,
            "tick {k}, fract {fract}: the camera is at {y} but the palette is at {expect} — the camera \
             is not on the palette's bracket");
    }
}

/// SISTER: the pre-fix order — sample the camera from the bracket as it is, THEN advance it — must
/// show the backward jump, so the two assertions above cannot pass by accident.
#[test]
fn the_old_order_shows_the_backward_jump() {
    let (mut prev, mut cur) = (None, None);
    let mut drawn = Vec::new();
    for k in 0..8usize {
        for f in [0.0f32, 0.25, 0.5, 0.75] {
            let advanced = f == 0.0 && k > 0;
            // pre-fix: draw from the OLD bracket at this fract ...
            let pair: (Option<Camera>, Option<Camera>) = (prev, cur);
            let y = match pair {
                (Some(a), Some(b)) => a.pos.y + (b.pos.y - a.pos.y) * f,
                _ => -0.25 * k as f32,
            };
            drawn.push(y);
            // ... and only then advance the bracket (this is where the palette was rebuilt).
            if advanced || cur.is_none() { prev = cur; cur = Some(cam(-0.25 * k as f32)); }
        }
    }
    let steps: Vec<f32> = drawn.windows(2).map(|w| w[1] - w[0]).collect();
    let worst = steps[4..].iter().cloned().fold(0.0f32, |m, d| m.max((d + 0.0625).abs()));
    assert!(worst > 0.15, "the old order must show the one-tick sawtooth (worst step error {worst:.3}); steps {steps:?}");
}
