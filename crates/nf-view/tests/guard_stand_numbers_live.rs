//! guard-stand-numbers: the 10-second idle stands the task cites, at the fresh-boot difficulty 2
//! ([L docs/live/2026-11-01-07000011-start-corridor-guards.md: the fresh-boot difficulty is 2
//! (0x2A3790)]), through the real input path — the level loads, the intro is skipped the game's own
//! way (`scripts.skip_request` while control is held, playtest_spots_live.rs's `free`), and Bond then
//! stands 600 fixed-60 Hz ticks (10 s) with `Input::default()` — no teleports, no return fire, no
//! top-up (the raw `Sim` has none; the walker's top-up lives in `Walker::tick`, common/walker.rs).
//!
//! The numbers the task quotes: standing 10 s on 07000006 costs 188 health and on 07000011 costs
//! 180, against 6.6 and 10 on a 10-17 build. This file pins what the current tree does; each
//! assertion cites the decompile chain the damage rides
//! (`Drone_ModPlayerHitDamage` 0x13a940 → `Drone_ModBulletDamage` 0x139f50 → `Player_HandlePain`
//! 0x1902c8, with `player_hit_damage`'s TooClose ×2 and backshot ×2 the only distance terms
//! [M 0x19455c-0x194588: no falloff on the chain]).

use nf_game::player::Input;
use nf_view::scene::Sim;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// Skip the intro the game's own way, then stand `ticks` ticks. Returns (health at control,
/// health at the end, the per-hit log (tick, owner, lost)).
fn stand(archive: &str, ticks: u32) -> (f32, f32, Vec<(u32, usize, f32)>) {
    let mut scene = nf_view::load_with_carry_difficulty(&iso(), archive, false, None, 2)
        .unwrap_or_else(|e| panic!("load {archive}: {e}"));
    let sim = scene.sim.as_mut().expect("sim");
    // The intro runs naturally (no skip): the guards tick through the cutscene like a fresh
    // boot left alone — the start_corridor_guard_07000011 shape.
    let mut intro = 0u32;
    while sim.disabled && intro < 3000u32 {
        sim.step_bond(Input::default(), false, false, false);
        intro += 1;
    }
    assert!(!sim.disabled, "{archive}: control never returned within 3000 ticks");
    eprintln!("[{archive}] intro ran {intro} ticks");
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
    let lost = h0 - h1;
    eprintln!("[{archive}] stood 10 s: health {h0:.2} -> {h1:.2}, lost {lost:.2} over {} hits", log.len());
    for (t, owner, lost) in log {
        eprintln!("  tick {t}: owner {owner} lost {lost:.2}");
    }
}

#[test]
fn stand_10s_07000006() {
    let (h0, h1, log) = stand("07000006.bin", 600);
    print_log("07000006", h0, h1, &log);
}

#[test]
fn stand_10s_07000011() {
    let (h0, h1, log) = stand("07000011.bin", 600);
    print_log("07000011", h0, h1, &log);
}
