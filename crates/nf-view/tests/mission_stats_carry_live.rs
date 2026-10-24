//! 987-mission-stats-carry: the pause SCORE counters survive the mission's level change.
//!
//! The game's carry [M ACTION.ELF]: the exit trigger's `Player_RamSave` 0x195670:10-11 copies
//! `PlrMissionStats[0]` (0x38) and `BondMoments` (0x14) into the `RamSave` staging buffer;
//! `Trigger_Activate` 0x1b2318:108-116 saves it before `ResetMap_LevelToLoad(next, 1, 0)`; on the
//! warm load `Player_RamLoad` 0x1957a8:15-16 copies both back (the whole body is gated on
//! `IsWarmReset != 0`), and `Mission_Init` 0x182270:59-66 wipes them ONLY when the loaded map IS
//! the mission's base map — 07000006 is order 1 of The Exchange (base 07000005), so the hop keeps
//! the tally. The pause SCORE tab reads the live record through `PlrStat_GetScore(1)`
//! [M 0x204408:413 → 0x1984b0:66-84 — the 0x07 maps refill the rows from `PlrMissionStats[0]` on
//! every call], so the carried record IS what the tab shows.
//!
//! The flow below is the real path: the walker drives Bond with the real input only — the
//! gatehouse porch plate (ch 0x5C), the crouched truck ride that completes the AND(0x70,0x71,0x72)
//! mux → ch 0x65 = bond move 101 (placement #316 [D placements 316/311-314/318]) — then a
//! teleport+fire kill through `step_bond` (the game's own fire path, the headless tests'
//! convention) for the Dispatched, all before the ScriptPlayer's end-level event latches the
//! castle-door exit (0x1D → 0x07000006 [D placement 118; level_end_live route_a]). The crossing
//! rides the App's exact functions: `warm_exit_save` (the capture `start_async_load`'s campaign
//! arm builds — `Player_RamSave`'s shape) → `load_with_carry` (the worker's call —
//! async_level_load_live pins its parity with the sync path) → the first `step_bond` tick, whose
//! `Mission_Init` latch applies `Player_RamLoad`'s tally restore.
//!
//! Run with: `NF_ISO=… cargo test -p nf-view --test mission_stats_carry_live -- --nocapture --test-threads=1`

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;
use walker::Walker;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn yaw_to(from: [f32; 3], at: [f32; 3]) -> f32 {
    // The port's forward is (sin yaw, 0, cos yaw) (0 faces +Z [M Bond's obj+0x54 doc]; the fire
    // ray builds right = (−cos yaw, 0, sin yaw) [M scene.rs run]).
    let (dx, dz) = (at[0] - from[0], at[2] - from[2]);
    dx.atan2(dz)
}

/// The first list-order live team-1 guard (fix_progress_live's proven kill target) within
/// `max_dy` of Bond's height (a level shot can hit) and at least `min_dist` from `away_from`
/// (horizontal) — test 1 keeps the truck chain's ch-0x72 park guard (~(28, 37, 44)) out of the
/// crosshair.
fn pick_guard(sim: &nf_view::scene::Sim, away_from: [f32; 2], min_dist: f32, max_dy: f32) -> Option<usize> {
    let by = sim.bond.pos[1];
    sim.drones.list.iter().enumerate()
        .filter(|(_, d)| d.present && !d.corpse && d.team == 1 && d.health > 0.0)
        .filter(|(_, d)| (d.pos[1] - by).abs() <= max_dy)
        .filter(|(_, d)| {
            let dx = d.pos[0] - away_from[0];
            let dz = d.pos[2] - away_from[1];
            (dx * dx + dz * dz) >= min_dist * min_dist
        })
        .map(|(i, _)| i)
        .next()
}

