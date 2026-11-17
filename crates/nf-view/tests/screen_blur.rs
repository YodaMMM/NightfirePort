//! The screen blur (`psiSetScreenBlur` 0x11ca98 / `CreateSqueezeFrame` 0x11b140 / `Game_Draw`
//! 0x1e6f50:65-71): the decay, the keep share and its frame-rate conversion.
use nf_view::gpu::blur_keep_for_frame;
use nf_view::scene::{screen_blur_decay, screen_blur_keep};

#[test]
fn decay_steps_by_0x10_then_zero() {
    // [M 0x1e6f50:66-71] below 0x10 → 0, else −0x10.
    assert_eq!(screen_blur_decay(0x7f), 0x6f);
    assert_eq!(screen_blur_decay(0x10), 0x00);
    assert_eq!(screen_blur_decay(0x0f), 0x00);
    assert_eq!(screen_blur_decay(0), 0);
    // The zoom pulse (0x7f) lasts 8 game frames: 7f 6f 5f 4f 3f 2f 1f 0f → 0.
    let mut b = 0x7fu8;
    let mut n = 0;
    while b != 0 { b = screen_blur_decay(b); n += 1; }
    assert_eq!(n, 8);
    // Sister: a wrong decay (−8) would take 16 frames.
    assert_ne!(n, 16);
}

#[test]
fn keep_is_fix_complement() {
    // FIX = 0x80 − (blur >> 2); kept share = (blur >> 2)/128 [M 0x11ca98, packet 0x11b140].
    assert_eq!(screen_blur_keep(0), 0.0);
    assert_eq!(screen_blur_keep(0x7f), 31.0 / 128.0); // zoom
    assert_eq!(screen_blur_keep(0x91), 36.0 / 128.0); // underwater
    assert_eq!(screen_blur_keep(0xff), 63.0 / 128.0);
    assert_ne!(screen_blur_keep(0x91), 0x91 as f32 / 255.0);
}

#[test]
fn keep_matches_per_second_at_any_rate() {
    let k60 = screen_blur_keep(0x91);
    // One 60 Hz frame keeps exactly k60; four 240 Hz frames keep the same in total.
    assert!((blur_keep_for_frame(k60, 1.0 / 60.0) - k60).abs() < 1e-6);
    let k240 = blur_keep_for_frame(k60, 1.0 / 240.0);
    assert!((k240.powi(4) - k60).abs() < 1e-5);
    // Sister: using k60 unchanged at 240 Hz would fade 4× slower.
    assert!((k60.powi(4) - k60).abs() > 0.1);
    assert_eq!(blur_keep_for_frame(0.0, 1.0 / 240.0), 0.0);
}
