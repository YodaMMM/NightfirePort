//! The stuck-guard sweep (task copter-real-path-walks, part 2, pass 1): a headless sim sweep over
//! the on-foot SP levels. For each level: cold load, Bond stands at the level's own spawn with no
//! input, 3600 ticks (60 s). Every 30 ticks each live drone's position is sampled; a drone whose
//! AI state wants it to TRAVEL (the MOVING set below — the port's state constants, each with the
//! game's decompile name in drone.rs `state`) yet whose position moves < 0.1 over the last 180
//! ticks (3 s) is a HIT. Per hit: level, drone index, position, state name, distance to the
//! nearest door, and the drone's home.
//!
//! PASS-1 SCOPE (honest): this is the UNALERTED surface — Bond never fires, no alarm; guards are
//! in their boot states (patrols, idles, ambush waits). The alerted passes (Bond walked into each
//! guard's sight through the real vision path [M EnemyLookForOpponent], or the alarm channel the
//! game's alert path raises) are the next step. The user symptom was combat-adjacent ("stuck in
//! the helipad section"), so pass 1 may miss the exact combat movers — the table still measures
//! the class on the shared path-following surface.

use nf_view::scene::Sim;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

/// The MOVING set: AI states whose update wants the drone to travel [M drone.rs `state` constants,
/// each citing its NDRone2_DSTATE_* decompile address]. Idle/aim/fire/wait states are excluded —
/// a drone parked in COMBAT_NO_MOVE (0x59) or AMBUSH_WAIT (0x38) is standing by design.
fn moving_name(s: u8) -> Option<&'static str> {
    use nf_game::drone::state::*;
    Some(match s {
        PATROL => "PATROL(0x07)",
        CIVILIAN_PATROL => "CIVILIAN_PATROL(0x14)",
        RETURN_TO_PATROL_PATH => "RETURN_TO_PATROL_PATH(0x15)",
        FOLLOW => "FOLLOW(0x25)",
        GO_TO_GOAL_ALLY => "GO_TO_GOAL_ALLY(0x28)",
        ALERT_TO_POSITION => "ALERT_TO_POSITION(0x62)",
        GO_TO_GOAL => "GO_TO_GOAL(0x63)",
        SEARCH_AREA => "SEARCH_AREA(0x65)",
        COMBAT => "COMBAT(0x58)",
        COMBAT_OUT_OF_RANGE => "COMBAT_OUT_OF_RANGE(0x5a)",
        INVESTIGATE => "INVESTIGATE(0x7e)",
        ENEMY_RUN_TO_POINT => "ENEMY_RUN_TO_POINT(0x85)",
        RUN_FOR_COVER => "RUN_FOR_COVER(0x8c)",
        HEARD_NOISE => "HEARD_NOISE(0xa2)",
        HEARD_NOISE_AWARE => "HEARD_NOISE_AWARE(0xa3)",
        HEARD_NOISE_SUSPECT => "HEARD_NOISE_SUSPECT(0xa4)",
        HEARD_NOISE_ALERT => "HEARD_NOISE_ALERT(0xa5)",
        _ => return None,
    })
}

const LEVELS: &[&str] = &[
    "07000001.bin", "07000002.bin", "07000003.bin", "07000004.bin", "07000005.bin",
    "07000006.bin", "07000007.bin", "07000008.bin", "07000009.bin", "0700000a.bin",
    "0700000b.bin", "0700000c.bin", "0700000d.bin", "07000011.bin", "07000012.bin",
    "07000013.bin", "07000014.bin", "07000015.bin", "07000016.bin", "0700001b.bin",
    "07000041.bin", "0700004a.bin",
];

#[test]
fn sweep_the_onfoot_sp_levels_for_moving_but_stuck_drones() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut total_hits = 0usize;
    for level in LEVELS {
        let Ok(scene) = nf_view::load(&iso, level, false) else { eprintln!("[sweep] {level}: LOAD FAILED"); continue };
        let mut sim = scene.sim.expect("sim");
        let n = sim.drones.list.len();
        // per-drone sample ring: last positions every 30 ticks + the state at each sample
        let mut samples: Vec<(Vec<[f32; 3]>, Vec<u8>)> = vec![(Vec::new(), Vec::new()); n];
        let mut printed: Vec<bool> = vec![false; n];
        for t in 0..3600u32 {
            sim.step_bond(Default::default(), false, false, false);
            if t % 30 != 0 { continue; }
            for (i, dr) in sim.drones.list.iter().enumerate() {
                let (positions, states) = &mut samples[i];
                positions.push(dr.pos);
                states.push(dr.state);
                if positions.len() > 7 { positions.remove(0); states.remove(0); }
                if positions.len() < 7 { continue; }
                let Some(name) = moving_name(states[6]) else { continue };
                // the drone stayed in a MOVING state the whole 3 s window
                if !states.iter().all(|&s| s == states[6]) { continue; }
                let p0 = positions[0];
                let p6 = positions[6];
                let d = ((p6[0] - p0[0]).powi(2) + (p6[1] - p0[1]).powi(2) + (p6[2] - p0[2]).powi(2)).sqrt();
                if d >= 0.1 || printed[i] { continue; }
                printed[i] = true;
                total_hits += 1;
                let door = sim.drones.doors.list.iter()
                    .map(|dd| ((dd.pos[0] - p6[0]).powi(2) + (dd.pos[1] - p6[1]).powi(2) + (dd.pos[2] - p6[2]).powi(2)).sqrt())
                    .fold(f32::INFINITY, f32::min);
                eprintln!(
                    "HIT|{level}|drone {i}|pos ({:.2},{:.2},{:.2})|state {name}|home ({:.2},{:.2},{:.2})|nearest door {door:.2}|hp {:.0}",
                    p6[0], p6[1], p6[2], dr.home[0], dr.home[1], dr.home[2], dr.health);
            }
        }
        eprintln!("[sweep] {level}: done, {n} drones");
    }
    eprintln!("[sweep] TOTAL hits {total_hits}");
    // The sweep is a measurement tool: it passes as long as it ran; the table goes to the report.
    assert!(total_hits < 1000, "sweep sanity bound");
}