/// Fires at the guard `gi` until it dies (the P2K is semi-auto — the trigger is a fresh press per
/// shot, so the run pulses fire/release, re-aiming each tick [fix_progress_live's idiom]).
fn shoot_guard_dead(sim: &mut nf_view::scene::Sim, gi: usize) -> u32 {
    sim.bond.pos = [sim.drones.list[gi].pos[0] + 2.2, sim.drones.list[gi].pos[1], sim.drones.list[gi].pos[2]];
    let mut fired = 0u32;
    while sim.drones.list[gi].health > 0.0 && fired < 300 && sim.drones.mission.status() < 2 {
        let g = sim.drones.list[gi].pos;
        sim.bond.yaw = yaw_to(sim.bond.pos, g);
        // The fire ray's forward is [cp·sy, sp, cp·cy] [M scene.rs run] — pitch tracks the
        // height difference (the asin of dy/horiz, the camera's own look-at shape, scene.rs:3433).
        let dy = g[1] - sim.bond.pos[1];
        let horiz = ((g[0] - sim.bond.pos[0]).powi(2) + (g[2] - sim.bond.pos[2]).powi(2)).sqrt().max(1e-4);
        sim.bond.pitch = (dy / horiz).clamp(-1.0, 1.0).asin();
        sim.step_bond(Input { aiming: true, ..Default::default() }, true, false, false);
        fired += 1;
        let g = sim.drones.list[gi].pos;
        sim.bond.yaw = yaw_to(sim.bond.pos, g);
        let dy = g[1] - sim.bond.pos[1];
        let horiz = ((g[0] - sim.bond.pos[0]).powi(2) + (g[2] - sim.bond.pos[2]).powi(2)).sqrt().max(1e-4);
        sim.bond.pitch = (dy / horiz).clamp(-1.0, 1.0).asin();
        sim.step_bond(Input { aiming: true, ..Default::default() }, false, false, false);
        fired += 1;
    }
    fired
}

