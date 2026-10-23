//! The guard-stuck sweep (`level-03-interactables-3b` part 2): every on-foot SP level, real
//! ticks through `Sim::step_bond`, and a log of every guard whose state wants it travelling yet
//! whose position stands still. The user report (2026-10-22, playing 0.1.17): "The exchange had
//! a few guards stuck in the helipad section", then "Not just 05" — a symptom; this sweep
//! measures it instead of guessing.
//!
//! The alert is the game's own path: Bond's trigger fires a real bullet whose noise enters the
//! guards' sound pipeline (`HandleSoundAlerts` → `DroneVision_EnemyAlerts` [M
//! NDrone2_HandleSoundAlerts 0x1424e0-family dispatch; the port's drain drone.rs `HandleSoundAlerts`])
//! — the same channel a real player's gunshot alerts a guard. Shots are fired straight from the
//! spawn yaw (walls take them); the sweep measures the ALERT walk, not marksmanship.
//!
//! The MOVING set is the port's `drone::state` constants whose modes travel by design [M
//! crates/nf-game/src/drone.rs `pub mod state`]: ALERT 0x05 (entering alert), PATROL 0x07 (the
//! patrol walk `NDrone2_DSTATE_Patrol`), ATTACK 0x56, COMBAT 0x58, COMBAT_OUT_OF_RANGE 0x5a
//! (`NDrone2_DSTATE_CombatOutOfRange` 0x164690), COMBAT_NEW_SIGHTING 0x5b, COMBAT_NO_SIGHT 0x5c,
//! COMBAT_WAIT 0x5e (`NDrone2_DSTATE_CombatWait` 0x164960), COMBAT_NO_ROUTE 0x5f, NO_OPPONENT
//! 0x60, GO_TO_GOAL 0x63 (`NDrone2_DSTATE_GoToGoalPosition`), HOSTAGE_GO_TO_GOAL 0x64,
//! ALT_ATTACK 0x72, INVESTIGATE 0x7e (`NDrone2_DSTATE_Investigate` 0x168070). A guard in one of
//! these that moves < 0.1 over a 3 s window, twice in a row (6 s), is a hit — one line per
//! episode. DRONE_STUCK 0x7f is the game's OWN stuck state [M `NDrone2_DSTATE_DroneStuck`
//! 0x165ef0 — ENTER falls through to CombatNoRoute 0x5f] and is logged on sight. A still guard
//! in IDLE 0x04 or an animation/fire state is normal and never logged.
#[path = "common/walker.rs"]
mod walker;

use nf_game::mission::LEVELS;
use nf_game::player::Input;

/// The sample window: 180 ticks = 3 s at the fixed 60 Hz [D docs/systems/input.md:126].
const WINDOW: u32 = 180;
/// The total sweep: 3600 ticks = 60 s per level.
const TOTAL: u32 = 3600;
/// The travel-wanting states [M drone.rs `pub mod state`, the addresses in the module docs].
const MOVING: &[u8] = &[0x05, 0x07, 0x56, 0x58, 0x5a, 0x5b, 0x5c, 0x5e, 0x5f, 0x60, 0x63, 0x64, 0x72, 0x7e];
/// The game's own stuck state [M drone.rs `pub mod state` DRONE_STUCK].
const DRONE_STUCK: u8 = 0x7f;

fn iso() -> Option<std::path::PathBuf> {
    std::env::var_os("NF_ISO").map(std::path::PathBuf::from)
}

fn settle(scene: &mut nf_view::scene::Scene, ticks: u32) {
    for _ in 0..3000 { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); if !scene.sim.as_ref().unwrap().disabled { break; } }
    for _ in 0..ticks { scene.sim.as_mut().unwrap().step_bond(Input::default(), false, false, false); }
}

#[test]
fn sweep_logs_stuck_movers_over_the_onfoot_sp_levels() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    for def in LEVELS.iter().filter(|d| d.level & 0xffff_0000 == 0x0700_0000 && d.level & 0xff < 0x50) {
        let name = format!("{:08x}.bin", def.level);
        let Ok(mut scene) = nf_view::load(iso.to_str().unwrap(), &name, false) else { eprintln!("SWEEP {name}: load failed"); continue };
        settle(&mut scene, 300);
        // The alert: two real shots 2 s apart at t=1 s and t=3 s of the sweep (the game's own
        // sound-alert path — see the module docs).
        let mut shots = 0u32;
        let n = scene.sim.as_ref().unwrap().drones.list.len();
        let mut last: Vec<Option<[f32; 3]>> = vec![None; n];
        let mut still: Vec<u32> = vec![0; n];
        let mut hits: Vec<String> = Vec::new();
        for tick in 0..TOTAL {
            let trigger = (tick == 60 || tick == 180) && shots < 2;
            if trigger { shots += 1; }
            scene.sim.as_mut().unwrap().step_bond(Input::default(), trigger, false, false);
            if tick % WINDOW != WINDOW - 1 { continue; }
            let sim = scene.sim.as_ref().unwrap();
            for (i, dr) in sim.drones.list.iter().enumerate() {
                if !dr.present || dr.corpse || !dr.enabled { still[i] = 0; last[i] = None; continue; }
                let moving = MOVING.contains(&dr.state);
                let stuck = dr.state == DRONE_STUCK;
                if !moving && !stuck { still[i] = 0; last[i] = None; continue; }
                if let Some(p) = last[i].replace(dr.pos) {
                    let d = ((dr.pos[0] - p[0]).powi(2) + (dr.pos[1] - p[1]).powi(2) + (dr.pos[2] - p[2]).powi(2)).sqrt();
                    if d < 0.1 { still[i] += 1; } else { still[i] = 0; }
                }
                if still[i] >= 2 || stuck {
                    // The blocking context: the nearest door within 3.0 (its state), and the
                    // guard's own goal point if the state carries one.
                    let door = sim.drones.doors.list.iter().enumerate()
                        .map(|(k, d)| (k, (d.pos[0]-dr.pos[0]).powi(2) + (d.pos[1]-dr.pos[1]).powi(2) + (d.pos[2]-dr.pos[2]).powi(2)))
                        .filter(|(_, q)| *q < 9.0)
                        .min_by(|a, b| a.1.total_cmp(&b.1))
                        .map(|(k, q)| (k, sim.drones.doors.list[k].state, q.sqrt()));
                    let goal = dr.goal.map(|g| format!("({:.1},{:.1},{:.1})", g[0], g[1], g[2]))
                        .unwrap_or_else(|| "-".into());
                    hits.push(format!(
                        "{name} drone#{i} placement#{} skin {:#x} state {:#04x} mode {} pos ({:.2},{:.2},{:.2}) still_windows {} goal {goal} {}",
                        dr.placement, dr.skin, dr.state, dr.mode, dr.pos[0], dr.pos[1], dr.pos[2], still[i],
                        door.map_or("no door within 3.0".into(), |(k, st, dist)| format!("door #{k} dist {dist:.2} state {st:#x}"))));
                    still[i] = 0; // one line per episode
                }
            }
        }
        eprintln!("SWEEP {}: {} drones, {} hit lines, {} shots", name, n, hits.len(), shots);
        for h in &hits { eprintln!("  HIT {h}"); }
    }
}
