//! `NDrone2_FindOpponent` 0x1424e0's class-0x3B arm and the team-2 close sight probe, on the
//! one shipped level that carries both a team-2 drone and a class-0x3B object: 07000008 (the
//! ally #21, dtype 0xC; the helicopter #45). [D census: `nf_view::load` over `dev::LEVELS`,
//! team-2 drones on 07000001/02/08/16, copters on 07000005/08/0B/12; 08 is the only overlap.]
//!
//! The game [M]:
//! - Class 0x3B is the helicopter alone: `Copter_Create` 0x17d340:62 is the ELF's only
//!   `obj+0xFF = 0x3b` store; its state short `obj+0xF4` starts 0 [:63] and reads 1 from the
//!   first post-gate tick [M 0x17dbb8:115-137].
//! - A team-2 drone walks the control objects first [M 0x1424e0:123-147]: class 0x3B with
//!   `0 < +0xF4 < 4` and the `Vec_DirToRot` pitch of (object − drone) in [−0.5235988,
//!   0.17453294] becomes the opponent, with no range, no sight test, no budget.
//! - A held class-0x3B opponent re-validates with `NDrone2_CanSeeObject` (one budget probe) and
//!   drops once `+0xF4 > 3` [M :152, :176-177].
//! - `DroneVision_OpponentVisibility` 0x175af0:24-30: under 2.0 to the opponent with a sight
//!   range, `NDrone2_CanSeeObject(drone, +0x170, 5, 0)` runs first and a clear ray is 1.0,
//!   whatever the cone says. The opponent is the team-2 drone's own pick, not Bond.
//!
//! Opt-in via NF_ISO.

use nf_game::drone::{Data, Team2Opp};
use nf_game::player::Input;

fn iso() -> Option<String> { std::env::var("NF_ISO").ok() }

fn load08() -> nf_view::scene::Scene {
    nf_view::load(&iso().unwrap(), "07000008.bin", false).expect("load 07000008")
}

macro_rules! data {
    ($sim:expr) => {
        Data { tables: &$sim.tables, props: &$sim.props, scripts: &$sim.scripts, weapons: &$sim.weapons, world: &$sim.world, level: $sim.level, sequences: &$sim.seqs, skins: &$sim.skins, skeletons: &$sim.skeletons, nav: &$sim.nav, covers: &$sim.covers, tuning: &$sim.tuning, hitboxes: &$sim.hitboxes, volumes: &$sim.volumes, sfx: None, models: &$sim.models, surfaces: None, voices: None, difficulty: 1,
            ai_points: &$sim.ai_points }
    };
}

fn ally(sim: &nf_view::scene::Sim) -> usize {
    sim.drones.list.iter().position(|d| d.present && d.team == 2).expect("07000008's team-2 ally")
}

fn put_copter(sim: &mut nf_view::scene::Sim, p: [f32; 3], state: u16) {
    let c = sim.drones.copters.iter_mut().find(|c| c.placement == 45).expect("copter #45");
    c.matrix[3][0] = p[0]; c.matrix[3][1] = p[1]; c.matrix[3][2] = p[2];
    c.state = state;
}

/// Through the real tick: the gate channel 5 opens the flight (state 1) and the ally takes the
/// flying copter as her opponent while it sits in her pitch band. The pre-port code had no
/// class-0x3B arm: with no guard near, her opponent stayed empty and her aim point her own
/// position (the `SetOpponent(0)` fold).
#[test]
fn the_ally_takes_the_flying_copter_through_the_tick() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set, skipping"); return };
    let mut scene = load08();
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..300 { sim.step_bond(Input::default(), false, false, false); }
    let z = ally(sim);
    let f = sim.drones.frame;
    sim.drones.channels.set(5, 1, f);
    let mut took = None;
    for t in 0..120 {
        sim.step_bond(Input::default(), false, false, false);
        let c = sim.drones.copters.iter().find(|c| c.placement == 45).unwrap();
        let zd = &sim.drones.list[z];
        let to = [zd.opp_target[0] - c.pos()[0], zd.opp_target[1] - c.pos()[1], zd.opp_target[2] - c.pos()[2]];
        // One tick of flight separates the pick (before the copter moves) from the read.
        if c.state == 1 && (to[0] * to[0] + to[1] * to[1] + to[2] * to[2]).sqrt() < 1.0 { took = Some(t); break; }
    }
    let t = took.expect("the ally's aim point follows the flying copter within 120 ticks of the gate");
    let zd = &sim.drones.list[z];
    assert_eq!(zd.opp_copter, Some(45), "t{t}: her +0x170 is the copter");
    assert_eq!(zd.opp_drone, None);
    let cp = sim.drones.copters.iter().find(|c| c.placement == 45).unwrap().pos();
    let d3 = ((cp[0] - zd.pos[0]).powi(2) + (cp[1] - zd.pos[1]).powi(2) + (cp[2] - zd.pos[2]).powi(2)).sqrt();
    assert!((zd.dist - d3).abs() < 2.0, "t{t}: the opponent distance is the copter's ({} vs {d3})", zd.dist);
}

