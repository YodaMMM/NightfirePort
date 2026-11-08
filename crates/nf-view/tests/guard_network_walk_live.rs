//! Guards walk on the AI network alone, on every level that has one.
//!
//! The game's guard move has no world-collision gate: root motion moves the guard
//! (`DroneAnim_PreTransCallback` 0x13b490) and the only horizontal limit is the AI-network bounds
//! push on the drone's sphere (`NDrone2_Collision` 0x140e90:169-174 →
//! `AINetwork_GetBoundsPushVectorForSphere` 0x157bd0), gated on `drone+0x4F8 < 0`, the bit
//! `DroneInit_Collision` 0x14b1e8:39 sets [M]. Its contact walk drops world hits
//! (`NDrone2_DealWithObjHit` 0x13fff0 returns on a HITDATA with no object) [M]. The combat-move
//! choice (`NDrone2_CanStepLeft` 0x14f640 and the whole Can* family) asks `NDrone2_MoveTest`
//! 0x155330, which ends in `AINetwork_BoundsTest` [M].
//!
//! The sweep per level: difficulty 3, every unaware watching guard alarmed
//! every 5 s through msg 0x1e [M `DroneVision_EnemyAlerts`
//! 0x177c18:113-114], 60 s of ticks with Bond held 6 units in front of a
//! different live guard every 5 s. Two failures:
//! - a guard in a moving state whose step the mover rejects with no shut door near: scenery
//!   stopping a guard the game would walk on (one row per spot);
//! - a guard in a step or strafe state for 2 s that never gets 0.1 from where it started: a
//!   step the bounds refuse, picked and pushed back every tick.
//!
//! Opt-in: set `NF_ISO`.

#[path = "common/walker.rs"]
mod walker;

use nf_game::drone::{msg, state as st};
use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

const LEVELS: [&str; 17] = ["07000001", "07000002", "07000004", "07000005", "07000006", "07000007", "07000008", "07000009", "0700000a", "0700000b", "0700000c", "0700000d", "07000012", "07000013", "07000014", "07000015", "07000016"];

fn mover(s: u8) -> bool {
    matches!(s, st::ATTACK | st::RUN_FOR_COVER | st::ALERT_TO_POSITION | st::SEARCH_AREA | st::GO_TO_GOAL
        | st::PATROL | st::COMBAT_OUT_OF_RANGE | st::UNDER_COVER_RETURN)
}

fn stepper(s: u8) -> bool {
    matches!(s, st::STEP_AIM_LEFT | st::STEP_AIM_RIGHT | st::STRAFE_AIM_LEFT | st::STRAFE_AIM_RIGHT
        | st::STRAFE_DODGE_LEFT | st::STRAFE_DODGE_RIGHT)
}

/// msg 0x1e (ALARM) on every unaware watching guard [M 0x177c18:113-114].
fn alarm(sim: &mut nf_view::scene::Sim) {
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
            && matches!(d.state, st::IDLE | st::ALERT | st::PATROL) {
            sim.drones.list[i].send(&data, &mut sim.drones.rng, frame, msg::ALARM);
        }
    }
}