/// The Exchange hop: a Dispatched and a Bond Move earned on 07000005 through the real input path,
/// the castle-door exit latched, the warm load into 07000006 — and the counters hold on the new
/// level's live record (what the pause SCORE tab draws from).
#[test]
fn the_tally_survives_the_exchange_hop() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut w = Walker::new(nf_view::load(&iso, "07000005.bin", false).expect("load 07000005"));
    w.skip_cutscenes = false;
    {
        let sim = w.scene.sim.as_ref().unwrap();
        let st = sim.drones.progress.stats();
        assert_eq!((st.dispatched, st.bond_moments), (0, 0), "zero tally at the mission start");
    }
    w.wait(30);
    // The walk home position (post-intro) — the kill teleports Bond away and the walk legs must
    // start from the same spot bond_moves_live's proven legs start from.
    let home = {
        let sim = w.scene.sim.as_ref().unwrap();
        (sim.bond.pos, sim.bond.yaw)
    };

    // ── The Dispatched: the teleport+fire kill through `step_bond` (the game's own fire path),
    // before the walk — the first list-order live guard kept ≥40 from the truck chain's ch-0x72
    // park guard (~(28, 37, 44) [D level_end_live route_a]) so the ride's mux stays intact; the
    // walker's `top_up_health` (the default) carries the alarmed walk+ride.
    let fired = {
        let sim = w.scene.sim.as_mut().unwrap();
        let gi = pick_guard(sim, [28.0, 44.0], 40.0, 3.0).expect("a live guard away from the park");
        eprintln!("[carry] kill target {} at {:?}", gi, sim.drones.list[gi].pos);
        let r = shoot_guard_dead(sim, gi);
        eprintln!("[carry] after the kill: fired {r}, level_exit {:?}", sim.drones.level_exit);
        r
    };
    {
        let sim = w.scene.sim.as_ref().unwrap();
        assert_eq!(sim.drones.progress.stats().dispatched, 1,
            "PlrStat_LogEnemyDispatched per player kill [M 0x145bd0:57-59] (fired {fired})");
        assert!(sim.drones.level_exit.is_none(), "the kill must not latch a level exit");
    }
    // Back to the walk home — the kill spot's nav routes elsewhere (a gate-0 exit trigger sits on
    // the way from it); the walk legs below are bond_moves_live's, from the level start.
    {
        let sim = w.scene.sim.as_mut().unwrap();
        sim.bond.pos = home.0;
        sim.bond.yaw = home.1;
    }

    // ── The Bond Move: the gatehouse porch plate → the crouched truck ride (bond_moves_live's legs).
    w.walk_nav_to([102.0, 25.0, 110.5], "courtyard by the gatehouse porch", 1.6);
    assert!(w.climb_stairs([99.5, 26.0, 107.0], "onto the gatehouse porch", 1.0, 600),
        "Bond must climb the gatehouse porch step");
    w.walk_to(walker::Wp::new(99.0, 26.0, 106.6).r(0.7), "onto the ch-0x5C plate");
    let mut saw_5c = 0u32;
    for _ in 0..2400u32 {
        w.tick(Input::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.channels.on(0x5c) { saw_5c = w.total_ticks; break; }
    }
    assert!(saw_5c > 0, "the porch plate must fire ch 0x5C (the hand-off)");
    let mut input = Input::default();
    let mut prev_dis = true;
    let mut crouch_armed = false;
    let mut award = false;
    for _ in 0..6000u32 {
        {
            let sim = w.scene.sim.as_ref().unwrap();
            let dis = sim.disabled;
            if !dis && prev_dis && sim.bond.pos[0] < 95.0 { crouch_armed = true; }
            prev_dis = dis;
        }
        input.crouch = if crouch_armed && !w.scene.sim.as_ref().unwrap().bond.crouched { crouch_armed = false; true } else { false };
        w.tick(input, false);
        if w.scene.sim.as_ref().unwrap().drones.progress.stats().bond_moments >= 1 { award = true; break; }
        if w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { break; }
    }
    assert!(award, "the truck ride must award bond move 101 (the AND(0x70,0x71,0x72) mux → ch 0x65)");

    // ── The exit latches: the truck chain's ScriptPlayer → event 0x11 → ch 0x1D → the castle-door
    // LOAD_LEVEL (placement 118) auto-fires → level 0x07000006.
    let mut exit_latched = false;
    for _ in 0..1200u32 {
        w.tick(Input::default(), false);
        if w.scene.sim.as_ref().unwrap().drones.level_exit.is_some() { exit_latched = true; break; }
    }
    assert!(exit_latched, "the truck chain must latch the level exit");
    assert_eq!(w.scene.sim.as_ref().unwrap().drones.level_exit, Some(0x0700_0006),
        "the castle-door exit loads 07000006 [D placement 118]");

    // ── The crossing: the App's exact functions. `warm_exit_save` is the capture
    // `start_async_load`'s campaign arm hands the worker (`Player_RamSave` 0x195670's shape,
    // tally included); `load_with_carry` is the worker's call; the first tick runs the
    // `Mission_Init` latch (part 1 → no wipe; `Player_RamLoad`'s tally restore applies).
    let save = nf_view::warm_exit_save(w.scene.sim.as_ref().unwrap());
    let carried = save.tally.expect("the warm save carries the tally [M 0x195670:10-11]");
    assert_eq!((carried.stats.dispatched, carried.stats.bond_moments), (1, 1),
        "the save snapshots the earned counters");
    assert_eq!(carried.bond_moments[0], 1, "BondMoments[0x65−0x65] rides the save");
    drop(w);

    let mut scene6 = nf_view::load_with_carry(&iso, "07000006.bin", false, Some(&save))
        .expect("the warm load of 07000006");
    let sim6 = scene6.sim.as_mut().expect("a level scene");
    assert_eq!(sim6.level, 0x0700_0006, "the next Exchange level loaded");
    assert!(sim6.drones.tally_carry.is_some(), "the load seeded the tally carry");
    sim6.step_bond(Input::default(), false, false, false); // the Mission_Init latch tick
    {
        let st = sim6.drones.progress.stats();
        assert_eq!(st.dispatched, 1, "the Dispatched survived the hop (the pause SCORE tab's row 1 source)");
        assert_eq!(st.bond_moments, 1, "the Bond Move survived the hop (the tab's row 0 source)");
        assert_eq!(sim6.drones.progress.bond_moments[0], 1, "the moment flag survived (BondMoments[0x65−0x65])");
        assert_eq!(st.spawned, carried.stats.spawned,
            "the record copies back whole [M 0x1957a8:15-16 — the 0x38 memcpy]");
        assert_eq!(st.timer_run, 0, "Player_Init 0x1910f8:187's LogTimerUnpause re-stamp cleared +0x34");
        assert!(st.elapsed >= carried.stats.elapsed, "the elapsed timer resumed, no wrap");
    }
    // Continued logging lands on the carried record: a kill on 07000006 stacks on the hop's tally.
    // The party level opens disabled (the entry intro) — warm until Bond's input is live.
    for _ in 0..600u32 {
        if !sim6.disabled { break; }
        sim6.step_bond(Input::default(), false, false, false);
    }
    let fired6 = {
        let gi = pick_guard(sim6, [0.0, 0.0], 0.0, 3.0).expect("07000006 has live guards at Bond's level");
        eprintln!("[carry] 07000006 kill target {gi} at {:?}, disabled {}", sim6.drones.list[gi].pos, sim6.disabled);
        shoot_guard_dead(sim6, gi)
    };
    assert_eq!(sim6.drones.progress.stats().dispatched, 2,
        "PlrStat_LogEnemyDispatched keeps counting on the carried record (fired {fired6})");

    // The restart-LEVEL serve (the fail screen's option 0) restores the entry snapshot — which on
    // 07000006 holds the carried tally [M P_ENDMISSION_Handler 0x216618:19-21 → Player_RamLoad(…,1)].
    let entry6 = nf_view::entry_checkpoint_save(sim6);
    assert_eq!(entry6.tally.as_ref().expect("the entry checkpoint carries the tally").stats.dispatched, 2);
    let serve = nf_view::fail_restart_save(Some(&entry6), sim6);
    assert_eq!(serve.tally.as_ref().expect("the restart-level serve carries the tally").stats.dispatched, 2,
        "a restart-level keeps the mission's counters [M 0x216618:21]");
}

