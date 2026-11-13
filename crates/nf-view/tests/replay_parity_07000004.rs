//! replay-parity-walker-side-effects: the 07000004 full-mission walk, RECORDED and REPLAYED.
//!
//! The sim is deterministic and a walk's inputs replay to the same state, tick for tick, once
//! every stepped tick is captured. The one hole was the shared damage_log fight loop's reload
//! branch stepping `step_bond` directly, bypassing the walker's tick path (the keep_alive
//! top-ups, the cut-scene skip request, rise_watch, drone_watch never ran on those ticks). The
//! reload step now goes through `tick_inner`, every walker-stepped tick steps through ONE site
//! (`step_recorded`), and the recorder at that site captures each tick's exact input in the
//! replay file's own form (`TickInput::from_parts`, stepped with `to_parts()` — the sim sees
//! exactly what is recorded).
//!
//! This test walks the 07000004 full-mission route (the same route walkreal_07000004_live's
//! `walk_the_full_mission_start_to_finish` walks: difficulty 1 Operative [M the `difficulty[]`
//! table 0x2df4c0 row carries 1; load_with_carry_difficulty rides the front end's launch path],
//! top_up_health FALSE — real damage, no top-ups — and skip_cutscenes FALSE, the pickups lap
//! StormUzi #7 → BodyArmourHeavy #9 [D pickups] then the 0x0500001e target shot), records every
//! tick, then replays the inputs on a FRESH `nf_view::load` and asserts per-tick equality of
//! Bond pos/yaw/health, the drone frame and the mission_end latch, and the same mission_end
//! tick. The walk stops at the FIRST tick `mission_end` is Some (the route used to keep walking
//! the lap and the kill rounds ~6300 ticks past it). Expected: the mission ends around tick
//! 3003 (the target dies to return fire at ~2683 — the driver's measured run of this route).

#[path = "common/walker.rs"]
mod walker;

use nf_game::mission;
use walker::{WalkTickState, Walker};

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The mission-table index of THIS level's record [L ee_start_05.bin 0x2a4350].
fn mission_row(level: u32) -> usize {
    mission::LEVELS.iter().position(|l| l.level == level).expect("the level's mission record")
}

/// One replayed tick's snapshot — the same fields `step_recorded` captures (the comparison is
/// bit-exact: the same f32s, no tolerances).
fn replay_state(scene: &nf_view::scene::Scene) -> WalkTickState {
    let sim = scene.sim.as_ref().unwrap();
    WalkTickState {
        pos: sim.bond.pos,
        yaw: sim.bond.yaw,
        health: sim.drones.health.health,
        frame: sim.drones.frame,
        mission_end: sim.drones.mission_end,
    }
}

