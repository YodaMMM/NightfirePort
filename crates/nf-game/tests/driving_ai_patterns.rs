//! The per-pattern hostile steering checked against the DRIVING.ELF decompile
//! (docs/systems/driving-ai.md §11): the aim point and pursue/road switch of FUN_00131770, the
//! dispatch FUN_00130508, the chase drive FUN_00133f60 (+ FUN_00133238, FUN_001349c0), the
//! avoidance FUN_00137310 / FUN_001369a0 / FUN_00135b10 / FUN_00135cf0, the turret
//! FUN_0012f170, the world cap FUN_002012d0, the helicopter leaf rules FUN_00138ef0 /
//! FUN_0013b4c8 / FUN_0013b4f8 and the gunner sweep FUN_00122a68. Every expected number is the
//! game's constant or arithmetic worked by hand; the last test runs each pattern on its mission
//! from the disc (NF_ISO).

use nf_formats::crp::Crp;
use nf_formats::driveai::{AiCommand, MapAi, TYPE_VEHICLE};
use nf_formats::iso9660::IsoImage;
use nf_formats::refpack;

use nf_game::driving::ai::{gated_stop, lookahead, DriveRng, RoadNav, SimCar, NAV_KIND_TRAFFIC};
use nf_game::driving::hostile::{chase_target_speed, ChaseSpeedInputs};
use nf_game::driving::patterns::*;
use nf_game::driving::road::{LaneSet, RoadNet};
use nf_game::driving::{Car, CarInput, CarParams, FlatWorld, Rigid, DT};

fn body(pos: [f32; 3], vel: [f32; 3]) -> Body {
    Body { pos, vel, side: [1.0, 0.0, 0.0], up: [0.0, 1.0, 0.0], fwd: [0.0, 0.0, 1.0], box_z: 2.0, box_x: 1.0 }
}

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() <= 1e-4
}
fn near3(a: [f32; 3], b: [f32; 3]) -> bool {
    near(a[0], b[0]) && near(a[1], b[1]) && near(a[2], b[2])
}

// ---------------------------------------------------------------- sin (FUN_002cbd50)

#[test]
fn sin_of_turns() {
    for x in [0.0f32, 0.1, 0.25, 0.4, 0.5, 0.75, 0.9, 1.3, -0.2] {
        let want = (x * std::f32::consts::TAU).sin();
        // The game's polynomial stops at t⁹: its error grows to 0.0033 at ±π (x = 0.5).
        assert!((sin_turns(x) - want).abs() < 3.5e-3, "sin_turns({x}) = {} vs {want}", sin_turns(x));
    }
    assert!((sin_turns(0.25) - 1.0).abs() < 2e-4);
    assert!((sin_turns(0.5) - 0.003_270_4).abs() < 1e-5, "the truncated series, not libm");
}

// ---------------------------------------------------------------- the aim point (0x131770)

fn aim_in() -> AimInputs {
    AimInputs {
        profile: 0xd,
        pattern: PATTERN_ROAD,
        target_mode: 0,
        goal: [7.0, 0.0, 7.0],
        has_target: true,
        target_pos: [0.0, 0.0, 50.0],
        target_vel: [0.0, 0.0, 20.0],
        lost: false,
        self_body: body([0.0, 0.0, 0.0], [0.0, 0.0, 10.0]),
        lead: 15.0,
        follow: 15.0,
        prev_dist: 40.0,
        prev_aim: [9.0, 0.0, 9.0],
        nav_carrot: [3.0, 0.0, 30.0],
        nav_10: [0.0, 0.0, 30.0],
    }
}

#[test]
fn the_pattern_3_lead_and_the_velocity_lead() {
    // k = min(max(15, 0.2), 20) = 15 → 50 + 15.
    let a = aim_point(&aim_in());
    assert!(near3(a.point, [0.0, 0.0, 65.0]), "{:?}", a.point);
    assert!(near(a.dist, 65.0) && near3(a.dir, [0.0, 0.0, 1.0]));
    assert!(!a.rejoin, "20·15 > follow 15");
    // The lead floor 0.2, and the re-join ask (20·0.2 = 4 ≤ 15, last distance 40 > max(15, 15)).
    let mut i = aim_in();
    i.lead = 0.0;
    let a = aim_point(&i);
    assert!(near3(a.point, [0.0, 0.0, 50.2]), "{:?}", a.point);
    assert!(a.rejoin);
    i.prev_dist = 15.0;
    assert!(!aim_point(&i).rejoin, "15 ≤ max(follow, 15)");
    // The lead is capped by the target speed: lead 30 at speed 20 → 20.
    let mut i = aim_in();
    i.lead = 30.0;
    assert!(near3(aim_point(&i).point, [0.0, 0.0, 70.0]));
    // Below speed 5 no lead.
    i.target_vel = [0.0, 0.0, 4.9];
    assert!(near3(aim_point(&i).point, [0.0, 0.0, 50.0]));
    // Pattern 2: pos + v·0.2 = 50 + 4, never a re-join.
    let mut i = aim_in();
    i.pattern = PATTERN_PURSUE;
    let a = aim_point(&i);
    assert!(near3(a.point, [0.0, 0.0, 54.0]) && !a.rejoin);
    // Profile 0xf: the bare target position.
    let mut i = aim_in();
    i.profile = 0xf;
    assert!(near3(aim_point(&i).point, [0.0, 0.0, 50.0]));
    // Nav direction +0x210 = n(nav+0x10 − pos).
    assert!(near3(aim_point(&aim_in()).nav_dir, [0.0, 0.0, 1.0]));
}

#[test]
fn goal_mode_and_the_lost_aims() {
    let mut i = aim_in();
    i.target_mode = 1;
    assert!(near3(aim_point(&i).point, [7.0, 0.0, 7.0]), "mode 1 drives to the goal");
    let mut i = aim_in();
    i.has_target = false;
    let a = aim_point(&i);
    assert!(a.lost, "no target, mode 0 → lost");
    // Lost, road pattern: the carrot, distance from the previous aim (9, 0, 9).
    assert!(near3(a.point, [3.0, 0.0, 30.0]));
    assert!(near(a.dist, (81.0f32 + 81.0).sqrt()));
    // Lost pursuit: the target, or own pos + forward.
    let mut i = aim_in();
    i.pattern = PATTERN_PURSUE;
    i.lost = true;
    assert!(near3(aim_point(&i).point, [0.0, 0.0, 50.0]));
    i.has_target = false;
    assert!(near3(aim_point(&i).point, [0.0, 0.0, 1.0]));
    // Lost with a target on a road pattern asks for the re-join; profile 0xe never does.
    let mut i = aim_in();
    i.lost = true;
    assert!(aim_point(&i).rejoin);
    i.profile = 0xe;
    assert!(!aim_point(&i).rejoin);
}