/// Sister: a mission RESTART zeroes the tally. The pause menu's Restart loads the BASE map cold
/// [M C_GCPAUSE_Handler 0x204408:139-155 → ResetMap_LevelToLoad(BaseMapHCode, 0, 1)], the cold
/// load skips `Player_RamLoad`'s whole body [M 0x1957a8:9], and `Mission_Init` on the base map
/// wipes PlrMissionStats + BondMoments [M 0x182270:59-66 → PlrStat_ResetForMission 0x1993a0].
#[test]
fn a_mission_restart_zeroes_the_tally() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut scene = nf_view::load(&iso, "07000005.bin", false).expect("load 07000005");
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    let gi = pick_guard(sim, [0.0, 0.0], 0.0, 3.0).expect("a live guard");
    shoot_guard_dead(sim, gi);
    assert_eq!(sim.drones.progress.stats[0].dispatched, 1, "the kill logged before the restart");
    drop(scene);

    // The restart arm: the cold reload of the base map (`level_exit_cold`, save = None
    // [M lib.rs's arm ← 0x204408:139]) — no restore, and the part-0 `Mission_Init` wipe.
    let mut scene2 = nf_view::load_with_carry(&iso, "07000005.bin", false, None).expect("the cold restart load");
    let sim2 = scene2.sim.as_mut().unwrap();
    assert!(sim2.drones.tally_carry.is_none(), "a cold load carries no tally [M 0x1957a8:9]");
    sim2.step_bond(Input::default(), false, false, false); // the Mission_Init latch tick
    let st = &sim2.drones.progress.stats[0];
    assert_eq!(st.dispatched, 0, "PlrStat_ResetForMission wiped the counters [M 0x182270:65]");
    assert_eq!(st.bond_moments, 0);
    assert_eq!(sim2.drones.progress.bond_moments, [0; 10], "BondMoments wiped [M 0x1993a0]");
    assert!(st.spawned >= 13, "the fresh level's spawns re-logged ([L] ee_start_05: 13 at the start)");
}