/// The 07000005 courtyard/helipad focus (the user's report): the same measurement, but the level's
/// drones get their per-drone trail printed even below the hit bar, so the next pass can compare
/// what each stuck guard's path query did.
#[test]
fn sweep_07000005_courtyard_detail() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let scene = nf_view::load(&iso, "07000005.bin", false).expect("load 05");
    let mut sim = scene.sim.expect("sim");
    for t in 0..3600u32 {
        sim.step_bond(Default::default(), false, false, false);
        if t % 300 == 0 {
            for (i, dr) in sim.drones.list.iter().enumerate() {
                if dr.corpse || dr.health <= 0.0 { continue; }
                if let Some(name) = moving_name(dr.state) {
                    eprintln!("[05 t{t}] drone {i} ({:.2},{:.2},{:.2}) {name}", dr.pos[0], dr.pos[1], dr.pos[2]);
                }
            }
        }
    }
}

/// The alerted variant, honestly scoped: ONE load per level; Bond hops between the guards' own
/// posts every 600 ticks (10 s), so each post's neighbourhood gets a real sighting window through
/// the unmodified vision path (the score/LOS machine [M EnemyLookForOpponent]); a guard that SEES
/// Bond raises its own alerts through the game's states. A drone that then sits in a MOVING combat
/// state without travelling for 3 s is the user's symptom shape.
#[test]
fn sweep_alerted_pass_over_the_sp_levels() {
    let Some(iso) = iso() else { eprintln!("NF_ISO not set — skipping"); return };
    let mut total_hits = 0usize;
    for level in LEVELS {
        let Ok(scene) = nf_view::load(&iso, level, false) else { continue };
        let mut sim = scene.sim.expect("sim");
        let posts: Vec<[f32; 3]> = sim.drones.list.iter()
            .filter(|d| !d.corpse && d.health > 0.0)
            .map(|d| d.pos).collect();
        let n = sim.drones.list.len();
        let mut samples: Vec<(Vec<[f32; 3]>, Vec<u8>)> = vec![(Vec::new(), Vec::new()); n];
        let mut printed: Vec<bool> = vec![false; n];
        let mut hits_here = 0usize;
        for t in 0..3600u32 {
            if t % 600 == 0 {
                if let Some(post) = posts.get((t / 600) as usize) { sim.bond.pos = *post; }
            }
            sim.step_bond(Default::default(), false, false, false);
            if t % 30 != 0 { continue; }
            for (i, dr) in sim.drones.list.iter().enumerate() {
                let (positions, states) = &mut samples[i];
                positions.push(dr.pos);
                states.push(dr.state);
                if positions.len() > 7 { positions.remove(0); states.remove(0); }
                if positions.len() < 7 { continue; }
                let Some(name) = moving_name(states[6]) else { continue };
                if !states.iter().all(|&s| s == states[6]) { continue; }
                let p0 = positions[0];
                let p6 = positions[6];
                let d = ((p6[0] - p0[0]).powi(2) + (p6[1] - p0[1]).powi(2) + (p6[2] - p0[2]).powi(2)).sqrt();
                if d >= 0.1 || printed[i] { continue; }
                printed[i] = true;
                hits_here += 1;
                let door = sim.drones.doors.list.iter()
                    .map(|dd| ((dd.pos[0] - p6[0]).powi(2) + (dd.pos[1] - p6[1]).powi(2) + (dd.pos[2] - p6[2]).powi(2)).sqrt())
                    .fold(f32::INFINITY, f32::min);
                eprintln!(
                    "ALERT-HIT|{level}|post t{}|drone {i}|pos ({:.2},{:.2},{:.2})|state {name}|home ({:.2},{:.2},{:.2})|nearest door {door:.2}|hp {:.0}",
                    (t / 600) * 600, p6[0], p6[1], p6[2], dr.home[0], dr.home[1], dr.home[2], dr.health);
            }
        }
        eprintln!("[sweep-alerted] {level}: {hits_here} hits over {} posts", posts.len());
        total_hits += hits_here;
    }
    eprintln!("[sweep-alerted] TOTAL hits {total_hits}");
    assert!(total_hits < 100000, "sweep sanity bound");
}