fn sw() -> SwitchInputs {
    SwitchInputs {
        profile: 0xd,
        pattern: PATTERN_ROAD,
        lost: false,
        now: 100,
        hold_until: 0,
        nav_repeat: false,
        dist: 40.0,
        dir: [0.0, 0.0, 1.0],
        follow: 15.0,
        nav_dir_30: [0.0, 0.0, 1.0],
        target_speed: 20.0,
        self_fwd: [0.0, 0.0, 1.0],
        self_speed: 12.0,
        target_lane: 1.0,
        target_road_half_width: 6.0,
        target_road_dir: [0.0, 0.0, 1.0],
    }
}

#[test]
fn the_pursue_road_switch() {
    // On the road, target ahead along it, moving: stay.
    assert_eq!(pursue_switch(&sw()).unwrap(), SwitchOut::default());
    // The target off the road's direction (dot < 0.5) → pattern 2 for 60 ticks.
    let mut i = sw();
    i.dir = [1.0, 0.0, 0.0];
    let o = pursue_switch(&i).unwrap();
    assert_eq!((o.pattern, o.mode_7c, o.hold_until), (Some(PATTERN_PURSUE), Some(3), Some(160)));
    // A slow target inside the follow distance → pattern 2.
    let mut i = sw();
    i.target_speed = 4.0;
    i.dist = 10.0;
    assert_eq!(pursue_switch(&i).unwrap().pattern, Some(PATTERN_PURSUE));
    i.dist = 15.0;
    assert_eq!(pursue_switch(&i).unwrap().pattern, None, "15 is not < follow 15");
    // The hold timer blocks, then clears.
    let mut i = sw();
    i.hold_until = 101;
    assert!(pursue_switch(&i).is_none());
    i.hold_until = 100;
    assert_eq!(pursue_switch(&i).unwrap().hold_until, Some(0));
    // Pattern 2: the target on its road (|lane| 1 < 6, along 1 > 0.7) at speed ≥ 10 → re-join.
    let mut i = sw();
    i.pattern = PATTERN_PURSUE;
    let mut o = pursue_switch(&i).unwrap();
    assert!(o.rejoin && o.hold_until == Some(160));
    assert!(!finish_rejoin(&mut o, 0.69));
    assert!(finish_rejoin(&mut o, 0.7));
    assert_eq!((o.pattern, o.mode_7c), (Some(PATTERN_ROAD), Some(2)));
    i.self_speed = 9.9;
    assert!(!pursue_switch(&i).unwrap().rejoin, "below 10 only a lost target re-joins");
    i.lost = true;
    assert!(pursue_switch(&i).unwrap().rejoin);
    let mut i = sw();
    i.pattern = PATTERN_PURSUE;
    i.target_lane = 6.0;
    assert!(!pursue_switch(&i).unwrap().rejoin, "the target off its road");
    // The +0x27c latch.
    assert_eq!(road_heading_latch(false, 0.7), (true, false));
    assert_eq!(road_heading_latch(false, 0.69), (false, false));
    assert_eq!(road_heading_latch(true, -0.16), (false, true));
    assert_eq!(road_heading_latch(true, -0.15), (true, false));
}

// ---------------------------------------------------------------- the dispatch (0x130508)

fn disp() -> DispatchInputs {
    DispatchInputs {
        state: 5,
        pattern: PATTERN_PURSUE,
        alive: 1.0,
        stop_query: false,
        stop_byte: false,
        target_speed: 30.0,
        avoid_on: true,
        nav_repeat: false,
        self_pos: [1.0, 0.0, 2.0],
        dir: [0.0, 0.0, 1.0],
        lookahead: 9.5,
        carrot: [4.0, 0.0, 4.0],
    }
}

#[test]
fn the_dispatch_per_pattern() {
    assert_eq!(dispatch(&disp()), Arm::Chase { speed: 30.0, aim: [1.0, 0.0, 11.5], avoid: Avoid::Chase });
    let mut i = disp();
    i.state = 2;
    assert!(matches!(dispatch(&i), Arm::Chase { avoid: Avoid::Traffic, .. }));
    i.avoid_on = false;
    assert!(matches!(dispatch(&i), Arm::Chase { avoid: Avoid::None, .. }));
    let mut i = disp();
    i.pattern = PATTERN_HOLD;
    assert_eq!(dispatch(&i), Arm::Chase { speed: 0.0, aim: [1.0, 0.0, 2.0], avoid: Avoid::None });
    i.state = 2;
    assert_eq!(dispatch(&i), Arm::Idle);
    let mut i = disp();
    i.pattern = PATTERN_ROAD;
    assert_eq!(dispatch(&i), Arm::Chase { speed: 30.0, aim: [4.0, 0.0, 4.0], avoid: Avoid::Road });
    i.state = 4;
    i.alive = 0.6;
    assert_eq!(dispatch(&i), Arm::Traffic);
    i.alive = 0.4;
    assert_eq!(dispatch(&i), Arm::Gated);
    let mut i = disp();
    i.pattern = PATTERN_ROAD_FOLLOW;
    i.nav_repeat = true;
    assert_eq!(dispatch(&i), Arm::Chase { speed: 30.0, aim: [4.0, 0.0, 4.0], avoid: Avoid::None });
    i.pattern = PATTERN_PATH;
    assert_eq!(dispatch(&i), Arm::Path);
    i.alive = 0.0;
    assert_eq!(dispatch(&i), Arm::Gated);
    let mut i = disp();
    i.stop_byte = true;
    assert_eq!(dispatch(&i), Arm::Gated);
}

// ---------------------------------------------------------------- the chase drive (0x133f60)

fn cdi() -> ChaseDriveInputs {
    ChaseDriveInputs {
        profile: 0xd,
        pattern: PATTERN_PURSUE,
        spawn_delay: 0,
        hold_160: false,
        hold_15c: false,
        nav_repeat: false,
        has_target: true,
        target_is_player: false,
        target_pos: [0.0, 0.0, 80.0],
        target_vel: [0.0, 0.0, 10.0],
        target_fwd: [0.0, 0.0, 1.0],
        dir: [0.0, 0.0, 1.0],
        held_steer: 0.0,
        fps: 60,
    }
}