/// The walk's gates, one call at a time: the state short 1..3 and the pitch band, no budget.
#[test]
fn the_class_0x3b_walk_gates_on_state_and_pitch() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set, skipping"); return };
    let mut scene = load08();
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    let z = ally(sim);
    let zp = sim.drones.list[z].pos;
    // (offset, state, taken): pitch = -atan2(dy, 10).
    let cases: [([f32; 3], u16, bool); 9] = [
        ([10.0, 1.0, 0.0], 1, true),     // pitch -0.0997
        ([10.0, 1.0, 0.0], 2, true),
        ([10.0, 1.0, 0.0], 3, true),
        ([10.0, 1.0, 0.0], 0, false),    // the pre-gate state [M 0x17d340:63]
        ([10.0, 1.0, 0.0], 4, false),    // crash/arrive
        ([10.0, -1.7, 0.0], 1, true),    // pitch +0.168, inside +0.1745
        ([10.0, -1.8, 0.0], 1, false),   // pitch +0.178, past it
        ([10.0, 5.7, 0.0], 1, true),     // pitch -0.518, inside -0.5236
        ([10.0, 5.9, 0.0], 1, false),    // pitch -0.533, past it
    ];
    for (off, st, taken) in cases {
        put_copter(sim, [zp[0] + off[0], zp[1] + off[1], zp[2] + off[2]], st);
        {
            let zd = &mut sim.drones.list[z];
            zd.opp_drone = None; zd.opp_copter = None;
        }
        let d = data!(sim);
        let before = sim.drones.los_probes;
        let (picks, _) = sim.drones.team2_opponent_picks(&d);
        let got = picks[z].map(|p| p.0);
        if taken {
            assert_eq!(got, Some(Team2Opp::Copter(45)), "offset {off:?} state {st}: the walk takes it");
            assert_eq!(sim.drones.los_probes, before, "offset {off:?}: the walk casts nothing [M :132-141]");
        } else {
            assert_ne!(got, Some(Team2Opp::Copter(45)), "offset {off:?} state {st}: the walk refuses it");
        }
    }
}

/// A held copter re-validates with one probe, and a state past 3 drops it whatever the ray says.
#[test]
fn a_held_copter_is_revalidated_and_dropped_past_state_3() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set, skipping"); return };
    let mut scene = load08();
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..30 { sim.step_bond(Input::default(), false, false, false); }
    let z = ally(sim);
    let zp = sim.drones.list[z].pos;
    // Out of the pitch band (straight overhead): the walk refuses, the held arm decides.
    let cp = [zp[0] + 1.0, zp[1] + 6.0, zp[2]];
    for (st, keep_if_seen) in [(1u16, true), (4, false), (5, false)] {
        put_copter(sim, cp, st);
        {
            let zd = &mut sim.drones.list[z];
            zd.opp_drone = None; zd.opp_copter = Some(45);
        }
        let d = data!(sim);
        let ex = sim.drones.copter_stream_hashes(sim.drones.copters.iter().position(|c| c.placement == 45).unwrap());
        let seen = d.world.line_of_sight_excluding(sim.drones.list[z].head(), cp, nf_game::world::LOS_GUARD_VISION, &ex);
        let before = sim.drones.los_probes;
        let (picks, _) = sim.drones.team2_opponent_picks(&d);
        assert!(sim.drones.los_probes.wrapping_sub(before) >= 1, "state {st}: the held re-validation probes [M :152]");
        let kept = picks[z].is_some_and(|p| p.0 == Team2Opp::Copter(45));
        assert_eq!(kept, keep_if_seen && seen, "state {st} (ray clear: {seen}): kept only while seen and state <= 3 [M :176-177]");
    }
}

/// The close probe: a team-2 drone 1.5 from her drone opponent, turned away from him (the cone
/// formula reads 0), gets visibility 1.0 from the head ray [M 0x175af0:24-30]. The pre-port code
/// skipped the probe for team 2 and stored the formula's 0.
#[test]
fn the_team2_close_probe_sees_the_adjacent_opponent() {
    let Some(_) = iso() else { eprintln!("NF_ISO not set, skipping"); return };
    let mut scene = load08();
    let sim = scene.sim.as_mut().unwrap();
    for _ in 0..300 { sim.step_bond(Input::default(), false, false, false); }
    let z = ally(sim);
    let g = sim.drones.list.iter().position(|d| d.present && d.team == 1 && !d.corpse && d.health > 0.0)
        .expect("a live team-1 guard on 07000008");
    sim.drones.list[g].enabled = true; // the AI-active bit the scan's alive test reads [M 0x1430b8]
    let zp = sim.drones.list[z].pos;
    let mut best = None;
    for _ in 0..1 {
        sim.drones.list[g].pos = [zp[0] + 1.5, zp[1], zp[2]];
        {
            let zd = &mut sim.drones.list[z];
            zd.opp_drone = Some(g); zd.opp_copter = None;
            zd.yaw = nf_game::trig::atan2_approx(-1.0, 0.0); // facing -x, away from the guard at +x
        }
        sim.step_bond(Input::default(), false, false, false);
        let zd = &sim.drones.list[z];
        best = Some((zd.opp_drone, zd.dist, zd.visibility, zd.vision_on, zd.sight_range, zd.half_fov, zd.yaw));
    }
    let (opp, dist, vis, von, sr, hf, yaw) = best.unwrap();
    eprintln!("[close] opp {opp:?} dist {dist:.2} vis {vis} vision_on {von} sight_range {sr} half_fov {hf} yaw {yaw:.2}");
    assert!(von && sr > 0.0, "the ally's sight pipeline is on (vision bit, sight range)");
    assert_eq!(opp, Some(g), "she still holds the adjacent guard (re-validated by its ray)");
    assert!(dist < 2.0, "the opponent distance is under 2.0");
    assert_eq!(vis, 1.0, "the close ray's 1.0, not the cone formula's 0 [M 0x175af0:26-29]");
}
