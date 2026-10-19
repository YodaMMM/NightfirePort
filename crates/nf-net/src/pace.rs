//! net-internet: the online match clock's per-frame tick budget, with stall smoothing.
//!
//! The app runs the online sim on a match clock: tick `t` is due at `t / 60` s after the
//! match started, and the ticks a stall held back must be made up — otherwise that machine
//! falls behind and its inputs (scheduled `delay` ticks ahead of the tick it runs) reach the
//! others late, stalling them in turn. The old budget made the backlog up as fast as the
//! frame cap allowed (≤ 4 ticks in one frame: a 67 ms jump of the world in a single frame
//! — the visible fast-forward after every stall). The pacer keeps on-time frames exactly as
//! before (the frame's own 0–2 ticks run at once) and spreads a backlog: it is made up at a
//! bounded rate over the following frames — 1.25× real time for a short backlog (≤ 20
//! ticks), 2× up to a second behind, and the old frame-cap fast-forward beyond that (a long
//! stall or a late joiner's replay gap must not leave this machine behind for seconds) —
//! never more than `max_per_frame` in one frame.
//!
//! Pure arithmetic on the app's elapsed time (no clock read here; the session layer is
//! outside the Sim, so this never touches hashed state).

/// The per-frame tick budget. One per online match; `reset` at the match start.
#[derive(Clone, Debug, Default)]
pub struct TickPacer {
    /// Fractional ticks of catch-up allowance carried between frames.
    allow: f64,
}

/// A backlog at or below this many ticks is a normal frame (its own ticks run at once).
pub const ON_TIME_TICKS: u32 = 2;
/// Beyond this backlog the catch-up rate doubles …
pub const FAR_BEHIND_TICKS: u32 = 20;
/// … and beyond this one (a second: a long stall, a late joiner's replay gap) it is the old
/// fast-forward — the frame cap every frame — since every tick this machine stays behind
/// delays everyone else's ticks too.
pub const FAST_FORWARD_TICKS: u32 = 60;

impl TickPacer {
    pub fn new() -> TickPacer {
        TickPacer::default()
    }

    pub fn reset(&mut self) {
        self.allow = 0.0;
    }

    /// Ticks to run this frame: `target` is the tick the match clock has reached
    /// (`elapsed · 60`), `net_tick` the next tick to run, `dt_s` this frame's length.
    pub fn ticks_for_frame(&mut self, target: u32, net_tick: u32, dt_s: f64, max_per_frame: u32) -> u32 {
        let backlog = target.saturating_sub(net_tick);
        if backlog <= ON_TIME_TICKS {
            self.allow = 0.0;
            return backlog.min(max_per_frame);
        }
        if backlog > FAST_FORWARD_TICKS {
            self.allow = 0.0;
            return backlog.min(max_per_frame);
        }
        let rate = if backlog > FAR_BEHIND_TICKS { 2.0 } else { 1.25 };
        // The allowance accrues at `rate` ticks per tick of real time; a frame never banks
        // more than one frame cap (a long render hitch must not release a burst).
        self.allow = (self.allow + dt_s.max(0.0) * 60.0 * rate).min(max_per_frame as f64);
        let n = (self.allow.floor() as u32).min(backlog).min(max_per_frame);
        self.allow -= n as f64;
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn on_time_frames_run_their_ticks() {
        let mut p = TickPacer::new();
        assert_eq!(p.ticks_for_frame(10, 9, 1.0 / 60.0, 4), 1);
        assert_eq!(p.ticks_for_frame(12, 10, 1.0 / 30.0, 4), 2);
        assert_eq!(p.ticks_for_frame(12, 12, 1.0 / 240.0, 4), 0);
    }

    #[test]
    fn a_backlog_is_spread_not_burst() {
        // 8 ticks behind at 60 Hz: the old budget ran 4 + 4; the pacer runs 1 or 2 a frame.
        let mut p = TickPacer::new();
        let (mut target, mut tick) = (100u32, 92u32);
        let mut max = 0;
        let mut frames = 0;
        while target.saturating_sub(tick) > ON_TIME_TICKS {
            target += 1; // the clock moves on one tick per 60 Hz frame
            let n = p.ticks_for_frame(target, tick, 1.0 / 60.0, 4);
            max = max.max(n);
            tick += n;
            frames += 1;
            assert!(frames < 100, "the backlog must be made up");
        }
        assert!(max <= 2, "never more than 2 ticks in a 60 Hz frame while catching up: {max}");
        // Sister: the old shape (min(backlog, 4)) bursts 4.
        assert_eq!((100u32 - 92).min(4), 4);
    }

    #[test]
    fn far_behind_catches_up_faster() {
        let mut slow = TickPacer::new();
        let mut fast = TickPacer::new();
        let a: u32 = (0..60).map(|_| slow.ticks_for_frame(1000, 990, 1.0 / 60.0, 4)).sum();
        let b: u32 = (0..60).map(|_| fast.ticks_for_frame(1000, 960, 1.0 / 60.0, 4)).sum();
        assert!(b > a, "{b} > {a}");
        // A second or more behind: the frame cap every frame (the old fast-forward).
        let mut ff = TickPacer::new();
        assert_eq!(ff.ticks_for_frame(1000, 900, 1.0 / 60.0, 4), 4);
    }
}