fn no_bump(_: bool) -> bool {
    false
}

#[test]
fn chase_pedals_and_ramp() {
    let b = body([0.0; 3], [0.0, 0.0, 10.0]);
    let aim = [5.0, 0.0, 50.0];
    let mut st = ReverseState::default();
    // ramp = clamp(min(|30 − 10|, 15)/15, 0.05, 1) = 1.
    let c = chase_drive(&b, aim, 30.0, &mut st, &cdi(), &mut no_bump);
    assert_eq!((c.throttle, c.brake, c.handbrake), (1.0, 0.0, false));
    let side = 5.0 / (25.0f32 + 2500.0).sqrt();
    assert!(near(c.steer.unwrap(), side), "steer = dot(d, row0)");
    // target 12 at 10: 2/15.
    let c = chase_drive(&b, aim, 12.0, &mut st, &cdi(), &mut no_bump);
    assert!(near(c.throttle, 2.0 / 15.0));
    // A near match (2 ≤ 2.5, target ≥ 5) holds.
    let b2 = body([0.0; 3], [0.0, 0.0, 12.0]);
    let c = chase_drive(&b2, aim, 10.0, &mut st, &cdi(), &mut no_bump);
    assert_eq!((c.throttle, c.brake), (0.0, 0.0));
    // Over speed by more: brake(ramp) = 6/15.
    let b3 = body([0.0; 3], [0.0, 0.0, 16.0]);
    let c = chase_drive(&b3, aim, 10.0, &mut st, &cdi(), &mut no_bump);
    assert!(near(c.brake, 6.0 / 15.0) && c.throttle == 0.0);
    // The floor 0.05.
    let b4 = body([0.0; 3], [0.0, 0.0, 9.99]);
    assert!(near(chase_drive(&b4, aim, 10.0, &mut st, &cdi(), &mut no_bump).throttle, 0.05));
    // Below speed 1 forward the steer is 0.
    let b5 = body([0.0; 3], [0.0, 0.0, 0.5]);
    assert_eq!(chase_drive(&b5, aim, 30.0, &mut st, &cdi(), &mut no_bump).steer, Some(0.0));
}

#[test]
fn chase_stop_rules() {
    let aim = [0.0, 0.0, 50.0];
    let mut st = ReverseState::default();
    let c = chase_drive(&body([0.0; 3], [0.0, 0.0, 0.5]), aim, 0.0, &mut st, &cdi(), &mut no_bump);
    assert_eq!((c.steer, c.handbrake, c.throttle, c.brake), (Some(0.0), true, 0.0, 0.0));
    let c = chase_drive(&body([0.0; 3], [0.0, 0.0, 0.05]), aim, 0.0, &mut st, &cdi(), &mut no_bump);
    assert!(!c.handbrake, "below 0.1 no handbrake");
    let c = chase_drive(&body([0.0; 3], [0.0, 0.0, 5.0]), aim, 0.0, &mut st, &cdi(), &mut no_bump);
    assert_eq!((c.brake, c.throttle), (1.0, 0.0));
    let c = chase_drive(&body([0.0; 3], [0.0, 0.0, -5.0]), aim, 0.0, &mut st, &cdi(), &mut no_bump);
    assert_eq!((c.brake, c.throttle), (0.0, 1.0), "rolling backwards: throttle to stop");
}

#[test]
fn the_reverse_hysteresis_and_the_bump() {
    // Aim behind (ahead = −1 < −0.15) → reversing, full lock opposite the side.
    let b = body([0.0; 3], [0.0, 0.0, 0.0]);
    let mut st = ReverseState::default();
    let c = chase_drive(&b, [1.0, 0.0, -20.0], 20.0, &mut st, &cdi(), &mut no_bump);
    assert!(st.reversing);
    assert_eq!(c.steer, Some(-1.0), "side > 0 → −1");
    assert_eq!(c.brake, 0.5, "reverse at ≤ 7.5");
    let c = chase_drive(&body([0.0; 3], [0.0, 0.0, -8.0]), [1.0, 0.0, -20.0], 20.0, &mut st, &cdi(), &mut no_bump);
    assert_eq!((c.brake, c.throttle), (0.0, 0.0), "reversing past 7.5: no pedal");
    let c = chase_drive(&body([0.0; 3], [0.0, 0.0, 3.0]), [1.0, 0.0, -20.0], 20.0, &mut st, &cdi(), &mut no_bump);
    assert_eq!(c.brake, 1.0, "reversing while still rolling forward: brake 1");
    // Off once the aim is ahead (≥ −0.15) with forward motion.
    chase_drive(&body([0.0; 3], [0.0, 0.0, 3.0]), [0.0, 0.0, 20.0], 20.0, &mut st, &cdi(), &mut no_bump);
    assert!(!st.reversing);
    // A bump flips the direction for fps ticks.
    let mut st = ReverseState::default();
    let mut hit = |rev: bool| {
        assert!(!rev);
        true
    };
    let c = chase_drive(&body([0.0; 3], [0.0, 0.0, 2.0]), [0.0, 0.0, 20.0], 20.0, &mut st, &cdi(), &mut hit);
    assert_eq!((st.timer, st.timed), (60, true));
    assert_eq!(c.brake, 1.0, "rolling forward while the bump reverses");
    let c = chase_drive(&body([0.0; 3], [0.0, 0.0, 0.0]), [0.0, 0.0, 20.0], 20.0, &mut st, &cdi(), &mut no_bump);
    assert_eq!((st.timer, c.brake), (59, 0.5));
    // In its second half (≤ fps/2) an aim well ahead (> 0.7) ends it.
    st.timer = 30;
    chase_drive(&body([0.0; 3], [0.0, 0.0, 0.0]), [0.0, 0.0, 20.0], 20.0, &mut st, &cdi(), &mut no_bump);
    assert_eq!(st.timer, 0);
    st.timer = 31;
    chase_drive(&body([0.0; 3], [0.0, 0.0, 0.0]), [0.0, 0.0, 20.0], 20.0, &mut st, &cdi(), &mut no_bump);
    assert_eq!(st.timer, 30);
}

