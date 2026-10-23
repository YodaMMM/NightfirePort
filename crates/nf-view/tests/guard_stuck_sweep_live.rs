//! guard-barks-first-alert-3: the stuck-guard sweep (task 961b's sweep tool). User report
//! (2026-10-22, playing 0.1.17): "The exchange had a few guards stuck in the helipad section",
//! then "Not just 05" — symptom only, the cause not known; this sweep MEASURES it.
//!
//! Method per on-foot SP level: load through the real path (`nf_view::load`), let the level start,
//! then wake every live watching guard through the game's OWN alarm path — msg 0x1e (ALARM)
//! [M `DroneVision_EnemyAlerts` case 0x1e 0x177c18:113-114 — `+0x4F8 |= 0x4000000 | 0x40000000`,
//! SetState(0x56); the port's ALARM row stamps `alert_pending |= 0x40000000` and Attack] — and run
//! 60 s of fixed 60 Hz ticks. A drone is a HIT when it sits in a MOVING state — its mode/substate
//! wants it to travel:
//!   - ATTACK 0x56 — the chase/combat movement (the entry is `DroneFunc_FirstAttack` 0x148010, the
//!     tail runs the approach/strafe cycle) [M];
//!   - RUN_FOR_COVER 0x8c — run-to-cover [M `NDrone2_DSTATE_RunForCover` 0x169150];
//!   - ALERT_TO_POSITION 0x62 — walk to the alerted position [M `AlertToPosition` 0x165270];
//!   - SEARCH_AREA 0x65 — investigate [M `SearchArea` 0x165bf0];
//!   - HEARD_NOISE_AWARE/SUSPECT/ALERT 0xa3/0xa4/0xa5 — the sound-investigate family [M
//!     `HeardNoiseSuspect` 0x16b0e0];
//!   - GO_TO_GOAL 0x63 / GO_TO_GOAL_ALLY 0x28 — path follow (the port's GoToGoal, the
//!     GoToGoalPosition 0x63 arms) [M];
//!   - PATROL 0x07 and CIVILIAN_MISSION 0x16 — the mission-route walkers (`update_mission_route`'s
//!     `route.follow`, the `NDrone2_ReachedDestNode` 0x153358 machinery) [M];
//!   - INTEROGATE_ASSIST 0x39 — walk to the assist arrival point [M the TIMER_C walk 0x15add0 +
//!     0x1517d8:80-83];
//! yet its position moves < 0.1 over 3 s (180 ticks). Per hit the sweep logs: level, drone
//! index, skin, state, mode byte (+0xC5), position, the goal it holds (`+0x558`-family target),
//! `route_blocked` (the nav follow's BLOCKED/NO_PATH latch [M `Network::push_out` /
//! `update_mission_route`]), the goal distance, and whether it stands inside 07000005's helipad
//! courtyard box (the parked helicopter statics' spheres (−17.0, 43.0..44.8, −37.2..−43.6) [D
//! fix-portal5.md §6] — the user's reported section). This test is LOG-ONLY (a measurement, not a
//! pin): the hits' table goes to docs/reports/guard-barks-first-alert-3.md; a follow-up fix turns
//! the exact rows into assertions.

#[path = "common/walker.rs"]
mod walker;

use nf_game::drone::{msg, state};
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The on-foot SP levels the port loads (the census/walkreal coverage: 07000001–0d, 11–16, 1b(?),
/// 4a). `SWEEP_LEVELS` overrides (comma-separated) for a targeted run.
fn levels() -> Vec<String> {
    std::env::var("SWEEP_LEVELS").unwrap_or_else(|_| {
        "07000005,07000007,0700000c,07000001,07000002,07000004,07000006,07000008,07000009,0700000a,0700000b,0700000d,07000012,07000013,07000014,07000015,07000016".to_string()
    })
    .split(',')
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty())
    .collect()
}

