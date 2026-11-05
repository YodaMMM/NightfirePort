//! guard-stand-numbers: the 10-second idle stands at the fresh-boot difficulty 2 ([L]
//! docs/live/2026-11-01-07000011-start-corridor-guards.md: the fresh-boot difficulty is 2,
//! 0x2A3790), through the real input path — the level loads, Bond stands at his own spawn for 600
//! fixed-60 Hz ticks with `Input::default()` — no teleports, no return fire, no top-up (the raw
//! `Sim` has none; the walker's top-up lives in `Walker::tick`, common/walker.rs).
//!
//! The task's numbers: standing 10 s on 07000006 costs 188 health and on 07000011 costs 180,
//! against 6.6 and 10 on a 10-17 build (measured on the driver's own stands, which are not the
//! level spawns — see the report). This file pins what the decompile says the per-hit damage and
//! the 10 s totals must be, through the real guard-fire path:
//!
//! - Per hit on 07000011 at d2: exactly 12.0 — raw `weapon_data+0x0C` = 10 [D weapon table] ×
//!   `Drone+0x100` = 2.0 for the held-weapon class {2, 6, 0xA, 0xB, 0xE, 0x10, 0x3B..0x40}
//!   [M NDrone2_DefaultInit 0x14b300:267-293 — the switch's `*(iVar3+0x100) = 0x40000000`;
//!   verified in work/decomp/ACTION.ELF/0014b300] × Plr_DMod_Normal(07000011) = 0.6 at d2
//!   [M ReadTuningVars 0x1d3a80, group 11–13+4A 0.5/0.6/1.0 @ 0x1d42bc] = 12.0. This equals the
//!   live-measured 10/12/20 per hit at d1/d2/d3 [L the same doc] — no distance falloff anywhere
//!   in the chain [M 0x19455c-0x194588: HIT+8 + Hurt_GetDamage; the only distance terms are the
//!   TooClose ×2 and the backshot ×2, Drone_ModBulletDamage 0x139f50], and neither can fire here
//!   (the guards' anim set is 4, not 0x12; they fire from 6–9 u [L], beyond TooClose).
//! - The hit roll: `DroneWeap_DoBulletAccuracy` 0x179740 as ported in `hit_chance` — including
//!   the game's own double TooClose_Accuracy multiply [M 0x179740, both sites read in the
//!   decompile], the Rand_FRand(100) roll, the NewSighting/lost-sight gate and the <3 u
//!   always-hit.
//! - The 10-17 build (9c71ceea7) took 3 × 6.0 = 18 on this stand (the ×2 class write did not
//!   exist yet — it landed 2026-10-22, 3b3bd4eeb standins-drone-922) and 0 hits on 07000006's
//!   spawn; the rework commits 065d6e53f (fire: pellets per shot, tuning hit chance), 48edddf6e
//!   (Bond's hit volume) and adbcbe69e (placement stats + the ×2 re-land) carried it to the
//!   decompile numbers.

use nf_game::player::Input;
use nf_view::scene::Sim;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// Skip nothing: the intro is already disabled=false on a plain `load_with_carry_difficulty` load.
/// Stand `ticks` ticks at the spawn. Returns (health at control, health at the end, the per-hit log).
fn stand(archive: &str, ticks: u32) -> (f32, f32, Vec<(u32, usize, f32)>) {
    let mut scene = nf_view::load_with_carry_difficulty(&iso(), archive, false, None, 2)
        .unwrap_or_else(|e| panic!("load {archive}: {e}"));
    let sim = scene.sim.as_mut().expect("sim");
    let mut intro = 0u32;
    while sim.disabled && intro < 3000u32 {
        sim.step_bond(Input::default(), false, false, false);
        intro += 1;
    }
    assert!(!sim.disabled, "{archive}: control never returned within 3000 ticks");
    let h0 = sim.drones.health.health;
    let mut log = Vec::new();
    let mut last = 0u32;
    for t in 0..ticks {
        sim.step_bond(Input::default(), false, false, false);
        for &(f, owner, lost) in sim.drones.player_hits.iter().skip_while(|&&(f, _, _)| f <= last) {
            log.push((t, owner, lost));
        }
        last = sim.drones.frame;
    }
    let h1 = sim.drones.health.health;
    (h0, h1, log)
}

fn print_log(archive: &str, h0: f32, h1: f32, log: &[(u32, usize, f32)]) {
    eprintln!("[{archive}] stood 10 s: health {h0:.2} -> {h1:.2}, lost {:.2} over {} hits", h0 - h1, log.len());
    for (t, owner, lost) in log {
        eprintln!("  tick {t}: owner {owner} lost {lost:.2}");
    }
}

/// The 07000011 spawn stand: the two start-corridor office guards (#126/#127, drones the hits'
/// owners) walk in and fire — the [L] shape ("idle Bond dies at f1113 (d2)"). The 10 s total is
/// RNG (the hit rolls draw the shared stream), but every hit must be the decompile's 12.0 and
/// the stand must stay alive (the [L] death is at ~f1113, past the 600-tick stand).
#[test]
fn stand_10s_07000011_per_hit_is_the_decompile_12() {
    let (h0, h1, log) = stand("07000011.bin", 600);
    print_log("07000011", h0, h1, &log);
    assert!(!log.is_empty(), "the corridor guards never hit an idle 10 s Bond (the [L] doc fires by f200-f600)");
    for (t, owner, lost) in &log {
        assert_eq!(*lost, 12.0, "tick {t}: owner {owner} hit for {lost} — the d2 per-hit must be \
            raw 10 × Drone+0x100 2.0 [M 0x14b300] × Plr_DMod_Normal 0.6 [M 0x1d42bc] = 12.0 [L 10/12/20]");
    }
    let lost = h0 - h1;
    assert_eq!(lost, 12.0 * log.len() as f32, "the total is the per-hit sum (no other Bond-damage site fired)");
    assert!(h1 > 0.0, "an idle 10 s stand at d2 must survive ([L] the death is at f1113, ~18.5 s)");
}

/// The 07000006 spawn stand: the start platform's guards are beyond engagement range for an idle
/// 10 s (0 hits, both on master and on the 10-17 build — measured). The driver's 188-health stand
/// on this level is NOT the spawn; the per-hit chain it exercises is the same one the 07000011
/// test pins, at the level's own Plr_DMod group (05–08: 0.6/0.7/1.0 [M 0x1d3c44] → d2 ×0.7).
#[test]
fn stand_10s_07000006_spawn_takes_no_hits() {
    let (h0, h1, log) = stand("07000006.bin", 600);
    print_log("07000006", h0, h1, &log);
    assert!(log.is_empty(), "the 07000006 spawn stand took {:?} — the start guards are out of range for 10 idle s", log);
    assert_eq!(h0, h1, "no other Bond-damage site may fire on an idle spawn stand");
}