#[test]
fn head_on_with_the_player_skips_the_probe() {
    let mut i = cdi();
    i.target_is_player = true;
    i.target_pos = [0.0, 0.0, 9.0];
    i.target_vel = [0.0, 0.0, -10.0];
    let mut st = ReverseState::default();
    let mut probed = false;
    let mut p = |_: bool| {
        probed = true;
        true
    };
    let c = chase_drive(&body([0.0; 3], [0.0, 0.0, 0.5]), [0.0, 0.0, 9.0], 0.0, &mut st, &i, &mut p);
    assert!(!probed);
    assert!(!c.handbrake, "the head-on rule cancels the stop handbrake");
    // Outside 10 the probe runs.
    i.target_pos = [0.0, 0.0, 10.5];
    let mut st = ReverseState::default();
    let mut probed = false;
    let mut p = |_: bool| {
        probed = true;
        false
    };
    chase_drive(&body([0.0; 3], [0.0, 0.0, 0.5]), [0.0, 0.0, 9.0], 0.0, &mut st, &i, &mut p);
    assert!(probed);
}

#[test]
fn spawn_delay_and_steer_holds() {
    let mut i = cdi();
    i.spawn_delay = 5;
    i.held_steer = 0.3;
    let mut st = ReverseState::default();
    let b = body([0.0; 3], [0.0, 0.0, 10.0]);
    assert!(near(chase_drive(&b, [9.0, 0.0, 9.0], 20.0, &mut st, &i, &mut no_bump).steer.unwrap(), 0.6));
    i.held_steer = 0.0;
    assert!(near(chase_drive(&b, [9.0, 0.0, 9.0], 20.0, &mut st, &i, &mut no_bump).steer.unwrap(), 0.2));
    i.held_steer = 0.7;
    assert_eq!(chase_drive(&b, [9.0, 0.0, 9.0], 20.0, &mut st, &i, &mut no_bump).steer, Some(1.0));
    i.pattern = PATTERN_ROAD_FOLLOW;
    i.hold_15c = true;
    assert_eq!(chase_drive(&b, [9.0, 0.0, 9.0], 20.0, &mut st, &i, &mut no_bump).steer, None);
}

#[test]
fn the_bump_probe_rays() {
    let b = body([0.0; 3], [0.0; 3]);
    let mut rays = Vec::new();
    let mut contact = true;
    assert!(!bump_probe(&mut contact, 5, false, &b, &mut |f, t| {
        rays.push((f, t));
        9
    }));
    assert!(contact && rays.is_empty(), "kind 5: no probe, contact kept");
    assert!(bump_probe(&mut contact, 6, false, &b, &mut |f, t| {
        rays.push((f, t));
        2
    }));
    assert!(!contact);
    assert_eq!(rays, vec![([0.0; 3], [0.0, 0.0, 4.0])], "forward 2·box_z");
    let mut rays = Vec::new();
    let mut contact = true;
    assert!(!bump_probe(&mut contact, 6, true, &b, &mut |f, t| {
        rays.push((f, t));
        1
    }));
    // back = (0,0,−1); corner side (back.z, back.y, −back.x) = (−1, 0, 0)·box_x, then the mirror.
    assert_eq!(rays, vec![([0.0; 3], [-1.0, 0.0, -4.0]), ([0.0; 3], [1.0, 0.0, -4.0])]);
}

// ---------------------------------------------------------------- the flying arm

#[test]
fn the_altitude_hold() {
    let mut fs = FlyState { hold: true, ceiling: 40.0, floor: 10.0, phase: 0.0, bob: 0.0 };
    let mut aim = [0.0, 0.0, 0.0];
    fly_aim(&mut fs, &mut aim, 25.0, DT);
    assert_eq!(aim[1], 25.0);
    fly_aim(&mut fs, &mut aim, 5.0, DT);
    assert_eq!(aim[1], 10.0, "the floor");
    fly_aim(&mut fs, &mut aim, 50.0, DT);
    assert_eq!(aim[1], 40.0, "the ceiling");
    fs.bob = 10.0;
    fs.phase = 0.25 - DT * 0.05;
    fly_aim(&mut fs, &mut aim, 20.0, DT);
    assert!(near(fs.phase, 0.25) && (aim[1] - 30.0).abs() < 2e-3, "{:?}", aim);
    fs.phase = 0.99999;
    fly_aim(&mut fs, &mut aim, 20.0, DT);
    assert!(fs.phase < 0.01, "the phase wraps at 1");
    // Without the hold nothing moves.
    let mut fs = FlyState::default();
    let mut aim = [1.0, 2.0, 3.0];
    fly_aim(&mut fs, &mut aim, 99.0, DT);
    assert_eq!(aim, [1.0, 2.0, 3.0]);
}

#[test]
fn the_flying_drive() {
    let b = body([0.0; 3], [0.0, 0.0, 10.0]);
    let mut st = ReverseState::default();
    // Aim up and to the side: side = up × fwd = (1, 0, 0); pitch call = −dot(d, up').
    let aim = [3.0, 4.0, 12.0];
    let c = fly_drive(&b, aim, 20.0, false, &mut st, false, 60, &mut no_bump);
    let n = (9.0f32 + 16.0 + 144.0).sqrt();
    assert!(near(c.steer.unwrap(), 3.0 / n) && near(c.pitch, -4.0 / n));
    assert!(near(c.throttle, 1.0), "ramp over 5: |20 − 10| → 1");
    // Off the nose (ahead < 0.7) the target becomes 0.75·s = 7.5 → brake(2.5/5).
    let c = fly_drive(&b, [10.0, 0.0, 1.0], 20.0, false, &mut st, false, 60, &mut no_bump);
    assert!(near(c.brake, 0.5) && c.throttle == 0.0, "{c:?}");
    // The hold drops a pitch that would steepen a nose past 0.4.
    let mut nose = body([0.0; 3], [0.0, 0.0, 10.0]);
    let r = 0.6f32;
    nose.fwd = [0.0, r, (1.0 - r * r).sqrt()];
    let c = fly_drive(&nose, [0.0, 50.0, 10.0], 20.0, true, &mut st, false, 60, &mut no_bump);
    assert_eq!(c.pitch, -0.0);
    let c = fly_drive(&nose, [0.0, 50.0, 10.0], 20.0, false, &mut st, false, 60, &mut no_bump);
    assert!(c.pitch < 0.0);
}

// ---------------------------------------------------------------- avoidance (0x137310)

