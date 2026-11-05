//! The movie hold counts 60 Hz ticks of time, whatever the display rate.

use nf_view::movie_polls;

/// 192 ticks is a 3.2 s movie: it takes 3.2 s at 60, 144 and 240 Hz alike. Sister: at 240 Hz the
/// old once-per-frame count ran it out in 0.8 s.
#[test]
fn the_hold_takes_the_same_time_at_any_display_rate() {
    for hz in [60.0f32, 144.0, 240.0, 360.0] {
        let (mut clock, mut ticks, mut frames) = (0.0f32, 0u32, 0u32);
        while ticks < 192 {
            ticks += movie_polls(&mut clock, 1.0 / hz);
            frames += 1;
        }
        let secs = frames as f32 / hz;
        assert!((secs - 3.2).abs() < 0.05, "{hz} Hz took {secs} s");
    }
}

/// The headless harness steps exactly 1/60: one tick per call, never zero or two.
#[test]
fn a_sixty_hertz_step_is_one_tick() {
    let mut clock = 0.0f32;
    for _ in 0..100_000 { assert_eq!(movie_polls(&mut clock, 1.0 / 60.0), 1); }
}