/// The 07000004 full-mission walk with the recorder on, stopping at the FIRST tick mission_end
/// is Some. Returns (inputs, per-tick states). The route mirrors walkreal_07000004_live's
/// `walk_the_full_mission_start_to_finish_at(1)` — the same legs, the same waits — with a
/// mission_end break after every leg and inside every wait (that walk's route kept walking the
/// lap and the kill rounds after the mission had ended).
fn record_full_mission(iso: &str) -> (Vec<nf_view::replay::TickInput>, Vec<WalkTickState>) {
    let mut w = Walker::new(nf_view::load_with_carry_difficulty(iso, "07000004.bin", false, None, 1)
        .expect("load 07000004"));
    // The parity run's own switches: real damage (no top-ups — never lowered damage or health),
    // the cut-scenes PLAY (no skip request — a recorded route must see the script's own ticks),
    // and the recorder rides the walker's one step site.
    w.top_up_health = false;
    w.skip_cutscenes = false;
    w.start_recording();
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.level, 0x0700_0004);
        let p = sim.bond.pos;
        assert!((p[0] - 1.87).abs() < 2.0 && (p[2] - 41.07).abs() < 2.0, "start #0 [D], got {p:?}");
    }
    // Load and let the intro + story scenes PLAY (no skip) until the AI re-enables and ch 0x53
    // (the castle-C hand-off [D script 0x06000082 stream 9]) is written — the same settle the
    // walkreal route runs, ticking through the scenes like a player who does not press skip.
    let mut ready = false;
    for _ in 0..6000u32 {
        w.tick(Default::default(), false);
        let sim = w.scene.sim.as_ref().unwrap();
        if sim.drones.npc_enabled && !sim.disabled && sim.drones.channels.on(0x53) { ready = true; break; }
    }
    assert!(ready, "the intro + story scenes must settle (the AI re-enabled, ch 0x53 written)");
    let row = mission_row(0x0700_0004);
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission.states[row][0], 3,
            "obj0 'Defeat the assassin' announced through the real monitor (enable ch 0x53 on) [M 0x1827d8:55-67]");
        assert_eq!(sim.drones.mission.state, 1, "the mission runs [M Mission_Init 0x182270]");
    }
    // The armament lap (the route's own pickups [D #6/#7/#8 StormUzi + #9 BodyArmourHeavy]):
    // the open-lawn uzi #7 (−10.66, 4.39, 42.66), then the armour #9 (−1.85, 4.37, 46.07).
    // Operative keeps the pickup lap (prefer_head stays off, the walkreal route's difficulty<2
    // branch). The mission can END mid-leg (the target dies to Bond's return fire — the leg's
    // `return_fire` answer kills her, the death channel + the OUT mux + the banner land, and
    // mission_end latches while the lap still walks): stop at the first such tick.
    let (uzi, armour) = {
        let sim = w.scene.sim.as_ref().unwrap();
        let uzi = sim.drones.pickups.iter()
            .find(|p| p.kind == 0 && p.weapon == 18 && !p.gone && (p.start[2] - 42.66).abs() < 1.0)
            .or_else(|| sim.drones.pickups.iter().find(|p| p.kind == 0 && p.weapon == 18 && !p.gone))
            .map(|p| p.start).expect("the StormUzi pickup");
        let armour = sim.drones.pickups.iter().find(|p| p.kind == 3 && !p.gone).map(|p| p.start)
            .expect("the BodyArmourHeavy pickup");
        (uzi, armour)
    };
    let target_alive = |w: &Walker| w.scene.sim.as_ref().unwrap().drones.list.iter().any(|d| d.skin == 0x0500_001e && d.health > 0.0);
    let mission_ended = |w: &Walker| w.scene.sim.as_ref().unwrap().drones.mission_end.is_some();
    w.walk_nav_to(uzi, "the StormUzi pickup", 0.4);
    if !mission_ended(&w) {
        if target_alive(&w) {
            let sim = w.scene.sim.as_ref().unwrap();
            assert!(sim.drones.inventory.owned.get(18).copied().unwrap_or(false),
                "walking over the pickup owns weapon 18 [M Pickup_Handler 0x18e650 type-0 arm]");
        }
        if target_alive(&w) { w.walk_nav_to(armour, "the armour pickup", 0.4); }
        if target_alive(&w) && !mission_ended(&w) {
            let sim = w.scene.sim.as_ref().unwrap();
            assert!(sim.drones.health.armour > 0.0,
                "the take raised armour from 0 [M 0x18e650:220-225]");
        }
    }
    // The kill (if the return fire has not already done it): walk back to the target and shoot
    // — the player's own fire path, the walkreal route's kill rounds with the mission_end stop.
    let target_i = w.scene.sim.as_ref().unwrap().drones.list.iter()
        .position(|d| d.skin == 0x0500_001e).expect("the target drone exists");
    if !mission_ended(&w) && target_alive(&w) {
        let mut killed = w.shoot(target_i, "the target");
        for round in 1..4 {
            let sim = w.scene.sim.as_ref().unwrap();
            if killed || sim.drones.list[target_i].corpse || sim.drones.list[target_i].health <= 0.0 { break; }
            let ammo = sim.gun.clip + sim.gun.spare;
            let live = sim.drones.pickups.iter().find(|p| p.kind == 0 && p.weapon == 18 && !p.gone).map(|p| p.start);
            drop(sim);
            if mission_ended(&w) { break; }
            if let Some(p) = live {
                eprintln!("[parity] round {round}: resupply at ({:.2},{:.2},{:.2}) with ammo {ammo}", p[0], p[1], p[2]);
                w.walk_nav_to(p, &format!("the StormUzi resupply {round}"), 0.4);
                if mission_ended(&w) { break; }
            }
            killed = w.shoot(target_i, "the target");
        }
    }
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert!(sim.drones.list[target_i].corpse || sim.drones.list[target_i].health <= 0.0,
            "the target must die (the walk's own shots or its return fire)");
    }
    // The death channel settles over the death anim; the objective folds through the REAL
    // monitor; the OUT mux mirrors it into ch 100 every tick with NO touch check [M 0x1b1a78
    // case 5:147-160]; Mission_Update's banner runs 240 ticks; mission_end latches. Every wait
    // stops at the FIRST tick of its condition — the mission-end wait at the first mission_end
    // tick (the route used to keep walking ~6300 ticks past it).
    if !mission_ended(&w) {
        let mut raised = false;
        for _ in 0..900u32 {
            w.tick(Default::default(), false);
            let sim = w.scene.sim.as_ref().unwrap();
            if sim.drones.channels.on(0x54) || sim.drones.mission_end.is_some() { raised = sim.drones.channels.on(0x54); break; }
        }
        assert!(raised || mission_ended(&w),
            "the target's death must raise ch 0x54 [M DroneFunc_SetDeathChannel 0x13e660] (or the mission already ended)");
        for _ in 0..30u32 { w.tick(Default::default(), false); if mission_ended(&w) { break; } }
    }
    let mut ended = false;
    for _ in 0..900u32 {
        w.tick(Default::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.mission_end.is_some() { ended = true; break; }
    }
    assert!(ended, "the mission-complete sequence must land (case 6 banner → case 4 ResetMap)");
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.mission_end, Some(0x0730_0004),
            "LevelEnd 0x0730004 → the end sequence with LevelToEndTo 0x0730004 [M 0x182c60:65-76, 89-90]");
    }
    let inputs = w.record_inputs.take().expect("the recorder ran");
    let states = w.record_states.take().expect("the recorder ran");
    assert_eq!(inputs.len(), states.len(), "one state per recorded tick");
    eprintln!("[parity] the walk recorded {} ticks; mission_end at tick {}",
        inputs.len(), states.iter().position(|s| s.mission_end.is_some()).map(|i| i + 1).unwrap_or(0));
    (inputs, states)
}