#[test]
fn the_slot_build() {
    let mut sl = Slots::default();
    slots_init(&mut sl, (None, None));
    assert_eq!(sl.cost[0], 160);
    assert_eq!(sl.cost[10], 110);
    assert_eq!(sl.cost[63], 155);
    assert_eq!((sl.cost[0x1f], sl.cost[0x20], sl.cost[0x21]), (0, 0, 0));
    assert_eq!(sl.cost[0x1e], 10);
    // A wall 5 units out on the + side fills every slot above 37; on the − side 3 out, below 29.
    slots_init(&mut sl, (Some(5.0), Some(3.0)));
    assert_eq!((sl.cost[37], sl.cost[38], sl.cost[63]), (25, 255, 255));
    assert_eq!((sl.cost[28], sl.cost[29]), (255, 15));
}

#[test]
fn the_course_test_and_the_slot_pick() {
    let me = [0.0; 3];
    let dir = [0.0, 0.0, 1.0];
    assert!(on_course(5.0, 0.0, me, [0.0, 0.0, 20.0], dir, false), "r = max(10, 20)");
    assert!(!on_course(5.0, 0.0, me, [0.0, 0.0, 20.1], dir, false));
    assert!(on_course(30.0, 0.0, me, [0.0, 0.0, 35.0], dir, false), "r = min(60, 35)");
    assert!(!on_course(30.0, 0.0, me, [0.0, 0.0, 35.1], dir, false));
    assert!(!on_course(30.0, 31.0, me, [0.0, 0.0, 16.0], dir, false), "a faster car: 15");
    assert!(!on_course(5.0, 0.0, me, [0.0, 0.0, -5.0], dir, false), "behind");
    assert!(!on_course(5.0, 0.0, me, [0.0, 5.1, 5.0], dir, false), "height band");
    assert!(on_course(5.0, 0.0, me, [0.0, 5.1, 5.0], dir, true));
    let mut sl = Slots::default();
    slots_init(&mut sl, (None, None));
    assert_eq!(best_slot(&sl), 0x20, "the clear centre");
    sl.width = 4;
    assert_eq!(best_slot(&sl), 0x1f + 1, "4 wide: start 30 (10+0+0+0) → 30 + 2");
}

#[test]
fn avoiding_a_car_ahead() {
    let me = body([0.0; 3], [0.0, 0.0, 10.0]);
    let car = OtherCar { id: 7, pos: [0.0, 0.0, 15.0], sim_6d: 0, sim_6e: 0, side: [1.0, 0.0, 0.0], box_z: 2.0, width: 3, is_target: false };
    let mut sl = Slots::default();
    let mut aim = [0.0, 0.0, 10.0];
    avoid_chase(&mut sl, &me, 1, [0.0, 0.0, 1.0], false, (None, None), &[car], &mut aim);
    // Slots 31..33 get +0x80; the best 3-window starts at 28 (20 + 15 + 10) → centre 29 → −3.
    assert_eq!((sl.cost[31], sl.cost[32], sl.cost[33]), (128, 128, 128));
    assert!(near3(aim, [-3.0, 0.0, 10.0]), "{aim:?}");
    // The target and traffic are not avoided.
    for c in [OtherCar { is_target: true, ..car }, OtherCar { sim_6e: 3, ..car }, OtherCar { id: 1, ..car }] {
        let mut aim = [0.0, 0.0, 10.0];
        avoid_chase(&mut sl, &me, 1, [0.0, 0.0, 1.0], false, (None, None), &[c], &mut aim);
        assert!(near3(aim, [0.0, 0.0, 10.0]));
    }
    // An even own width shifts half a slot.
    let mut sl = Slots { width: 2, ..Slots::default() };
    let mut aim = [0.0, 0.0, 10.0];
    avoid_chase(&mut sl, &me, 1, [0.0, 0.0, 1.0], false, (None, None), &[], &mut aim);
    // Width 2: first minimum is start 31 (0 + 0) → centre 32 → 0 − 0.5.
    assert!(near3(aim, [-0.5, 0.0, 10.0]), "{aim:?}");
}

// ---------------------------------------------------------------- the turret (0x12f170)

#[test]
fn the_turret_slews_the_short_way() {
    let mut t = Turret::default();
    // A target ahead-right (dir (0.98, 0, 0.196), fwd (0,0,1)): a = 1 − acos(0.196)/2π ≈ 0.781
    // ≥ 0.75 with the turret at 0 ≤ 0.25 → it backs through the wrap.
    let deg = turret_step(&mut t, false, 0, [0.980_580_7, 0.0, 0.196_116_1], [0.0, 0.0, 1.0], None).unwrap();
    assert!(near(t.angle, 0.995) && !t.on_target);
    assert_eq!(deg, 358);
    // On the left: a = 0.25, +0.005.
    let mut t = Turret::default();
    turret_step(&mut t, false, 0, [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0], None);
    assert!(near(t.angle, 0.005));
    // Within 0.005 it snaps.
    let mut t = Turret { angle: 0.247, on_target: false };
    turret_step(&mut t, false, 0, [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0], None);
    assert!(near(t.angle, 0.25) && t.on_target);
    // Lost: no update (unless target mode 1).
    let mut t = Turret::default();
    assert!(turret_step(&mut t, true, 0, [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0], None).is_none());
    assert!(turret_step(&mut t, true, 1, [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0], None).is_some());
    // But a target at 0.7 from 0 goes up (+), the long way: the wrap rule only covers a ≥ 0.75.
    let mut t = Turret::default();
    let a = 0.7f32 * std::f32::consts::TAU;
    turret_step(&mut t, false, 0, [-a.sin(), 0.0, a.cos()], [0.0, 0.0, 1.0], None);
    assert!(near(t.angle, 0.005), "{}", t.angle);
    // TURRET_RANGE: 90° is outside [0, 60] ∪ [300, 360] → hold.
    let mut t = Turret::default();
    turret_step(&mut t, false, 0, [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0], Some([0.0, 60.0, 300.0, 360.0]));
    assert_eq!(t.angle, 0.0);
    turret_step(&mut t, false, 0, [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0], Some([0.0, 91.0, 300.0, 360.0]));
    assert!(near(t.angle, 0.005));
}

// ---------------------------------------------------------------- the world cap (0x2012d0)

