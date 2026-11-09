//! pretrans-arms-2: the `Drone+0x38` root-motion zero of `DroneAnim_PreTransCallback` 0x13b490.
//!
//! The game [M]: `NDrone2_PreDroneControl` 0x148db0:23 clears `+0x38` at the top of every drone
//! frame; the state machine's UPDATE then runs (`Drone_Control` 0x139268:48-66) and a setter
//! may set it — JustStand case 3 [M 0x173fc0:16], StandFiddle case 3 [M 0x1687c0:29], OpenDoor
//! ENTER/UPDATE [M 0x1681f8:25/:74] (EXIT clears, :71), UnderCoverInit case 3 [M 0x169cc0:37],
//! AimStandFire case 3 with the aiming class [M 0x166328:122], the run-anim arm of
//! `DroneWeap_AimTarget_IsOpponent` [M 0x178ae0:52]; the anim tick that follows
//! (0x139268:69-91 → AnimFrameResolve → the callback) zeroes the root motion while it stands
//! (0x13b490:96-118, disasm 0x13b588-0x13b638), unless the body-contact latch arm ran.
//!
//! The pin: a patrolling 07000005 guard mid-walk (his walk anim playing, root motion carrying
//! him) is put in JustStand without its ENTER, so the walk anim keeps playing: JustStand's
//! UPDATE zeroes every frame's root step, and he stays put. Back in Patrol he walks again on
//! the very next frames — the zero is per frame, not a latch that sticks (the rejected
//! s/pretrans-arms made it sticky). Red on the old code: JustStand's UPDATE was unhandled and
//! the walk anim kept moving him.

use nf_game::drone::state;
use nf_game::player::Input;

fn iso() -> String { std::env::var("NF_ISO").expect("NF_ISO") }

fn flat(a: [f32; 3], b: [f32; 3]) -> f32 { ((a[0] - b[0]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() }

#[test]
fn just_stand_zeroes_the_walk_root_each_frame_and_patrol_walks_again() {
    let mut scene = nf_view::load(&iso(), "07000005.bin", false).expect("load 07000005");
    let sim = scene.sim.as_mut().unwrap();
    // Settle with Bond idle at the spawn, then pick a guard walking his patrol: state Patrol,
    // the walk anim (0x37) playing, moving this tick.
    let mut pick = None;
    'find: for _ in 0..60 {
        let before: Vec<[f32; 3]> = sim.drones.list.iter().map(|d| d.pos).collect();
        for _ in 0..10 { sim.step_bond(Input::default(), false, false, false); }
        for (i, d) in sim.drones.list.iter().enumerate() {
            if d.present && d.enabled && !d.corpse && d.state == state::PATROL && d.anim.state == 0x37
                && d.anim.outgoing.is_empty() && flat(d.pos, before[i]) > 0.05 {
                pick = Some(i);
                break 'find;
            }
        }
    }
    let g = pick.expect("a patrolling guard walking on 07000005");
    let walk = |sim: &mut nf_view::scene::Sim, n: usize| -> f32 {
        let mut path = 0.0;
        for _ in 0..n {
            let p = sim.drones.list[g].pos;
            sim.step_bond(Input::default(), false, false, false);
            path += flat(sim.drones.list[g].pos, p);
        }
        path
    };
    let before = walk(sim, 40);
    assert!(before > 0.1, "the guard walks his patrol: {before:.3} in 40 ticks");
    // JustStand without its ENTER: the walk anim keeps playing, UPDATE sets +0x38 each frame.
    sim.drones.list[g].state = nf_game::kiko::JUST_STAND;
    let anim = sim.drones.list[g].anim.state;
    let held = walk(sim, 40);
    assert_eq!(sim.drones.list[g].anim.state, anim, "still the walk anim (no ENTER ran)");
    assert!(held < 0.01, "JustStand zeroes the walk's root motion every frame [M 0x173fc0:16 + 0x13b490:117-118]: moved {held:.4}");
    // Back to Patrol: the next frame's PreDroneControl clear [M 0x148db0:23] lets him walk at once.
    sim.drones.list[g].state = state::PATROL;
    let again = walk(sim, 40);
    assert!(again > 0.1, "Patrol walks again the next frames — the zero is per frame, not sticky: {again:.3}");
}

/// The second setter: `DroneWeap_AimTarget_IsOpponent(0, 1)`'s run arm [M 0x178ae0:47-53]. A
/// 07000001 guard caught running (anim state 0x1e, his opponent set) is put in RollLeftCrouch
/// without its ENTER, so the run anim keeps playing: the roll's UPDATE calls IsOpponent(0, 1)
/// [M 0x167e90:24], the run arm turns him half way to the opponent (`+0x694 = yaw −
/// AngleDifference(atan, yaw)·0.5`, :48-49) instead of all the way, and with the 0.4 ahead clear
/// (`CanMoveToRelativePos`, :50) zeroes his run root motion. Red on the old code: the roll's
/// UPDATE faced the opponent fully and the run anim carried him on.
#[test]
fn a_running_guard_in_an_aim_state_turns_half_way_and_stops_his_run_root() {
    let mut scene = nf_view::load(&iso(), "07000001.bin", false).expect("load 07000001");
    let sim = scene.sim.as_mut().unwrap();
    let mut pick = None;
    'find: for _ in 0..4000 {
        let before: Vec<[f32; 3]> = sim.drones.list.iter().map(|d| d.pos).collect();
        sim.step_bond(Input::default(), false, false, false);
        for (i, d) in sim.drones.list.iter().enumerate() {
            let off = nf_game::trig::angle_difference(d.yaw_to_opp, d.yaw).abs();
            if d.present && d.enabled && !d.corpse && d.opp_bond && d.anim.state == 0x1e
                && d.anim.outgoing.is_empty() && flat(d.pos, before[i]) > 0.02 && off > 0.2 {
                pick = Some(i);
                break 'find;
            }
        }
    }
    let g = pick.expect("a running guard with an opponent on 07000001");
    sim.drones.list[g].state = state::ROLL_LEFT_CROUCH;
    let p = sim.drones.list[g].pos;
    let (yaw, to_opp) = (sim.drones.list[g].yaw, sim.drones.list[g].yaw_to_opp);
    sim.step_bond(Input::default(), false, false, false);
    let dr = &sim.drones.list[g];
    assert_eq!(dr.anim.state, 0x1e, "still the run anim (no ENTER ran)");
    let half = yaw - nf_game::trig::angle_difference(to_opp, yaw) * 0.5;
    let want = nf_game::trig::angle_difference(half, dr.desired_yaw).abs();
    let full = nf_game::trig::angle_difference(dr.yaw_to_opp, dr.desired_yaw).abs();
    eprintln!("guard {g}: yaw {yaw:.3} to_opp {to_opp:.3} desired {:.3} (half {half:.3}) moved {:.4}", dr.desired_yaw, flat(dr.pos, p));
    assert!(want < 0.05 && full > 0.05, "the run arm sets the HALF-way yaw [M 0x178ae0:48-49]: desired {:.3}, half {half:.3}, opp {:.3}", dr.desired_yaw, dr.yaw_to_opp);
    assert!(flat(dr.pos, p) < 0.005, "the run arm's +0x38 zeroes the run root step [M 0x178ae0:50-53 + 0x13b490:117-118]: moved {:.4}", flat(dr.pos, p));
}
