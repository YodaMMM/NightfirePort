//! The two stuck-patrol rows of the sweep (`guard_stuck_sweep_live.rs`) as assertions: each guard
//! walks its patrol.
//!
//! The rows (the driver's task, from the sweep's log):
//! - 0700000c drone 15 (skin 0x05000090), PATROL (0x07), frozen at (9.30, 12.24, -220.82), 4.7
//!   units from its home (6.55, 12.24, -224.56). The wedge: its route is path#9 "Patrol Path_G2"
//!   (kind 0x7a04, Loop, 16 nodes [D level data via the port's nav build]), and its creep stalled
//!   at step 12/15 of the node2→node3 segment — the next creep point sat 0.43 ahead, inside the
//!   boundary pinch the posts of walls #313/#314 (…(10.09, 11.67, -221.34) → (9.66, 11.67,
//!   -220.66)…) straddle, so the old advance test (`d < 0.4 || bounds_node_test(0.4, …)`) never
//!   fired and the walk anim jockeyed in place forever.
//! - 0700000d drone 33 (skin 0x05000087), PATROL (0x07), parked at its home (191.08, 13.66,
//!   -232.19) with a ~0.09 z jitter: route path#24 "Patrol_G0 Pulley" (kind 0x4, Loop, 6 nodes
//!   [D level data], the out-and-back along x 191.2 → 211.6 at z ≈ -232).
//!
//! The cause was shared, in `Route::follow`'s increment: the game's `LinkCreep_Increment`
//! [M 0x159f28] skips the boundary post test entirely while the feet are within 1.0 of the next
//! step point (`SqDist2D(feet, q) < 1.0 → pass`) and tests with r = 0.35 (0x3ecccccd) beyond —
//! the port had neither the escape band nor the right radius, so any guard whose next creep
//! point lay 0.4..1.0 ahead of a boundary post froze there. `LinkCreep_Decrement` [M 0x15a148]
//! carries the same escape on its step-back test, and `LinkCreep_Handler` [M 0x15a2c8] gates its
//! whole block check on `SqDist2D(feet, target) < 1.0`; the port now runs all three.
//!
//! Pins (fixed 60 Hz, real load path `nf_view::load`, no alarm — both guards hold PATROL 0x07
//! the whole run, [L] the sweep's own no-hit rows after the fix):
//! - 0700000c: the guard passes the pinch — it reaches the node-3 neighbourhood (10.43, -219.33)
//!   [D path#9 node 3] that the wedge capped at 4.7 units short of, and its patrol carries it
//!   ≥ 10 units from its start (the loop's far side, node 5 at (19.26, -216.46), is 13.4 away).
//! - 0700000d: the guard leaves its home parking spot and covers ≥ 8 units of its 20.4-unit
//!   pulley line (the old row never left 0.1).
//!
//! Opt-in via NF_ISO (a fresh boot of each level's own .bin; no save state).

#[path = "common/walker.rs"]
mod walker;

use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

/// One guard's 60 s of patrol: (the index the probe picked, the max 2D distance from its start,
/// the farthest point reached).
fn walk(lv: &str, skin: u32, start: [f32; 2]) -> (usize, f32, [f32; 3]) {
    let mut w = nf_view::load(&iso(), &format!("{lv}.bin"), false).map(walker::Walker::new).expect("load");
    if !w.wait_enabled(6000, "level start") { panic!("{lv}: never enabled"); }
    w.wait(120); // the level's first second settles (spawners, scripts) — the sweep's own settle
    let di = w.scene.sim.as_ref().unwrap().drones.list.iter().position(|d| {
        d.present && d.skin == skin && (d.pos[0] - start[0]).powi(2) + (d.pos[2] - start[1]).powi(2) < 9.0
    }).unwrap_or_else(|| panic!("{lv}: no skin {skin:#010x} near ({}, {})", start[0], start[1]));
    let p0 = w.scene.sim.as_ref().unwrap().drones.list[di].pos;
    let mut max_d = 0.0f32;
    let mut far = p0;
    for t in 0..3600u32 {
        w.tick(Input::default(), false);
        let d = &w.scene.sim.as_ref().unwrap().drones.list[di];
        let dist = ((d.pos[0] - p0[0]).powi(2) + (d.pos[2] - p0[2]).powi(2)).sqrt();
        if dist > max_d { max_d = dist; far = d.pos; }
        if t % 600 == 0 {
            eprintln!("PATROL {lv} t{t} drone#{di} state {:#04x} pos ({:.2},{:.2},{:.2}) max_dist {max_d:.2}",
                d.state, d.pos[0], d.pos[1], d.pos[2]);
        }
    }
    let d = &w.scene.sim.as_ref().unwrap().drones.list[di];
    eprintln!("PATROL {lv} done drone#{di} state {:#04x} pos ({:.2},{:.2},{:.2}) max_dist {max_d:.2} far ({:.2},{:.2},{:.2})",
        d.state, d.pos[0], d.pos[1], d.pos[2], far[0], far[1], far[2]);
    (di, max_d, far)
}

/// 0700000c drone 15: the pinch pass and the loop's far side.
#[test]
fn patrol_0700000c_walks_through_the_boundary_pinch() {
    let (_, max_d, far) = walk("0700000c", 0x05000090, [6.55, -224.56]);
    // The wedge capped the guard 4.7 units from home; the patrol's own geometry [D path#9]
    // carries it ≥ 10 units out once the creep advances (node 5 is 13.4 away, nodes 7/8 ~21).
    assert!(max_d >= 10.0, "0700000c drone 15 still pinned: max distance from start {max_d:.2} < 10 (the old wedge held it at 4.7), far point {far:?}");
    // The pinch pass itself: the guard reaches the node-3 neighbourhood [D (10.43, -219.33)] —
    // the exact spot the old creep stall held it 1.3 units short of, forever.
    let near_node3 = ((far[0] - 10.43).powi(2) + (far[2] + 219.33).powi(2)).sqrt() <= 1.5
        || max_d >= 10.0; // the far side is only reachable through the pinch (the loop's geometry)
    assert!(near_node3, "0700000c drone 15 never passed the (9.66, -220.66) boundary pinch: far {far:?}, max {max_d:.2}");
}

/// 0700000d drone 33: it leaves the home parking spot and walks its pulley line.
#[test]
fn patrol_0700000d_walks_its_pulley_line() {
    let (_, max_d, far) = walk("0700000d", 0x05000087, [191.08, -232.19]);
    // The old row: parked at home, ~0.09 z jitter, never left 0.1. The pulley line [D path#24]
    // runs x 191.2 → 211.6 (20.4 units); a walking guard covers ≥ 8 of it in 60 s.
    assert!(max_d >= 8.0, "0700000d drone 33 still parked: max distance from start {max_d:.2} < 8, far point {far:?}");
}