#[test]
fn the_world_vehicle_cap() {
    let cars: Vec<([f32; 3], i8)> = vec![([10.0, 0.0, 0.0], 0), ([500.0, 0.0, 0.0], 2), ([90.0, 0.0, 0.0], 3), ([20.0, 0.0, 0.0], 0)];
    assert_eq!(world_cap(35, &cars, [0.0; 3], false), (None, true));
    assert_eq!(world_cap(36, &cars, [0.0; 3], false), (None, false), "at the cap: refused, nothing removed");
    assert_eq!(world_cap(37, &cars, [0.0; 3], false), (Some(2), false), "farthest non-kind-2 removed; 36 is not < 36");
    assert_eq!(world_cap(47, &cars, [0.0; 3], true), (None, true));
    assert_eq!(world_cap(49, &cars, [0.0; 3], true), (Some(2), false));
}

// ---------------------------------------------------------------- helicopter leaf rules

#[test]
fn helicopter_leaf_rules() {
    assert_eq!(heli_advance(10.0), 10.0);
    assert_eq!(heli_advance(30.0), 15.0);
    assert_eq!(heli_advance(50.0), 20.0);
    assert_eq!(heli_update_gate((true, true), 5, false, false), (true, true));
    assert_eq!(heli_update_gate((true, true), 9, false, false), (false, true));
    assert_eq!(heli_update_gate((true, false), 5, false, false), (false, false));
    assert_eq!(heli_update_gate((true, true), 5, true, true), (false, false));
    assert_eq!(approach(0.0, 1.0, 0.02), 0.02);
    assert_eq!(approach(0.5, 0.51, 0.02), 0.51);
    assert_eq!(approach(0.5, 0.0, 0.02), 0.48);
    // Lift: low (height 1 ≤ 2) → no random factor; 2·(min(30 − 1, 10) − 1) + 9.8 = 27.8.
    let mut rng = DriveRng::game_reset();
    let d0 = rng.draws;
    let l = heli_lift(1.0, 1.0, 0.0, 0.0, 30.0, 20.0, 5.0, 0.5, 0.1, &mut rng);
    assert_eq!(rng.draws, d0 + 1, "one draw per call");
    assert!(near(l.lift, 27.8) && l.goal_y == 30.0 && l.roll == 0.1);
    // High: the factor clamp(r·4/65536, 0.5, 1) and the roll step toward 0.5·20/50 = 0.2.
    let mut a = DriveRng::game_reset();
    let mut b = a.clone();
    let r = b.next() as i32 as f32 * 1.525_878_9e-5 * 4.0;
    let k = r.max(0.5).min(1.0);
    let l = heli_lift(10.0, 0.0, 0.0, 12.0, 0.0, 20.0, 5.0, 0.5, 0.1, &mut a);
    assert!(near(l.lift, (2.0 * 2.0 + 9.8) * k));
    assert!(near(l.roll, 0.12));
    let l = heli_lift(10.0, 0.0, 0.0, 12.0, 0.0, 20.0, 25.0, 0.5, 0.1, &mut a);
    assert_eq!(l.roll, 0.0, "at max speed the roll is 0");
}

#[test]
fn the_gunner_sweep() {
    let mut yaw = 0.0;
    assert_eq!(gunner_sweep(&mut yaw, false, 0, 0), Some(-53.0));
    assert_eq!(yaw, 2.0);
    let mut yaw = 79.0;
    assert_eq!(gunner_sweep(&mut yaw, false, 1, 1), Some(26.0));
    assert_eq!(yaw, 80.0, "clamped to 80 after the bone angle");
    let mut yaw = 90.0;
    assert_eq!(gunner_sweep(&mut yaw, false, 1, 0), Some(30.0));
    let mut yaw = -59.0;
    assert_eq!(gunner_sweep(&mut yaw, false, 0, 1), Some(-80.0));
    assert_eq!(yaw, -60.0);
    let mut yaw = 5.0;
    assert_eq!(gunner_sweep(&mut yaw, true, 0, 0), None);
    assert_eq!(yaw, 5.0);
}

// ---------------------------------------------------------------- the disc (NF_ISO)

fn body_of(car: &Car, p: &CarParams) -> Body {
    let (s, c) = car.yaw.sin_cos();
    let _ = p;
    Body { pos: car.pos, vel: car.vel, side: [c, 0.0, -s], up: [0.0, 1.0, 0.0], fwd: [s, 0.0, c], box_z: 2.0, box_x: 1.0 }
}

/// The scripted player: from `start` along `heading` at 15 u/s with a slow weave [port script].
fn player_at(start: [f32; 3], heading: [f32; 3], tick: i32) -> ([f32; 3], [f32; 3]) {
    let t = tick as f32 * DT;
    let side = [heading[2], 0.0, -heading[0]];
    let w = (t * 0.5).sin() * 8.0;
    let pos = [start[0] + heading[0] * 15.0 * t + side[0] * w, start[1], start[2] + heading[2] * 15.0 * t + side[2] * w];
    let wv = (t * 0.5).cos() * 4.0;
    (pos, [heading[0] * 15.0 + side[0] * wv, 0.0, heading[2] * 15.0 + side[2] * wv])
}

struct Run {
    start_gap: f32,
    end_gap: f32,
    /// The gap a car parked at the start would end with.
    parked_gap: f32,
    travelled: f32,
    max_cmd: f32,
}

