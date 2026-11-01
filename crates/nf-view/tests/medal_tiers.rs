//! medal-tiers: the results page's medal tier, end to end through the port's own scoring, and
//! the two pages that show the award afterwards (the 007 Bonus "Reward Card" and the Dossier
//! Records medal). Headless; numbers and ids only.
//!
//! Evidence [M = the decompile]:
//! - `P_NFRESULTS_Handler` 0x208e78:98-142 — the compare of the last score (entry +0x34) against
//!   +0x10 platinum, +0x0C gold, +0x08 silver, +0x04 bronze (unsigned `<`, so equal = that tier);
//!   medal counter 4/3/2/1/5, name 0x2c7/0x2c8/0x2c9/0x2ca/0x10001fd, emblem
//!   0x300013f/0x300013e/0x3000140/0x300013d/0x300019a; :169 `Menu_SetLevelBonus(base, medal, 0)`.
//! - `PlrStat_GetScore` 0x1984b0 — the rows, base = sum of raws, bonus = base x
//!   `ScoringTableDifMul` [0, 1, 2] by GameState+0x28 (1/2/3), last = base + bonus.
//! - `Menu_SetLevelBonus` 0x1ffa10:70-79 — counters medal..1 each set once.
//! - `P_NFBONUS_Handler` 0x209670:14-41 — tier 1's texture, then the labels of tiers 1, 4, 3, 2.
//! - `C_RBDSRECORDS` 0x20bf00:58-110 + disasm 0x20c0e4-0x20c154 — all four bonus flags ->
//!   0x306 / 0x300013f (platinum), k1..k3 -> 0x307 / 0x300013e (gold).
//!
//! The scored level is Equinox, 0x0700001B: thresholds [250000, 500000, 750000, 1000000]; rows
//! dispatched (15, 100000), accuracy (100, 100000), health (100, 100000), time (255, 0), pickups
//! (7, 150000) [the ScoringTable bytes at 0x2bc640 + 11 x 60, read from the EE snapshot].

use nf_game::progress::{results_medal, Progress, SCORING_TABLE};
use nf_view::menu::FrontEnd;

const EQUINOX: u32 = 0x0700_001B;
const EQUINOX_ROW: usize = 11; // its sp_level / Records row

/// A run's tally on a fresh progress layer.
struct Run { dispatched: u32, accurate: bool, health: f32, pickups: u32 }

fn finish(run: &Run, difficulty: u32) -> (FrontEnd, Progress) {
    let mut p = Progress::default();
    let s = &mut p.stats[0];
    s.dispatched = run.dispatched;
    if !run.accurate { s.shots_fired = 10; s.shots_hit_enemy = 0; } // no shots = the full par
    s.health = run.health;
    s.bonus_pickups = run.pickups;
    let mut fe = FrontEnd::new();
    assert!(fe.results_enter(&mut p, EQUINOX, 6, difficulty, None), "Equinox is a ScoringTable entry");
    fe.session_progress = Some(p.clone());
    (fe, p)
}

#[test]
fn equinox_thresholds_are_the_tables() {
    let e = SCORING_TABLE.iter().find(|e| e.hash == EQUINOX).unwrap();
    assert_eq!(e.thresholds, [250_000, 500_000, 750_000, 1_000_000]);
}

/// Each tier at 00 Agent (x2: total = 3 x base): the score, the medal, its name/emblem/target
/// line, and the reward bits the award sets.
#[test]
fn each_tier_from_a_real_score() {
    // (run, expected total, medal, name label, emblem, target label, target)
    let cases = [
        (Run { dispatched: 1, accurate: false, health: 0.0, pickups: 0 }, 20_001, 5, 0x100_01fd, 0x0300_019a, 0x2cf, 250_000),
        (Run { dispatched: 0, accurate: true, health: 0.0, pickups: 0 }, 300_000, 1, 0x2ca, 0x0300_013d, 0x2ce, 500_000),
        (Run { dispatched: 0, accurate: true, health: 100.0, pickups: 0 }, 600_000, 2, 0x2c9, 0x0300_0140, 0x2cd, 750_000),
        (Run { dispatched: 10, accurate: true, health: 100.0, pickups: 0 }, 800_001, 3, 0x2c8, 0x0300_013e, 0x2cc, 1_000_000),
        (Run { dispatched: 0, accurate: true, health: 100.0, pickups: 7 }, 1_050_000, 4, 0x2c7, 0x0300_013f, 0x2cc, 1_000_000),
    ];
    for (run, total, medal, name, emblem, tlabel, target) in cases {
        let (fe, p) = finish(&run, 3);
        let rf = fe.results.as_ref().expect("the results fields");
        assert_eq!(rf.score, total, "the run's total");
        assert_eq!(rf.medal, medal, "score {total} -> medal");
        assert_eq!((rf.medal_label, rf.emblem, rf.target_label, rf.target), (name, emblem, tlabel, target), "score {total}'s page fields");
        for k in 1..=4u32 {
            assert_eq!(p.has_medal(EQUINOX, k, 0), medal != 5 && k <= medal, "score {total}: reward counter {k}");
        }
    }
}

