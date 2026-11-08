//! engage-range: the 07000011 / 07000014 engage-range overrides and the stand-off they give the
//! start-corridor guards.
//!
//! - `NDrone2_DefaultInit` overrides the class draw of engage (+0xF0) and combat (+0xFC) range on two
//!   levels, by the anim set +0xDA: 07000011 sets 6/9 (5/8 for set 9), 07000014 sets engage 10 (7
//!   for set 9, set 5 keeps its draw) [M 0x14b300:429-457, raw 0x14bb60-0x14bc1c].
//! - The player's AI target (`NDrone2_NavNodeCache` 0x155cd8 → `AINetwork_BuildAITarget` 0x15ac78 →
//!   `AINetwork_NodesForPosition` 0x1586a8) searches with radius 0 among the nodes filed under the
//!   player's own cel (`AIPath_Link2Cel` 0x1d2ec8 lists, walked by `AINetwork_NodeSearchCel`
//!   0x156e18). Bond's 07000011 start cel holds no node of the guards' path, so the target has no
//!   goal and `AINetwork_DoAStarPath` returns 4 [M 0x1571b0:52-68]: the guards cannot route to him.
//!   `DroneFunc_CombatState` then keeps a firing guard firing (no CombatOutOfRange exit without a
//!   route [M 0x148338 case 0x10]) and sends a moving one to CombatNoRoute 0x5f [M case 2 default].
//! - The run-to point's own accuracy (+0x14, 1.0 on both office points [D]) is the goal radius
//!   [M 0x153168:42-45, 0x15acf8, 0x158b10:104].
//!
//! The fresh-boot capture has the idle start's guards holding 5.8-9 from Bond and firing
//! [L docs/live/2026-11-01-07000011-start-corridor-guards.md]. Before this change the port's
//! guards, given the override, walked to 4.1 from Bond at the AI boundary and stood there blind;
//! without it they closed to 3.8.

use nf_view::scene::Scene;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn load(diff: u32) -> Scene {
    nf_view::load_with_carry_difficulty(&iso(), "07000011.bin", false, None, diff).expect("load 07000011")
}

/// The overrides on every drone of both levels; sister: 0700000d keeps the class draw (12 and up).
#[test]
fn engage_ranges_follow_the_level() {
    let scene = nf_view::load(&iso(), "07000011.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    assert!(!sim.drones.list.is_empty());
    for d in &sim.drones.list {
        let want = if d.anim_set == 9 { (5.0, 8.0) } else { (6.0, 9.0) };
        assert_eq!((d.engage_range, d.combat_range), want, "drone {} set {} [M 0x14bb8c-0x14bbdc]", d.index, d.anim_set);
    }
    let scene = nf_view::load(&iso(), "07000014.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    for d in sim.drones.list.iter().filter(|d| d.anim_set != 5 && d.anim_set != 9 && d.mode != 0xc && d.mode != 0x11) {
        assert_eq!(d.engage_range, 10.0, "drone {} set {} [M 0x14bbe0-0x14bc1c]", d.index, d.anim_set);
    }
    let scene = nf_view::load(&iso(), "0700000d.bin", false).expect("load");
    let sim = scene.sim.as_ref().unwrap();
    assert!(sim.drones.list.iter().all(|d| d.engage_range >= 12.0), "sister: the class draw stands");
}

/// Bond's start cel holds no node of path 1, so the player target has neither a found node nor a
/// fallback and a route from either guard fails with 4. Sister: with Bond moved into the corridor
/// (beside node 148) the rebuilt target has a goal.
#[test]
fn the_start_cel_gives_the_guards_no_route() {
    let mut scene = load(2);
    for _ in 0..3 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    {
        let sim = scene.sim.as_ref().unwrap();
        let cel = sim.bond.cel.expect("Bond is linked to a cel");
        let tg = &sim.drones.target;
        for p in [126usize, 127] {
            let dr = sim.drones.list.iter().find(|d| d.placement == p && d.present).unwrap();
            let path = dr.nav_path.flatten().unwrap_or_else(|| sim.nav.nav_path_for(dr.feet()).expect("a path"));
            assert!(sim.nav.paths[path].nodes.iter().all(|n| sim.world.find_cel(n.pos) != Some(cel)),
                "no node of path {path} is filed under the start cel {cel} [M 0x1d2ec8]");
            let goal = tg.nodes.get(path).copied().unwrap_or_default();
            assert!(goal.found.is_none() && goal.fallback.is_none(), "p{p}: the player target has no goal on path {path}: {goal:?}");
            let r = nf_game::nav::Route::to_position(&sim.nav, &sim.world, Some(path), dr.feet(), tg.pos, &goal, &[], &|_, _| false);
            assert_eq!(r.err(), Some(4), "p{p}: no route to Bond [M 0x1571b0:52-68]");
        }
    }
    let n148 = scene.sim.as_ref().unwrap().nav.paths[1].nodes[148].pos;
    {
        let sim = scene.sim.as_mut().unwrap();
        sim.bond.pos = [n148[0] + 0.5, sim.bond.pos[1], n148[2] + 0.5];
        sim.bond.vel = [0.0; 3];
    }
    for _ in 0..3 { scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false); }
    let sim = scene.sim.as_ref().unwrap();
    let goal = sim.drones.target.nodes.get(1).copied().unwrap_or_default();
    assert!(goal.found.is_some() || goal.fallback.is_some(), "sister: in the corridor the target has a goal: {goal:?}");
}

