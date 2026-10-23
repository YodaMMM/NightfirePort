//! The stuck-guard sweep (task ally-heard-noise-attack-2). For every level in scope: load, let
//! Bond stand at the spawn, fire the weapon once every 90 ticks — the muzzle noise reaches the
//! drones through the game's own alert path (the port's `HandleSoundAlerts` / the heard-noise
//! chain 0xa2→0xa3→0xa4→0xa5→ATTACK [M the DSTATE table at 0x29b300; docs/systems/ally-heard-noise.md]).
//! A drone whose state WANTS travel (the MOVING set below — each entry a `NDrone2_StateFuncs[]`
//! DSTATE at [L 0x29b300 + state·4]) yet whose XZ position moves < 0.1 across a 180-tick (3 s)
//! window is logged once: index, skin, team, state, position, route index/length, blocked flag,
//! weapon. The sweep only MEASURES; any fix lands in shared code, never per level.

#[path = "common/walker.rs"]
mod walker;
use walker::Walker;
use nf_game::player::Input as WalkInput;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// The states whose DSTATE UPDATE requests travel. Names = the port's `state` module
/// (crates/nf-game/src/drone.rs); the DSTATE function addresses follow each entry.
const MOVING: &[(u8, &str)] = &[
    (0x1b, "ENEMY_MISSION alerted route walk [M 0x15bf18:32-37 arm]"),
    (0x28, "ALLY_GO_TO_GOAL_POSITION run-to-point [M 0x15ea28:54-58]"),
    (0x56, "ATTACK [M 0x164210]"),
    (0x5a, "COMBAT_OUT_OF_RANGE [M 0x164690]"),
    (0x5c, "COMBAT_NO_SIGHT [M 0x164838]"),
    (0x68, "COMBAT_MOVE (the movable exit) [M 0x1643e8:49-101]"),
    (0x8c, "RUN_FOR_COVER [M DroneFunc_CombatState 0x148338, WORKLOG 0023]"),
    (0xa4, "HEARD_NOISE move arm [M the 0xa2..0xa5 chain]"),
    (0xa5, "HEARD_NOISE_SUSPECT walk [M the 0xa2..0xa5 chain]"),
];

#[test]
fn the_stuck_guard_sweep_measures_the_on_foot_levels() {
    let Ok(iso) = std::env::var("NF_ISO") else { eprintln!("sweep: no NF_ISO — skip"); return };
    let levels: Vec<String> = std::env::var("NF_SWEEP_LEVELS").unwrap_or_else(|_| {
        "07000005,07000006,07000007,07000008,07000001".to_string()
    }).split(',').map(|s| s.trim().to_string()).collect();
    let ticks: u32 = std::env::var("NF_SWEEP_TICKS").ok().and_then(|v| v.parse().ok()).unwrap_or(3600);
    for level in &levels {
        let scene = match nf_view::load(&iso, &format!("{level}.bin"), false) {
            Ok(s) => s,
            Err(e) => { eprintln!("[sweep] {level}: load failed: {e}"); continue; }
        };
        let mut w = Walker::new(scene);
        eprintln!("[sweep] === {level} ===");
        // Per-drone window: (start frame, start pos, last logged state).
        let n = w.scene.sim.as_ref().unwrap().drones.list.len();
        let mut win: Vec<Option<(u32, [f32; 3])>> = vec![None; n];
        let mut logged: Vec<bool> = vec![false; n];
        for f in 0..ticks {
            let fire = f % 90 == 30; // the periodic muzzle noise
            let input = WalkInput::default();
            if fire { w.tick_fire(input); } else { w.tick(input, false); }
            let sim = w.scene.sim.as_ref().unwrap();
            let frame = sim.drones.frame;
            for (i, d) in sim.drones.list.iter().enumerate() {
                if !d.present || d.corpse || !d.enabled { win[i] = None; continue; }
                let moving = MOVING.iter().find(|(s, _)| *s == d.state).map(|(_, n)| *n);
                match (moving, win[i]) {
                    (Some(name), None) => win[i] = Some((frame, d.pos)),
                    (Some(name), Some((f0, p0))) => {
                        let dx = ((d.pos[0] - p0[0]).powi(2) + (d.pos[2] - p0[2]).powi(2)).sqrt();
                        if frame - f0 >= 180 && dx < 0.1 && !logged[i] {
                            logged[i] = true;
                            eprintln!("[sweep] {level} STUCK drone {i} skin {:#x} team {} state {:#x} ({name}) pos ({:.2},{:.2},{:.2}) moved {dx:.3} in {} ticks route {:?} blocked {} weapon {:#x} health {:.0}",
                                d.skin, d.team, d.state, d.pos[0], d.pos[1], d.pos[2], frame - f0,
                                d.route.as_ref().map(|r| (r.index, r.nodes.len())), d.route_blocked, d.weapon, d.health);
                        }
                        if dx >= 0.1 { win[i] = Some((frame, d.pos)); logged[i] = false; }
                    }
                    (None, _) => { win[i] = None; logged[i] = false; }
                }
            }
        }
        let stuck = logged.iter().filter(|l| **l).count();
        eprintln!("[sweep] {level}: {stuck} stuck movers over {ticks} ticks");
    }
}