/// The platinum finish: every page that shows the award says platinum. Before the Records fix
/// (records-medal) this exact state drew the GOLD emblem on the Records page; the sister pins
/// that a gold finish does draw gold there.
#[test]
fn a_platinum_finish_shows_platinum_everywhere() {
    let (mut fe, p) = finish(&Run { dispatched: 15, accurate: true, health: 100.0, pickups: 7 }, 3);
    let rf = fe.results.as_ref().unwrap();
    assert_eq!((rf.score, rf.medal, rf.emblem), (1_350_000, 4, 0x0300_013f), "the results page: platinum");
    assert!(fe.new_reward, "the first platinum sets new rewards");
    // The 007 Bonus page (the "Reward Card"): all four tiers, tier 1's picture, labels 1, 4, 3, 2.
    fe.bonus_enter(&p, EQUINOX, fe.new_reward);
    let b = fe.bonus.as_ref().unwrap();
    assert!(b.lines.iter().all(|l| l.has), "all four tiers stand after platinum");
    assert_eq!(b.texture, Some(b.lines[0].texture));
    assert_eq!(b.text_labels, vec![b.lines[0].label, b.lines[3].label, b.lines[2].label, b.lines[1].label]);
    // The Dossier Records medal.
    assert_eq!(fe.record_medal(EQUINOX_ROW), Some((0x306, 0x0300_013f)), "Records: platinum, not gold");

    // Sister: a gold finish is gold on both pages.
    let (mut fg, pg) = finish(&Run { dispatched: 10, accurate: true, health: 100.0, pickups: 0 }, 3);
    assert_eq!(fg.results.as_ref().unwrap().medal, 3);
    assert_eq!(fg.record_medal(EQUINOX_ROW), Some((0x307, 0x0300_013e)), "Records: gold");
    fg.bonus_enter(&pg, EQUINOX, fg.new_reward);
    assert!(!fg.bonus.as_ref().unwrap().lines[3].has, "no platinum tier after gold");
}

/// The difficulty multiplier decides the tier: the same perfect Equinox run is gold at Agent
/// (x1: 2 x 450000 = 900000 < 1000000) and platinum at 00 Agent (x2: 1350000); at Operative
/// (x0) it is 450000, bronze.
#[test]
fn the_difficulty_bonus_moves_the_tier() {
    let perfect = Run { dispatched: 15, accurate: true, health: 100.0, pickups: 7 };
    for (dif, total, medal) in [(1, 450_000, 1), (2, 900_000, 3), (3, 1_350_000, 4)] {
        let (fe, _) = finish(&perfect, dif);
        let rf = fe.results.as_ref().unwrap();
        assert_eq!((rf.score, rf.medal), (total, medal), "difficulty {dif}");
    }
}

/// The page and the shared chain agree on every table entry's exact thresholds.
#[test]
fn the_chain_at_every_threshold() {
    for e in SCORING_TABLE.iter() {
        let th = e.thresholds;
        for (tier, &t) in th.iter().enumerate() {
            assert_eq!(results_medal(t, th), tier as u32 + 1, "{:#x}: exactly the tier-{} threshold", e.hash, tier + 1);
            let below = if tier == 0 { 5 } else { tier as u32 };
            assert_eq!(results_medal(t - 1, th), below, "{:#x}: one under the tier-{} threshold", e.hash, tier + 1);
        }
    }
}