/// One hostile of `cmd`'s pattern on its mission for 600 ticks [port harness; S where noted].
fn run_pattern(net: &RoadNet, params: &CarParams, start: [f32; 3], facing: [f32; 3], cmd: &AiCommand, rng: &mut DriveRng) -> Run {
    let fwd = {
        let l = (facing[0] * facing[0] + facing[2] * facing[2]).sqrt().max(1e-6);
        [facing[0] / l, 0.0, facing[2] / l]
    };
    // The port's flat world has its ground at y = 0: the run is laid on it, keeping x/z [S].
    let start = [start[0], 0.0, start[2]];
    let mut car = Car::new(params.clone(), start, fwd[0].atan2(fwd[2]));
    let world = FlatWorld::paved();
    let rigid = Rigid::game_defaults();
    let pstart = [start[0] + fwd[0] * 40.0, start[1], start[2] + fwd[2] * 40.0];
    let pattern = cmd.pattern as u32;
    // Road patterns ride a navigator joined on the nearest piece (a brute-force search: the
    // game's FUN_00225490 grid query is not ported) [S]; the hinted kind-3/4 chooser is not
    // ported either, the traffic chooser stands in [S].
    let mut nav: Option<RoadNav> = None;
    if pattern == PATTERN_ROAD || pattern == PATTERN_ROAD_FOLLOW {
        let mut best = (f32::MAX, 0usize);
        for k in 0..net.segs.len() {
            for t in [0.0, 0.5, 1.0] {
                if let Ok(p) = net.seg_point(k, t) {
                    let d = (p[0] - start[0]).powi(2) + (p[2] - start[2]).powi(2);
                    if d < best.0 {
                        best = (d, k);
                    }
                }
            }
        }
        let dir = net.seg(best.1).unwrap().dir();
        let forward = dir[0] * fwd[0] + dir[2] * fwd[2] >= 0.0;
        nav = RoadNav::join(net, best.1, 0.5, 0, forward, NAV_KIND_TRAFFIC, LaneSet::Default).ok();
    }
    let mut st = ReverseState::default();
    let mut sl = Slots::default();
    let mut prev_aim = start;
    let mut prev_dist = 0.0;
    let mut max_cmd = 0.0f32;
    let (p0, _) = player_at(pstart, fwd, 0);
    let start_gap = ((p0[0] - start[0]).powi(2) + (p0[2] - start[2]).powi(2)).sqrt();
    for tick in 0..600 {
        let (ppos, pvel) = player_at(pstart, fwd, tick);
        let b = body_of(&car, params);
        let carrot = nav.as_ref().map(|n| n.pos).unwrap_or(b.pos);
        let aim = aim_point(&AimInputs {
            profile: cmd.profile as u32,
            pattern,
            target_mode: cmd.target_mode as u32,
            goal: cmd.goal,
            has_target: true,
            target_pos: ppos,
            target_vel: pvel,
            lost: false,
            self_body: b,
            lead: cmd.lead,
            follow: cmd.follow,
            prev_dist,
            prev_aim,
            nav_carrot: carrot,
            nav_10: carrot,
        });
        prev_aim = aim.point;
        prev_dist = aim.dist;
        let seg_limit = nav.as_ref().and_then(|n| net.seg(n.seg).ok()).map(|s| s.speed_limit()).unwrap_or(0.0);
        let speed = chase_target_speed(&ChaseSpeedInputs {
            state: 5,
            profile: cmd.profile as u32,
            pattern,
            alive: 1.0,
            stop_query: false,
            stop_byte: false,
            self_pos: b.pos,
            self_vel: b.vel,
            self_forward: b.fwd,
            self_box: b.box_z,
            has_target: true,
            target_is_player: true,
            target_vel: pvel,
            target_box: 2.0,
            aim: aim.point,
            dist: aim.dist,
            dir: aim.dir,
            dir_210: aim.nav_dir,
            follow: cmd.follow,
            catchup: cmd.catchup,
            cap: cmd.speed_cap,
            speed_mul: 1.0,
            ramming: false,
            lost: false,
            element_target: false,
            follow_target: cmd.follow_target != 0,
            spawn_delay: 0,
            seg_limit,
            speed_factor: 1.0,
        })
        .speed;
        // The carrot for road patterns (FUN_00130140's plain path) [M 0x130140].
        if let Some(n) = nav.as_mut() {
            let la = lookahead(b.speed(), b.box_z);
            let gap = ((b.pos[0] - n.pos[0]).powi(2) + (b.pos[1] - n.pos[1]).powi(2) + (b.pos[2] - n.pos[2]).powi(2)).sqrt();
            if gap < la {
                let _ = n.advance(net, la - gap, &mut || rng.next());
            }
        }
        let carrot = nav.as_ref().map(|n| n.pos).unwrap_or(b.pos);
        let arm = dispatch(&DispatchInputs {
            state: 5,
            pattern,
            alive: 1.0,
            stop_query: false,
            stop_byte: false,
            target_speed: speed,
            avoid_on: cmd.no_avoid == 0,
            nav_repeat: nav.as_ref().map(|n| n.repeated).unwrap_or(false),
            self_pos: b.pos,
            dir: aim.dir,
            lookahead: lookahead(b.speed(), b.box_z),
            carrot,
        });
        let cmdv = match arm {
            Arm::Chase { speed, mut aim, avoid } => {
                if avoid == Avoid::Chase {
                    avoid_chase(&mut sl, &b, 0, aim_dir(&b, aim), false, (None, None), &[], &mut aim);
                }
                let i = ChaseDriveInputs {
                    profile: cmd.profile as u32,
                    pattern,
                    spawn_delay: 0,
                    hold_160: false,
                    hold_15c: false,
                    nav_repeat: false,
                    has_target: true,
                    target_is_player: true,
                    target_pos: ppos,
                    target_vel: pvel,
                    target_fwd: fwd,
                    dir: prev_dir(&b, prev_aim),
                    held_steer: 0.0,
                    fps: 60,
                };
                chase_drive(&b, aim, speed, &mut st, &i, &mut |_| false).drive()
            }
            Arm::Gated => gated_stop(&SimCar { pos: b.pos, vel: b.vel, forward: b.fwd, right: b.side, alive: 1.0 }),
            _ => nf_game::driving::ai::DriveCommand::IDLE,
        };
        for v in [cmdv.steer, cmdv.throttle, cmdv.brake] {
            assert!(v.is_finite() && (-1.0..=1.0).contains(&v), "tick {tick}: command {cmdv:?}");
            max_cmd = max_cmd.max(v.abs());
        }
        car.step(&CarInput { throttle: cmdv.throttle, brake: cmdv.brake, steer: cmdv.steer, handbrake: cmdv.handbrake }, &world, &rigid, DT);
        // Same stand-in as `ai::AiDriver::step`: the velocity follows the heading (no tyre side
        // force in `Car::step`) [S].
        let (s, c) = car.yaw.sin_cos();
        let along = car.vel[0] * s + car.vel[2] * c;
        car.vel[0] = s * along;
        car.vel[2] = c * along;
        assert!(car.pos.iter().all(|x| x.is_finite()), "tick {tick}: position");
    }
    let (pend, _) = player_at(pstart, fwd, 599);
    let end_gap = ((pend[0] - car.pos[0]).powi(2) + (pend[2] - car.pos[2]).powi(2)).sqrt();
    let travelled = ((car.pos[0] - start[0]).powi(2) + (car.pos[2] - start[2]).powi(2)).sqrt();
    let parked_gap = ((pend[0] - start[0]).powi(2) + (pend[2] - start[2]).powi(2)).sqrt();
    Run { start_gap, end_gap, parked_gap, travelled, max_cmd }
}