/// The run-to point's accuracy 1.0 [D] is the arrival radius: on the tick a guard leaves GoToGoal
/// it stands within 1.0 (+ a tick's stride) of its owner point, where the fixed 2.0 left it 1.8 off.
#[test]
fn the_guards_run_to_their_point_within_its_accuracy() {
    let mut scene = load(2);
    let pts = [(126usize, [1.378f32, 16.715]), (127, [-1.476, 18.206])];
    let mut last = [None::<[f32; 3]>; 2];
    let mut left = [false; 2];
    for _ in 0..400u32 {
        scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
        let sim = scene.sim.as_ref().unwrap();
        for (k, (p, _)) in pts.iter().enumerate() {
            let dr = sim.drones.list.iter().find(|d| d.placement == *p && d.present).unwrap();
            if dr.state == nf_game::drone::state::GO_TO_GOAL { last[k] = Some(dr.pos); } else if last[k].is_some() { left[k] = true; }
        }
    }
    let sim = scene.sim.as_ref().unwrap();
    for (k, (p, pt)) in pts.into_iter().enumerate() {
        assert!(left[k], "p{p} left GoToGoal by t400");
        let a = sim.ai_points.iter().find(|a| (a.pos[0] - pt[0]).abs() < 0.01 && (a.pos[2] - pt[1]).abs() < 0.01).expect("the owner point [D]");
        assert_eq!(a.accuracy, 1.0, "p{p}'s point accuracy [D]");
        let q = last[k].unwrap();
        let d = ((q[0] - pt[0]).powi(2) + (q[2] - pt[1]).powi(2)).sqrt();
        assert!(d < 1.1, "p{p} arrived {d:.2} from its point, inside the 1.0 accuracy [M 0x158b10:104]");
    }
}

/// With no route the guards hold where they stand and fight: from t250 to t1000 neither comes
/// within 5.5 of the idle Bond (the game's are 5.8-9 [L]), and both hit him. At d1 #126 alone.
#[test]
fn the_corridor_guards_hold_off_the_idle_bond() {
    let start = [-0.40f32, 25.83];
    for (diff, guards) in [(1u32, &[126usize][..]), (2, &[126, 127][..]), (3, &[126, 127][..])] {
        let mut scene = load(diff);
        let mut near = vec![f32::MAX; guards.len()];
        let mut far = vec![0.0f32; guards.len()];
        for t in 1..=1000u32 {
            scene.sim.as_mut().unwrap().step_bond(Default::default(), false, false, false);
            if t < 250 { continue; }
            let sim = scene.sim.as_ref().unwrap();
            if sim.drones.health.health <= 0.0 { break; }
            for (k, &p) in guards.iter().enumerate() {
                let dr = sim.drones.list.iter().find(|d| d.placement == p && d.present).unwrap();
                let d = ((dr.pos[0] - start[0]).powi(2) + (dr.pos[2] - start[1]).powi(2)).sqrt();
                near[k] = near[k].min(d);
                far[k] = far[k].max(d);
            }
        }
        let sim = scene.sim.as_ref().unwrap();
        for (k, &p) in guards.iter().enumerate() {
            assert!(near[k] >= 5.5, "d{diff}: #{p} came within {:.2} of the idle Bond; the game's hold 5.8-9 [L]", near[k]);
            assert!(far[k] <= 11.0, "d{diff}: #{p} stood {:.2} off", far[k]);
        }
        assert!(!sim.drones.player_hits.is_empty(), "d{diff}: the guards hit the idle Bond from there [L]");
    }
}