fn state_name(s: u8) -> &'static str {
    match s {
        state::ATTACK => "ATTACK 0x56 (chase)",
        state::RUN_FOR_COVER => "RUN_FOR_COVER 0x8c",
        state::ALERT_TO_POSITION => "ALERT_TO_POSITION 0x62",
        state::SEARCH_AREA => "SEARCH_AREA 0x65",
        state::HEARD_NOISE_AWARE => "HEARD_NOISE_AWARE 0xa3",
        state::HEARD_NOISE_SUSPECT => "HEARD_NOISE_SUSPECT 0xa4",
        state::HEARD_NOISE_ALERT => "HEARD_NOISE_ALERT 0xa5",
        state::GO_TO_GOAL => "GO_TO_GOAL 0x63",
        state::GO_TO_GOAL_ALLY => "GO_TO_GOAL_ALLY 0x28",
        state::PATROL => "PATROL 0x07 (route walk)",
        state::CIVILIAN_MISSION => "CIVILIAN_MISSION 0x16 (route walk)",
        state::INTEROGATE_ASSIST => "INTEROGATE_ASSIST 0x39",
        _ => "?",
    }
}

fn moving(s: u8) -> bool {
    matches!(s, state::ATTACK | state::RUN_FOR_COVER | state::ALERT_TO_POSITION | state::SEARCH_AREA
        | state::HEARD_NOISE_AWARE | state::HEARD_NOISE_SUSPECT | state::HEARD_NOISE_ALERT
        | state::GO_TO_GOAL | state::GO_TO_GOAL_ALLY | state::INTEROGATE_ASSIST
        | state::PATROL | state::CIVILIAN_MISSION)
}

/// 07000005's helipad courtyard box [D fix-portal5.md §6 — the heli statics' spheres].
fn in_helipad_box(level: &str, p: [f32; 3]) -> bool {
    level == "07000005" && (-30.0..2.0).contains(&p[0]) && p[1] > 30.0 && (-58.0..-22.0).contains(&p[2])
}