#[test]
fn replay_parity_07000004() {
    let iso = iso();
    let (inputs, recorded) = record_full_mission(&iso);
    assert!(!inputs.is_empty(), "the walk recorded nothing");
    // The mission ends around tick 2307 (the target dies to return fire at ~1986 — the driver's
    // measured run of this route, once the assassin wakes seeing Bond at the cut-scene's
    // hand-back spot and closes to the sword sooner); the band pins the route's real timeline
    // without pinning a single tick of it.
    let end_tick = recorded.iter().position(|s| s.mission_end.is_some()).map(|i| i + 1)
        .expect("mission_end latched in the recorded run");
    assert!((1900..=3000).contains(&end_tick),
        "the mission must end around tick 2307 (measured; got {end_tick})");
    assert_eq!(recorded.last().unwrap().mission_end, Some(0x0730_0004),
        "the walk stopped AT the first mission_end tick [M 0x182c60:65-76]");
    // The replay: a FRESH load, the same difficulty, the sound attached as the walker attaches
    // it (the run's own setup), then every recorded tick fed back through `to_parts()` — the
    // sim sees exactly what is recorded.
    let mut scene = nf_view::load_with_carry_difficulty(&iso, "07000004.bin", false, None, 1)
        .expect("load 07000004 (replay)");
    nf_view::attach_headless_sound(&mut scene);
    let mut replayed: Vec<WalkTickState> = Vec::with_capacity(inputs.len());
    for ti in &inputs {
        let (input, trigger, reload, use_key) = ti.to_parts();
        scene.sim.as_mut().unwrap().step_bond(input, trigger, reload, use_key);
        replayed.push(replay_state(&scene));
    }
    assert_eq!(replayed.len(), recorded.len(), "the replay ran the recorded tick count");
    // Per-tick equality, bit-exact: Bond pos/yaw/health, the drone frame, and the mission_end
    // latch (WalkTickState's PartialEq is field-exact — no tolerances).
    if let Some((i, (a, b))) = replayed.iter().zip(&recorded).enumerate().find(|(_, (a, b))| a != b) {
        panic!("replay diverged at tick {}: recorded pos {:?} yaw {:.6} health {:.3} frame {} end {:?} | replayed pos {:?} yaw {:.6} health {:.3} frame {} end {:?}",
            i + 1, a.pos, a.yaw, a.health, a.frame, a.mission_end,
            b.pos, b.yaw, b.health, b.frame, b.mission_end);
    }
    assert_eq!(replayed, recorded, "per-tick parity of the replayed run");
    // And the same mission_end tick, both directions of the comparison.
    let replay_end = replayed.iter().position(|s| s.mission_end.is_some()).map(|i| i + 1)
        .expect("mission_end latched in the replayed run");
    assert_eq!(replay_end, end_tick, "the same mission_end tick");
}