/// (scenery stalls, steps in place) on one level; None when it has no AI-network bounds.
fn sweep(iso: &str, lv: &str) -> Option<(Vec<String>, Vec<String>)> {
    let mut w = nf_view::load_with_carry_difficulty(iso, &format!("{lv}.bin"), false, None, 3).map(walker::Walker::new).ok()?;
    if !w.scene.sim.as_ref().unwrap().nav.bounds { eprintln!("{lv}: no bounds"); return None; }
    w.wait_enabled(6000, "level start");
    w.wait(120);
    let sim = w.scene.sim.as_mut().unwrap();
    let mut spot = sim.bond.pos;
    let mut next = 0usize;
    alarm(sim);
    let n = sim.drones.list.len();
    let mut stall = vec![([0.0f32; 3], 0u32); n];
    let mut step = vec![([0.0f32; 3], 0u32, 0u8); n];
    let (mut stalls, mut steps) = (Vec::new(), Vec::new());
    let (mut moving, mut blocked, mut stepping) = (0u32, 0u32, 0u32);
    for t in 1..3600u32 {
        // Every 5 s Bond stands 6 units from the next live guard (with a floor there), so the
        // guards fight and move all over the level.
        if t % 300 == 1 {
            alarm(sim);
            for _ in 0..n {
                next = (next + 1) % n;
                let d = &sim.drones.list[next];
                if !(d.present && d.enabled && !d.corpse && d.health > 0.0 && d.team != 2) { continue; }
                let p = [d.pos[0] + 6.0 * d.yaw.sin(), d.pos[1], d.pos[2] + 6.0 * d.yaw.cos()];
                let Some(f) = sim.world.floor_below([p[0], p[1] + 1.0, p[2]], 3.0) else { continue };
                if !sim.world.line_of_sight([d.pos[0], d.pos[1] + 0.6, d.pos[2]], [p[0], f + 1.6, p[2]], nf_game::world::LOS_PROBE_8) { continue; }
                spot = [p[0], f + 1.03, p[2]];
                sim.bond.pos = spot;
                break;
            }
        }
        sim.step_bond(Input::default(), false, false, false);
        sim.drones.health.health = 100.0;
        sim.bond.pos[0] = spot[0];
        sim.bond.pos[2] = spot[2];
        let doors = sim.world.dynamic_boxes();
        let near_door = |p: [f32; 3]| doors.iter().any(|(_, lo, hi)|
            (lo[0] - 1.5..hi[0] + 1.5).contains(&p[0]) && (lo[2] - 1.5..hi[2] + 1.5).contains(&p[2]));
        for (i, d) in sim.drones.list.iter().enumerate().take(n) {
            let live = d.present && d.enabled && !d.corpse && d.health > 0.0;
            if live && mover(d.state) { moving += 1; if d.blocked_reach != 0.0 { blocked += 1; } }
            if live && stepper(d.state) { stepping += 1; }
            let dist = |a: [f32; 3]| ((d.pos[0] - a[0]).powi(2) + (d.pos[2] - a[2]).powi(2)).sqrt();
            // The mover rejected this tick's step with no shut door near: scenery held him.
            if live && mover(d.state) && d.blocked_reach != 0.0 && !near_door(d.pos) {
                if stall[i].1 == 0 || dist(stall[i].0) >= 1.0 {
                    stalls.push(format!("#{i} st {:#x} at ({:.2},{:.2},{:.2})", d.state, d.pos[0], d.pos[1], d.pos[2]));
                    stall[i] = (d.pos, t);
                }
            }
            if !live || !stepper(d.state) { step[i].1 = 0; }
            else if step[i].1 == 0 || step[i].2 != d.state || dist(step[i].0) >= 0.1 { step[i] = (d.pos, t, d.state); }
            else if t - step[i].1 >= 120 {
                if !near_door(d.pos) { steps.push(format!("#{i} st {:#x} at ({:.2},{:.2},{:.2})", d.state, d.pos[0], d.pos[1], d.pos[2])); }
                step[i].1 = t;
            }
        }
    }
    eprintln!("{lv}: guard-ticks moving {moving} (rejected {blocked}), stepping {stepping}");
    eprintln!("{lv}: {} scenery stall(s) {:?}; {} step(s) in place {:?}", stalls.len(), stalls, steps.len(), steps);
    Some((stalls, steps))
}

#[test]
fn guards_walk_on_the_network_on_every_level() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut bad = Vec::new();
    for lv in LEVELS {
        if let Some((stalls, steps)) = sweep(&iso, lv) {
            if !stalls.is_empty() || !steps.is_empty() { bad.push((lv, stalls.len(), steps.len())); }
        }
    }
    assert!(bad.is_empty(), "guards held by scenery or stepping in place (level, stalls, steps): {bad:?}");
}
