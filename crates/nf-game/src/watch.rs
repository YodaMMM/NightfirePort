//! `Player_SetWatch` 0x1a69d0 (1244 bytes) — the wristwatch's clock DATA, the sim-visible part [M].
//!
//! Called every tick as the last statement of `Player_CollisionHandler` 0x1949a8:204 [M]. The
//! function is the wrist-watch's attach/detach driver for the first-person view model; the
//! attach/detach half is bone/render work (draw side). The DATA the draw code needs —
//! the mission-clock base per level and the hand angles — lives here:
//!
//! - The clock reads `t = DAT_002bcc50 + level_base` [M 0x1a69d0:89]: the mission elapsed timer
//!   (centiseconds — the same timer `PlrStat_GetScore` 0x1984b0 reads) plus a per-level base from
//!   the switch on `GameState+0x10` [M :39-78].
//! - Each hand's angle = `fmod(t · rate, 2π)`, applied as `RotMatrixX(−ang − π/2)` [M :105-137]:
//!   hours 1.4544411e-06 rad/cs (one revolution / 12 h), minutes 1.7453292e-05 (one revolution /
//!   hour), seconds 0.0010471976 (one revolution / 60 s) [M :90-104].
//! - Attach gate: the fp view-model's skin id (`AnimObject+0x94 → +0`) == 0x0500004b [M :79]
//!   — the shared fp model of weapons 0x38/0x4E/0x4F/0x5A/0x5C [D weapon-model-coverage.md:78].
//!   Detach (skin differs): each hand obj gets +0xFE |= 1 (delete-me) and its global is zeroed
//!   [M :139-152]. MP gate: `MPSettings+0x384 == 0` (SP only) [M :36-38].
//!
//! The base table ([M :39-78]; GameState+0x10 = the level id):
//! | levels | base (cs) | clock |
//! |---|---|---|
//! | 0x7000001-04 | 3.24e6 | 09:00 |
//! | 0x7000005-08 | 7.02e6 | 19:30 |
//! | 0x7000009-0B | 8.1e6 | 22:30 |
//! | 0x700000C-0D | 2.88e6 | 08:00 |
//! | 0x7000011-13, 0x700004A | 2.16e6 | 06:00 |
//! | 0x7000014-16 | 1.44e6 | 04:00 |
//! | others | 0 | 00:00 |

/// The per-level mission-clock base, in centiseconds ([M 0x1a69d0:39-78] switch).
pub fn level_clock_base(level: u32) -> f32 {
    match level {
        0x07000001..=0x07000004 => 3_240_000.0,
        0x07000005..=0x07000008 => 7_020_000.0,
        0x07000009..=0x0700000b => 8_100_000.0,
        0x0700000c | 0x0700000d => 2_880_000.0,
        0x07000011..=0x07000013 | 0x0700004a => 2_160_000.0,
        0x07000014..=0x07000016 => 1_440_000.0,
        _ => 0.0,
    }
}

/// The hand angular rates, radians per centisecond ([M 0x1a69d0:90-104] f32 literals).
pub const HOURS_RATE: f32 = 1.4544411e-06;
pub const MINUTES_RATE: f32 = 1.7453292e-05;
pub const SECONDS_RATE: f32 = 0.0010471976;

/// One hand's angle: `fmod(t · rate, 2π)` [M 0x1a69d0:105-137] — the caller applies
/// `RotMatrixX(−ang − π/2)` (the bone-space mount).
pub fn hand_angle(t_cs: f32, rate: f32) -> f32 {
    let t = t_cs * rate;
    let a = t % std::f32::consts::TAU;
    if a < 0.0 { a + std::f32::consts::TAU } else { a }
}

/// The watch face's wall-clock readout for a mission time (the draw-side label helper):
/// (hours, minutes, seconds) as integers, from `t = base + elapsed` centiseconds.
pub fn readout(base_cs: f32, elapsed_cs: f32) -> (u32, u32, u32) {
    let total_s = ((base_cs + elapsed_cs) / 100.0).round() as i64;
    let s = ((total_s % 60) + 60) % 60;
    let m = ((total_s / 60) % 60 + 60) % 60;
    let h = ((total_s / 3600) % 24 + 24) % 24;
    (h as u32, m as u32, s as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    // [M 0x1a69d0:44-78] the base rows pin the level table's edges.
    #[test]
    fn level_base_table() {
        assert_eq!(level_clock_base(0x07000001), 3_240_000.0);
        assert_eq!(level_clock_base(0x07000004), 3_240_000.0);
        assert_eq!(level_clock_base(0x07000005), 7_020_000.0);
        assert_eq!(level_clock_base(0x07000008), 7_020_000.0);
        assert_eq!(level_clock_base(0x0700000b), 8_100_000.0);
        assert_eq!(level_clock_base(0x0700000c), 2_880_000.0);
        assert_eq!(level_clock_base(0x07000013), 2_160_000.0);
        assert_eq!(level_clock_base(0x0700004a), 2_160_000.0);
        assert_eq!(level_clock_base(0x07000016), 1_440_000.0);
        assert_eq!(level_clock_base(0x07000017), 0.0); // fall-through [M :78 default]
        assert_eq!(level_clock_base(0x07000009), 8_100_000.0);
        assert_eq!(level_clock_base(0x07000010), 0.0); // 0x10 not in any row
    }

    // [M :44 base 3.24e6 cs = 09:00] the readout round-trips the base.
    #[test]
    fn base_3_24e6_reads_09_00() {
        assert_eq!(readout(3_240_000.0, 0.0), (9, 0, 0));
        assert_eq!(readout(7_020_000.0, 0.0), (19, 30, 0));
        assert_eq!(readout(8_100_000.0, 0.0), (22, 30, 0));
        assert_eq!(readout(1_440_000.0, 0.0), (4, 0, 0));
    }

    // [M :90-104] the seconds rate: one revolution per 60 s → t = 60 s gives 2π (wrapped to 0).
    #[test]
    fn seconds_hand_wraps_once_per_minute() {
        let a = hand_angle(6000.0, SECONDS_RATE); // 6000 cs = 60 s
        assert!(a < 1e-3, "a={a}");
        let b = hand_angle(3000.0, SECONDS_RATE); // 30 s → half a turn
        assert!((b - std::f32::consts::PI).abs() < 1e-2, "b={b}");
    }

    // [M :90-104] the hours rate: 12 h per revolution → t = 6 h at base 0 → half turn.
    #[test]
    fn hours_hand_wraps_once_per_half_day() {
        let a = hand_angle(2_160_000.0, HOURS_RATE); // 2160000 cs = 6 h
        assert!((a - std::f32::consts::PI).abs() < 1e-2, "a={a}");
    }

    // [M :90-104] the minutes rate: one revolution per hour (the f32 product lands within a
    // quantisation step of 10π — accept the wrap either side, as the game's fmod would).
    #[test]
    fn minutes_hand_rate_matches_one_revolution_per_hour() {
        let a = hand_angle(3_600_000.0, MINUTES_RATE); // 1 h
        let tau = std::f32::consts::TAU;
        assert!(a < 1e-2 || a > tau - 1e-2, "a={a}");
    }
}