fn aim_dir(b: &Body, aim: [f32; 3]) -> [f32; 3] {
    let d = [aim[0] - b.pos[0], aim[1] - b.pos[1], aim[2] - b.pos[2]];
    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if l > 0.0 { [d[0] / l, d[1] / l, d[2] / l] } else { [0.0; 3] }
}
fn prev_dir(b: &Body, aim: [f32; 3]) -> [f32; 3] {
    aim_dir(b, aim)
}

/// The missions searched, in order, for the first ground vehicle whose spawn command has each
/// pattern [D].
const MISSIONS: [&str; 8] = ["MIS01.VIV", "MIS11.VIV", "MIS13A.VIV", "MIS13B.VIV", "MIS13C.VIV", "MIS3.VIV", "MIS4.VIV", "RACE.VIV"];

#[test]
fn each_pattern_on_its_mission() {
    let Some(path) = std::env::var_os("NF_ISO") else {
        eprintln!("NF_ISO not set — skipping");
        return;
    };
    let iso = IsoImage::open(&path).unwrap();
    let mut done = Vec::new();
    let mut helis = 0;
    for mission in MISSIONS {
        let e = iso.entries().iter().find(|e| e.path == format!("/DRIVING/{mission}")).cloned().unwrap();
        let bigf = nf_formats::disc::read_bigf(&iso, &e).unwrap();
        let base = e.lba as u64 * 2048;
        let read = |want: &dyn Fn(&str) -> bool| -> Vec<(String, Vec<u8>)> {
            bigf.entries
                .iter()
                .filter(|x| want(&x.name.to_ascii_lowercase()))
                .map(|x| {
                    let raw = iso.read_bytes(base + x.offset as u64, x.size as u64).unwrap();
                    let img = if refpack::is_refpack(&raw) { refpack::decompress(&raw).unwrap() } else { raw };
                    (x.name.to_ascii_lowercase(), img)
                })
                .collect()
        };
        let track = read(&|n| n.contains("track") && n.ends_with(".crp"));
        let crp = Crp::parse(&track[0].1).unwrap();
        let ai = MapAi::from_crp(&crp).unwrap();
        let net = RoadNet::parse_crp(&crp).unwrap();
        let atrs = read(&|n| n.contains("\\pvehicle\\") && n.ends_with(".atr"));
        // The element's spawn command: +0xf8 is an `sr` reference 0x7372_nnnn; nnnn − 1 is
        // read as the AICo index [D: every reference on the disc has the 0x7372 tag and an
        // index ≤ the AICo count; the index base is S].
        for el in &ai.elements {
            assert!(el.command_ref == 0 || (el.command_ref >> 16 == 0x7372 && (el.command_ref & 0xffff) as usize <= ai.commands.len()), "{mission}: ref {:#x}", el.command_ref);
        }
        if let Some(h) = ai.elements.iter().find(|e| e.kind == TYPE_VEHICLE && e.heli) {
            // The helicopter's lift rule (FUN_0013b4f8) for 600 ticks from its spawn height
            // toward 35 above the ground at y = 0, reading +0xa0 as the vertical acceleration
            // against 9.8 gravity [S: what consumes +0xa0 was not read].
            let mut rng = DriveRng::game_reset();
            let (mut y, mut vy, mut roll) = (h.pos[1].max(0.0), 0.0f32, 0.0f32);
            for _ in 0..600 {
                let l = heli_lift(y, vy, 0.0, 0.0, 35.0, 20.0, 0.0, 0.5, roll, &mut rng);
                roll = l.roll;
                vy += (l.lift - 9.8) * DT;
                y += vy * DT;
                assert!(y.is_finite() && l.lift.is_finite());
            }
            eprintln!("{mission} helicopter {}: spawn y {:.1} → {y:.2} (goal 35), roll {roll:.3}, rng draws {}", h.name, h.pos[1], rng.draws);
            assert_eq!(rng.draws, 600, "one draw per tick");
            assert!((y - 35.0).abs() < 2.0 && (roll - 0.2).abs() < 1e-4, "{mission}: the lift settles at the goal height");
            helis += 1;
        }
        for pattern in 1u8..=5 {
        if done.contains(&pattern) {
            continue;
        }
        let found = ai.elements.iter().find_map(|el| {
            if el.kind != TYPE_VEHICLE || el.heli || el.command_ref == 0 {
                return None;
            }
            let c = ai.commands.get((el.command_ref & 0xffff) as usize - 1)?;
            (c.pattern == pattern).then(|| (el.clone(), c.clone()))
        });
        let Some((el, cmd)) = found else { continue };
        let file = format!("\\pvehicle\\{}.atr", el.name.to_ascii_lowercase());
        let params = atrs.iter().find(|(n, _)| n.ends_with(&file)).map(|(_, t)| CarParams::parse_atr(&String::from_utf8_lossy(t))).unwrap_or_else(CarParams::game_defaults);
        let mut rng = DriveRng::game_reset();
        let r = run_pattern(&net, &params, el.pos, el.facing, &cmd, &mut rng);
        eprintln!(
            "{mission} pattern {pattern} ({}, profile {:#x}, follow {}, cap {}): gap {:.1} → {:.1} (parked {:.1}), travelled {:.1}, max pedal {:.2}, rng draws {}",
            el.name, cmd.profile, cmd.follow, cmd.speed_cap, r.start_gap, r.end_gap, r.parked_gap, r.travelled, r.max_cmd, rng.draws
        );
        match pattern {
            // Hold: aim at itself, speed 0 → it stays put and issues no pedal.
            1 => assert!(r.travelled < 1.0 && r.max_cmd == 0.0, "{mission}: a holding car moved {} / pedal {}", r.travelled, r.max_cmd),
            // Pursue: it gets under way and ends nearer the player than a parked car would.
            2 => assert!(r.travelled > 10.0 && r.end_gap < r.parked_gap, "{mission}: pursuit {:?}", (r.travelled, r.end_gap, r.parked_gap)),
            // Road patterns drive somewhere.
            3 | 4 => assert!(r.travelled > 10.0, "{mission}: road car stood still"),
            // Path: the scripted path object is the actor layer; the dispatch drives nothing.
            5 => assert!(r.travelled < 1.0 && r.max_cmd == 0.0, "{mission}: pattern 5 issued pedals"),
            _ => unreachable!(),
        }
        done.push(pattern);
        }
    }
    done.sort();
    assert_eq!(done, vec![1, 2, 3, 4, 5], "a hostile of every pattern ran");
    assert!(helis >= 3, "the helicopter missions ({helis})");
}