fn sweep_level(lv: &str) {
    let Ok(mut w) = nf_view::load(&iso(), &format!("{lv}.bin"), false)
        .map(walker::Walker::new) else { eprintln!("SWEEP {lv}: LOAD FAILED"); return; };
    if !w.wait_enabled(6000, "level start") { eprintln!("SWEEP {lv}: never enabled"); return; }
    w.wait(120); // the level's first second settles (spawners, scripts)
    // The game's own alarm path on every live watching guard: msg 0x1e [M 0x177c18:113-114].
    {
        let sim = w.scene.sim.as_mut().unwrap();
        let difficulty = sim.drones.difficulty;
        let data = nf_game::drone::Data {
            tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
            world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
            skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
            hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
            surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty,
            ai_points: &sim.ai_points,
        };
        let frame = sim.drones.frame;
        let mut woken = 0;
        for i in 0..sim.drones.list.len() {
            let d = &sim.drones.list[i];
            // The alarm row's own state band [M 0x161860 case 0x1e — EnemyAlerts serves the unaware
            // family; the port's ALARM arm covers IDLE|ALERT|PATROL|ALERT_TO_POSITION|SEARCH_AREA|
            // HEARD_NOISE_*|INTEROGATE_ASSIST_WAIT]. The route walkers (PATROL/CIVILIAN_MISSION)
            // are already moving — they are swept as-is.
            if d.present && d.enabled && !d.corpse && d.watching && d.team != 2 && d.health > 0.0
                && matches!(d.state, state::IDLE | state::ALERT | state::PATROL) {
                sim.drones.list[i].send(&data, &mut sim.drones.rng, frame, msg::ALARM);
                woken += 1;
            }
        }
        eprintln!("SWEEP {lv}: alarmed {woken} guards [M 0x177c18:113-114]");
    }
    // 60 s of ticks; per drone, an anchor (pos, tick) resets whenever it leaves a MOVING state or
    // moves ≥ 0.1; a hit is 180 consecutive ticks in a MOVING state within 0.1 of the anchor.
    const WINDOW: u32 = 180;
    let mut anchor: Vec<([f32; 3], u32)> = Vec::new();
    let mut hits: Vec<String> = Vec::new();
    for t in 0..3600u32 {
        w.tick(Input::default(), false);
        // A second alarm pass at t=1800: guards that lost the alert (the idle timeout msg 4 re-arms
        // [M 0x148db0:89-96]) re-engage, so a mid-level wedge in the second half still measures.
        if t == 1800 {
            let sim = w.scene.sim.as_mut().unwrap();
            let difficulty = sim.drones.difficulty;
            let data = nf_game::drone::Data {
                tables: &sim.tables, props: &sim.props, scripts: &sim.scripts, weapons: &sim.weapons,
                world: &sim.world, level: sim.level, sequences: &sim.seqs, skins: &sim.skins,
                skeletons: &sim.skeletons, nav: &sim.nav, covers: &sim.covers, tuning: &sim.tuning,
                hitboxes: &sim.hitboxes, volumes: &sim.volumes, sfx: sim.sfx.as_ref(), models: &sim.models,
                surfaces: sim.surfaces.as_ref(), voices: sim.voices.as_deref(), difficulty,
                ai_points: &sim.ai_points,
            };
            let frame = sim.drones.frame;
            for i in 0..sim.drones.list.len() {
                let d = &sim.drones.list[i];
                if d.present && d.enabled && !d.corpse && d.watching && d.team != 2 && d.health > 0.0
                    && matches!(d.state, state::IDLE | state::ALERT) {
                    sim.drones.list[i].send(&data, &mut sim.drones.rng, frame, msg::ALARM);
                }
            }
        }
        let sim = w.scene.sim.as_ref().unwrap();
        if anchor.len() != sim.drones.list.len() { anchor.resize(sim.drones.list.len(), ([0.0; 3], 0)); }
        for (i, d) in sim.drones.list.iter().enumerate() {
            if !d.present || !d.enabled || d.corpse || d.health <= 0.0 { continue; }
            if !moving(d.state) { anchor[i].1 = 0; continue; }
            let moved = ((d.pos[0] - anchor[i].0[0]).powi(2) + (d.pos[1] - anchor[i].0[1]).powi(2) + (d.pos[2] - anchor[i].0[2]).powi(2)).sqrt();
            if moved >= 0.1 { anchor[i] = (d.pos, t); continue; }
            if anchor[i].1 == 0 { anchor[i] = (d.pos, t); continue; }
            if t - anchor[i].1 >= WINDOW {
                let gd = d.goal.map(|g| ((g[0] - d.pos[0]).powi(2) + (g[1] - d.pos[1]).powi(2) + (g[2] - d.pos[2]).powi(2)).sqrt());
                let row = format!(
                    "HIT {lv} drone#{i} skin {:#010x} state {} mode {:#04x} pos ({:.2},{:.2},{:.2}) goal {:?} goal_dist {:?} route_blocked {} helipad_box {}",
                    d.skin, state_name(d.state), d.mode, d.pos[0], d.pos[1], d.pos[2],
                    d.goal.map(|g| (g[0], g[1], g[2])), gd.map(|x| (x * 100.0).round() / 100.0),
                    d.route_blocked, in_helipad_box(lv, d.pos));
                if !hits.contains(&row) { hits.push(row.clone()); eprintln!("{row}"); }
                anchor[i] = (d.pos, t); // re-anchor: one row per (drone, spot) wedge
            }
        }
    }
    eprintln!("SWEEP {lv}: {} hits over 60 s", hits.len());
}

/// The sweep itself: log-only. Every HIT line is a row of the report's table; a level that fails
/// to load or panics is a SKIP row (its absence from the table is explicit, not silent).
#[test]
fn the_sweep_logs_every_frozen_moving_drone() {
    for lv in levels() {
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| sweep_level(&lv)));
        if r.is_err() { eprintln!("SWEEP {lv}: SKIP (panic — see the report's skip list)"); }
    }
}
