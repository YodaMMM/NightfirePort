//! The AI helicopter controller and the rider gunners checked against the DRIVING.ELF
//! decompile (docs/systems/driving-ai.md §12): the command apply FUN_00138fa8, the state
//! machine FUN_00139be8, the movement FUN_00139380, the flight control FUN_0013b4f8 (yaw, roll,
//! lift), the gun FUN_001387a0, the rockets FUN_00138b88, the death check FUN_0013b2a8; the
//! rider seat table and the sight / aim / cadence functions. Every expected number is the
//! game's constant or arithmetic worked by hand. The last tests read the disc (NF_ISO): the
//! spawn-command references, where the Paris helicopter comes from, and each helicopter's state
//! machine for 1200 ticks against a scripted player.

use nf_formats::crp::Crp;
use nf_formats::driveai::{AiCommand, MapAi, AICO_RECORD, TYPE_VEHICLE};
use nf_formats::iso9660::IsoImage;
use nf_formats::refpack;

use nf_game::driving::ai::{DriveRng, RoadNav, NAV_KIND_TRAFFIC};
use nf_game::driving::heli::*;
use nf_game::driving::mission::MissionSet;
use nf_game::driving::patterns::Body;
use nf_game::driving::riders::*;
use nf_game::driving::road::{LaneSet, RoadNet};
use nf_game::driving::DT;

const SCALE: f32 = 1.525_878_9e-5;

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-4
}

fn body(pos: [f32; 3], vel: [f32; 3]) -> Body {
    Body { pos, vel, side: [1.0, 0.0, 0.0], up: [0.0, 1.0, 0.0], fwd: [0.0, 0.0, 1.0], box_z: 2.0, box_x: 1.0 }
}

fn cmd() -> AiCommand {
    AiCommand::parse(&[0u8; AICO_RECORD]).unwrap()
}

/// A scripted world: flat ground, a fixed target, a fake navigator that records its calls.
struct Env {
    ground: f32,
    target: Body,
    carrot: [f32; 3],
    dir: [f32; 3],
    road: u16,
    target_road: u16,
    differs: bool,
    probe: [f32; 3],
    hit: bool,
    busy: [bool; 2],
    path: Option<PathPose>,
    calls: Vec<String>,
}

impl Env {
    fn new() -> Env {
        Env { ground: 0.0, target: body([0.0, 0.0, 100.0], [0.0; 3]), carrot: [0.0; 3], dir: [0.0, 0.0, 1.0], road: 7, target_road: 7, differs: false, probe: [0.0; 3], hit: false, busy: [false; 2], path: None, calls: vec![] }
    }
}

impl HeliEnv for Env {
    fn ground(&mut self, _p: [f32; 3]) -> f32 {
        self.ground
    }
    fn target(&self) -> Body {
        self.target
    }
    fn path_pose(&mut self) -> Option<PathPose> {
        self.path
    }
    fn nav_carrot(&self) -> [f32; 3] {
        self.carrot
    }
    fn nav_dir(&self) -> [f32; 3] {
        self.dir
    }
    fn nav_road(&self) -> u16 {
        self.road
    }
    fn nav_rejoin(&mut self, _pos: [f32; 3], _dir: [f32; 3]) {
        self.calls.push("rejoin".into());
    }
    fn nav_reset(&mut self) {
        self.calls.push("reset".into());
    }
    fn nav_advance(&mut self, dist: f32, _dir: [f32; 3], road: Option<u16>) {
        self.calls.push(format!("advance {dist:.3} {road:?}"));
    }
    fn target_road(&mut self, _dir: [f32; 3]) -> u16 {
        self.target_road
    }
    fn route_differs(&mut self, _road: u16) -> bool {
        self.differs
    }
    fn probe_carrot(&mut self, _pos: [f32; 3], _fwd: [f32; 3]) -> [f32; 3] {
        self.probe
    }
    fn crash_hit(&mut self, _a: [f32; 3], _b: [f32; 3]) -> bool {
        self.hit
    }
    fn launcher_busy(&self, ch: u8) -> bool {
        self.busy[(ch - 7) as usize]
    }
}

fn commanded(state: u32, mode: u32) -> Heli {
    let mut h = Heli::reset();
    h.attached = true;
    h.commanded = true;
    h.state = state;
    h.mode = mode;
    h.hold_down = 0;
    h
}

// ---------------------------------------------------------------- the command apply

#[test]
fn the_command_apply() {
    let mut a = HeliActor::new(body([1.0, 2.0, 3.0], [0.0; 3]));
    let mut c = cmd();
    c.goal = [10.0, 20.0, 30.0];
    c.follow = -2.0; // |−2| < 5 → 5, and the lead is off
    c.altitude = 1.0; // < 3 → 3
    c.catchup = 1.1;
    c.speed_cap = 35.0;
    c.fire_rate = 5.0;
    c.ram_chance = 0.1; // (char)(int)0.1 = 0
    let mut h = Heli::reset();
    assert_eq!((h.state, h.mode, h.hold_down, h.road), (1, 1, 0xff, 0xffff), "the reset");
    h.apply_command(&c, &mut a, false);
    // +0x7a = 0 → state 10, which with no rockets gets 0xff; the point goes to the post.
    assert_eq!((h.state, h.rockets, h.post, h.goal), (10, 0xff, [10.0, 20.0, 30.0], [0.0; 3]));
    assert_eq!((h.mode, h.radius, h.lead, h.altitude, h.interval, h.hold_down), (2, 5.0, false, 3.0, 5.0, 0));
    assert!(h.commanded);
    // State 3, mode 3, rockets 2.9 → 2, positive radius 15 → lead on, altitude 20.
    let mut c3 = c.clone();
    c3.profile = 3;
    c3.pattern = 3;
    c3.ram_chance = 2.9;
    c3.follow = 15.0;
    c3.altitude = 20.0;
    let mut h3 = Heli::reset();
    h3.apply_command(&c3, &mut a, false);
    assert_eq!((h3.state, h3.mode, h3.rockets, h3.goal, h3.radius, h3.lead, h3.altitude), (3, 3, 2, [10.0, 20.0, 30.0], 15.0, true, 20.0));
    // State 2 parks the body at the goal with the command's rows and forces mode 1 → 2.
    let mut c2 = c.clone();
    c2.profile = 2;
    c2.pattern = 3;
    c2.placement = [[0.0, 0.0, -1.0, 0.0], [0.0, 1.0, 0.0, 0.0], [1.0, 0.0, 0.0, 0.0], [0.0; 4]];
    let mut h2 = Heli::reset();
    h2.apply_command(&c2, &mut a, false);
    assert_eq!((h2.state, h2.mode, a.body.pos, a.body.fwd), (2, 2, [10.0, 20.0, 30.0], [1.0, 0.0, 0.0]));
}

// ---------------------------------------------------------------- the flight control

#[test]
fn the_flight_yaw_roll_and_lift() {
    // Body at height 10, moving (2, 0, 5); side (1,0,0), forward (0,0,1); goal (3, 20, 4).
    let mk = || HeliActor::new(body([0.0, 10.0, 0.0], [2.0, 0.0, 5.0]));
    let mut env = Env::new();
    let first_k = {
        let mut r = DriveRng::game_reset();
        (r.next() as i32 as f32 * SCALE * 4.0).clamp(0.5, 1.0)
    };
    // Boost with max > 0: yaw = 8·(dot(flat delta, side) − 0.5·dot(side, v)) = 8·(3 − 1) = 16.
    let mut h = commanded(1, mode::HOVER);
    let mut a = mk();
    let mut rng = DriveRng::game_reset();
    let mut g = [3.0, 20.0, 4.0];
    h.flight(&mut a, &mut env, &mut rng, 0.0, 10.0, &mut g, true);
    assert!(near(a.yaw, 16.0), "boost yaw {}", a.yaw);
    // Roll: 0 → toward dot(n(3,0,4), fwd)·10/50 = 0.16 by 0.02.
    assert!(near(a.roll, 0.02), "roll {}", a.roll);
    // Lift: 2·(min(20 − 10, 10) − 0) + 9.8 = 29.8, × the clamped draw (height 10 > 2).
    assert!(near(a.lift, 29.8 * first_k), "lift {} vs {}", a.lift, 29.8 * first_k);
    assert_eq!(rng.draws, 1, "one draw per call");
    // No boost, not state 5: yaw = −dot(side, v) = −2.
    let mut a = mk();
    h.flight(&mut a, &mut env, &mut rng, 0.0, 10.0, &mut [3.0, 20.0, 4.0], false);
    assert!(near(a.yaw, -2.0), "plain yaw {}", a.yaw);
    // State 5: the raw input 3 while slower than max (|(2,0,5)| = 5.39 < 10), else 0.
    let mut h5 = commanded(5, mode::HOVER);
    let mut a = mk();
    h5.flight(&mut a, &mut env, &mut rng, 0.0, 10.0, &mut [3.0, 20.0, 4.0], false);
    assert!(near(a.yaw, 3.0), "state-5 yaw {}", a.yaw);
    let mut a = mk();
    h5.flight(&mut a, &mut env, &mut rng, 0.0, 5.0, &mut [3.0, 20.0, 4.0], false);
    assert!(near(a.yaw, 0.0) && near(a.roll, 0.0), "state-5 too fast {} {}", a.yaw, a.roll);
    // Mode 3 outside state 5 steers by the probe navigator's carrot: dot((−4, 0, 9) − pos, side) = −4.
    let mut h3 = commanded(3, mode::ROAD);
    env.probe = [-4.0, 10.0, 9.0];
    let mut a = mk();
    h3.flight(&mut a, &mut env, &mut rng, 0.0, 10.0, &mut [3.0, 20.0, 4.0], true);
    assert!(near(a.yaw, 8.0 * (-4.0 - 1.0)), "probe yaw {}", a.yaw);
    // alt ≠ 0 puts the goal at ground + alt: 0 + 5 → e = min(5 − 10, 10) = −5 → 2·(−5) + 9.8.
    let mut a = mk();
    let mut g = [3.0, 99.0, 4.0];
    let mut r2 = DriveRng::game_reset();
    let k2 = {
        let mut r = r2.clone();
        (r.next() as i32 as f32 * SCALE * 4.0).clamp(0.5, 1.0)
    };
    h.flight(&mut a, &mut env, &mut r2, 5.0, 10.0, &mut g, false);
    assert!(near(g[1], 5.0) && near(a.lift, -0.2 * k2), "goal {} lift {}", g[1], a.lift);
    // The hold-down counter: no speed, no boost, counts down.
    let mut hd = commanded(5, mode::HOVER);
    hd.hold_down = 3;
    let mut a = mk();
    hd.flight(&mut a, &mut env, &mut rng, 0.0, 10.0, &mut [3.0, 20.0, 4.0], true);
    assert!(near(a.yaw, 0.0) && hd.hold_down == 2, "hold-down {} {}", a.yaw, hd.hold_down);
    // Within 2 of the ground the roll is left alone and the lift is not scaled.
    let mut a = HeliActor::new(body([0.0, 1.0, 0.0], [2.0, 0.0, 5.0]));
    a.roll = 0.7;
    h.flight(&mut a, &mut env, &mut rng, 0.0, 10.0, &mut [3.0, 3.0, 4.0], false);
    assert!(near(a.roll, 0.7) && near(a.lift, 2.0 * 2.0 + 9.8), "low {} {}", a.roll, a.lift);
}

// ---------------------------------------------------------------- the movement

#[test]
fn hover_speed_and_pitch() {
    let mut env = Env::new();
    let mut rng = DriveRng::game_reset();
    let mut h = commanded(1, mode::HOVER);
    h.radius = 15.0;
    h.speed_k = 1.1;
    h.speed_cap = 35.0;
    h.goal = [30.0, 10.0, 40.0];
    let mut a = HeliActor::new(body([0.0, 10.0, 0.0], [0.0; 3]));
    h.movement(&mut a, &mut env, &mut rng, true, false);
    // dist 50 ≥ 15: min(50·1.1, 35) = 35 → the roll target dot(0.6,0,0.8 · fwd)·35/50 = 0.56.
    assert!(near(h.dist, 50.0) && near(a.roll, 0.02));
    // The pitch is dot(+0xc0, flat side) = 0.6.
    assert!(near(a.pitch, 0.6), "pitch {}", a.pitch);
    // Inside the radius the speed is 0: the roll goes to 0 (flat speed 0 < 0 is false).
    h.goal = [3.0, 10.0, 4.0];
    a.roll = 0.3;
    h.movement(&mut a, &mut env, &mut rng, true, false);
    assert!(near(a.roll, 0.0), "roll {}", a.roll);
    assert_eq!(rng.draws, 2);
}

#[test]
fn road_movement_calls() {
    let mut env = Env::new();
    let mut rng = DriveRng::game_reset();
    let mut h = commanded(3, mode::ROAD);
    h.radius = 5.0;
    h.speed_k = 1.1;
    h.speed_cap = 35.0;
    h.altitude = 20.0;
    h.goal = [0.0, 0.0, 100.0];
    // Own speed 30 → advance clamp(15, 10, 20) = 15; the carrot 4 away < 15 → advance 2·15 − 4.
    let mut a = HeliActor::new(body([0.0, 30.0, 0.0], [0.0, 0.0, 30.0]));
    env.carrot = [0.0, 30.0, 4.0];
    env.road = 7;
    env.target_road = 7;
    env.dir = [0.0, 0.0, -1.0]; // the road runs away from the goal → re-join
    h.movement(&mut a, &mut env, &mut rng, true, false);
    assert_eq!(env.calls, vec!["rejoin", "reset", "advance 15.000 None", "advance 26.000 Some(7)"]);
    assert_eq!(h.road, 7);
    // The same road heading toward the goal: no re-join.
    env.calls.clear();
    env.dir = [0.0, 0.0, 1.0];
    env.carrot = [0.0, 30.0, 40.0];
    h.movement(&mut a, &mut env, &mut rng, true, false);
    assert!(env.calls.is_empty(), "{:?}", env.calls);
    // Another road: the route query decides.
    env.target_road = 9;
    env.differs = true;
    h.movement(&mut a, &mut env, &mut rng, true, false);
    assert_eq!(env.calls[0], "rejoin");
}

#[test]
fn path_mode_copies_the_pose() {
    let mut env = Env::new();
    let mut rng = DriveRng::game_reset();
    let mut h = commanded(1, mode::PATH);
    let mut a = HeliActor::new(body([0.0; 3], [0.0; 3]));
    // No path object: nothing moves, no draw.
    h.movement(&mut a, &mut env, &mut rng, true, false);
    assert_eq!(a.body.pos, [0.0; 3]);
    h.has_path = true;
    env.path = Some(PathPose { side: [0.0, 0.0, -1.0], up: [0.0, 1.0, 0.0], fwd: [1.0, 0.0, 0.0], pos: [5.0, 6.0, 7.0], vel: [1.0, 0.0, 0.0] });
    a.a8 = true;
    h.movement(&mut a, &mut env, &mut rng, true, false);
    assert_eq!((a.body.pos, a.body.vel, a.body.fwd, a.a8), ([5.0, 6.0, 7.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0], false));
    assert_eq!(rng.draws, 0, "the path flies without the flight control");
}

// ---------------------------------------------------------------- the weapons

#[test]
fn the_gun_cone_cadence_and_spread() {
    let mut env = Env::new();
    let mut ev = Vec::new();
    let mut h = commanded(9, mode::HOVER);
    h.interval = 0.5;
    let mut a = HeliActor::new(body([0.0; 3], [0.0; 3]));
    let mut rng = DriveRng::game_reset();
    let mut want = rng.clone();
    // Straight ahead at 50 (range 65): a shot, three draws, the spread of the level-aim arm.
    h.gun(65.0, &mut a, [0.0; 3], [0.0, 0.0, 50.0], [0.0, 0.0, 1.0], &mut env, &mut rng, &mut ev);
    let (r1, r2, r3) = (want.next() as i32 as f32 * SCALE, want.next() as i32 as f32 * SCALE, want.next() as i32 as f32 * SCALE);
    let base = [0.0, 0.0, 1.0]; // n(aim + 5·dir)
    let exp = [base[0] + (r1 - 0.5) * 0.025, base[1] - r2 * 0.15, base[2] + (r3 - 0.5) * 0.025];
    assert_eq!(ev.len(), 1);
    assert!(near(h.gun_aim[0], exp[0]) && near(h.gun_aim[1], exp[1]) && near(h.gun_aim[2], exp[2]), "{:?} vs {exp:?}", h.gun_aim);
    assert_eq!((rng.draws, h.gun_timer, a.firing), (3, 30, true), "(short)(int)(60·0.5)");
    // The timer counts down first.
    for _ in 0..30 {
        h.gun(65.0, &mut a, [0.0; 3], [0.0, 0.0, 50.0], [0.0, 0.0, 1.0], &mut env, &mut rng, &mut ev);
    }
    assert_eq!((ev.len(), h.gun_timer), (1, 0));
    // Outside the 0.707 cone (45° off is 0.7071 — just in; 50° is out): flags off, no shot.
    let off = [50.0f32.to_radians().sin() * 50.0, 0.0, 50.0f32.to_radians().cos() * 50.0];
    h.gun(65.0, &mut a, [0.0; 3], off, [0.0, 0.0, 1.0], &mut env, &mut rng, &mut ev);
    assert_eq!((ev.len(), a.firing, rng.draws), (1, false, 3));
    // Out of range: nothing changes (the flags keep their value).
    a.firing = true;
    h.gun(65.0, &mut a, [0.0; 3], [0.0, 0.0, 66.0], [0.0, 0.0, 1.0], &mut env, &mut rng, &mut ev);
    assert!(a.firing && ev.len() == 1);
    // A downward aim takes the ±0.05 arm on every axis.
    let mut want = rng.clone();
    h.gun(65.0, &mut a, [0.0; 3], [0.0, -50.0, 1.0], [0.0, -1.0, 0.0], &mut env, &mut rng, &mut ev);
    let r = [want.next(), want.next(), want.next()].map(|x| (x as i32 as f32 * SCALE - 0.5) * 0.05);
    // base = n(aim + 5·n(to − from)) with to − from = (0, −50, 1).
    let l = (50.0f32 * 50.0 + 1.0).sqrt();
    let v = [0.0, -1.0 - 5.0 * 50.0 / l, 5.0 / l];
    let lv = (v[1] * v[1] + v[2] * v[2]).sqrt();
    let base = [0.0, v[1] / lv, v[2] / lv];
    assert!(near(h.gun_aim[0], base[0] + r[0]) && near(h.gun_aim[1], base[1] + r[1]) && near(h.gun_aim[2], base[2] + r[2]), "{:?}", h.gun_aim);
    // A negative interval never fires.
    h.interval = -1.0;
    h.gun_timer = 0;
    h.gun(65.0, &mut a, [0.0; 3], [0.0, 0.0, 50.0], [0.0, 0.0, 1.0], &mut env, &mut rng, &mut ev);
    assert_eq!(ev.len(), 2);
}

#[test]
fn the_rockets_alternate() {
    let mut env = Env::new();
    let mut ev = Vec::new();
    let mut h = commanded(10, mode::HOVER);
    h.interval = 0.2;
    h.rockets = 2;
    let mut a = HeliActor::new(body([0.0; 3], [0.0; 3]));
    let to = [0.0, -30.0, 100.0];
    h.rockets_fire(1000.0, 0.0, &mut a, [0.0; 3], to, [0.0, 0.0, 1.0], &mut env, &mut ev);
    // Launcher 8 first; the timer max(60·0.2, 60·0.5) = 30; the toggle flips.
    assert_eq!(ev, vec![HeliEvent::Rocket { channel: 8, target: to }]);
    assert_eq!((h.rockets, h.rocket_timer, h.launcher, h.rocket_target), (1, 30, 1, to));
    for _ in 0..30 {
        h.rockets_fire(1000.0, 0.0, &mut a, [0.0; 3], to, [0.0, 0.0, 1.0], &mut env, &mut ev);
    }
    h.rockets_fire(1000.0, 0.0, &mut a, [0.0; 3], to, [0.0, 0.0, 1.0], &mut env, &mut ev);
    assert_eq!(ev[1], HeliEvent::Rocket { channel: 7, target: to });
    // State 10 with none left: held.
    assert_eq!(h.rockets, 0);
    h.rocket_timer = 0;
    h.rockets_fire(1000.0, 0.0, &mut a, [0.0; 3], to, [0.0, 0.0, 1.0], &mut env, &mut ev);
    assert_eq!(ev.len(), 2);
    // Moving faster than 5 with a level aim holds the shot; outside the 0.8434 cone too.
    let mut h = commanded(7, mode::ROAD);
    h.interval = 1.0;
    h.rockets_fire(1000.0, 6.0, &mut a, [0.0; 3], to, [0.0, 0.0, 1.0], &mut env, &mut ev);
    assert_eq!(ev.len(), 2);
    let wide = [35.0f32.to_radians().sin() * 100.0, 0.0, 35.0f32.to_radians().cos() * 100.0];
    h.rockets_fire(1000.0, 0.0, &mut a, [0.0; 3], wide, [0.0, 0.0, 1.0], &mut env, &mut ev);
    assert_eq!(ev.len(), 2, "cos 35° = 0.819 < 0.8434");
    // A busy launcher holds the shot and keeps the toggle.
    env.busy = [false, true];
    h.rockets_fire(1000.0, 0.0, &mut a, [0.0; 3], to, [0.0, 0.0, 1.0], &mut env, &mut ev);
    assert_eq!((ev.len(), h.launcher, h.rocket_timer), (2, 0, 0));
    // Not in state 10, a zero count does not hold the rockets (it wraps to 0xff).
    env.busy = [false; 2];
    h.rockets_fire(1000.0, 0.0, &mut a, [0.0; 3], to, [0.0, 0.0, 1.0], &mut env, &mut ev);
    assert_eq!((ev.len(), h.rockets, h.rocket_timer), (3, 0xff, 60));
}

// ---------------------------------------------------------------- the state machine

#[test]
fn approach_state_leads_and_guns_only_armed() {
    let mut env = Env::new();
    env.target = body([0.0, 0.0, 100.0], [0.0, 0.0, 10.0]);
    let mut rng = DriveRng::game_reset();
    let mut ev = Vec::new();
    let mut h = commanded(3, mode::HOVER);
    h.radius = 15.0;
    h.lead = true;
    h.speed_k = 1.1;
    h.speed_cap = 35.0;
    h.interval = 1.0;
    let mut a = HeliActor::new(body([0.0, 20.0, 0.0], [0.0; 3]));
    h.update(&mut a, &mut env, &mut rng, &mut ev);
    // The goal leads the target by its velocity × 2·radius: 100 + 10·30 = 400.
    assert_eq!(h.goal, [0.0, 0.0, 400.0]);
    // No rockets (+0xdc = 0, MIS3's 0.1): no gun, flags off, one flight draw.
    assert!(ev.is_empty() && !a.firing && rng.draws == 1);
    // Armed: the target at 100 is out of the 65 gun range.
    h.rockets = 1;
    h.update(&mut a, &mut env, &mut rng, &mut ev);
    assert!(ev.is_empty());
    env.target = body([0.0, 20.0, 50.0], [0.0; 3]);
    h.update(&mut a, &mut env, &mut rng, &mut ev);
    assert_eq!(ev.len(), 1, "inside 65, dead ahead");
    assert_eq!(rng.draws, 3 + 3);
}

#[test]
fn posts_face_and_fire_at_the_point() {
    let mut env = Env::new();
    env.target = body([100.0, 0.0, 0.0], [0.0; 3]);
    let mut rng = DriveRng::game_reset();
    let mut ev = Vec::new();
    let mut h = commanded(10, mode::HOVER);
    h.interval = 5.0;
    h.rockets = 0xff;
    h.post = [0.0, -50.0, 200.0];
    h.radius = 5.0;
    h.speed_cap = 10.0;
    h.speed_k = 1.0;
    h.goal = [0.0, 20.0, 0.0];
    let mut a = HeliActor::new(body([0.0, 20.0, 0.0], [0.0; 3]));
    h.update(&mut a, &mut env, &mut rng, &mut ev);
    // +0x78 clear: it faces the target (+x) → pitch = dot(+0xc0, side) = 1; aim stays the
    // forward row and the rocket goes at the post (dead ahead, aim.y 0 > −0.05 but speed 0 ≤ 5).
    assert!(near(a.pitch, 1.0) && near(h.flat_dir[0], 1.0));
    assert_eq!(ev, vec![HeliEvent::Rocket { channel: 8, target: [0.0, -50.0, 200.0] }]);
    assert_eq!(h.rocket_timer, 300, "(short)(int)max(60·5, 30)");
    // The post's goal is not overwritten by the target (states 2/9/10).
    assert_eq!(h.goal, [0.0, 20.0, 0.0]);
    // +0x78 set: it faces the post instead.
    h.hold = 1;
    h.update(&mut a, &mut env, &mut rng, &mut ev);
    assert!(near(h.flat_dir[2], 1.0) && near(a.pitch, 0.0), "{:?}", h.flat_dir);
    // Nearly facing (|pitch| < 0.2) the aim becomes +0xb0; state 9 uses the gun.
    let mut g = commanded(9, mode::HOVER);
    g.post = [0.0, 20.0, 50.0];
    g.hold = 1;
    g.interval = 1.0;
    g.radius = 5.0;
    let mut ev = Vec::new();
    let mut b = HeliActor::new(body([0.0, 20.0, 0.0], [0.0; 3]));
    g.update(&mut b, &mut env, &mut rng, &mut ev);
    assert_eq!(ev.len(), 1);
    assert!(matches!(ev[0], HeliEvent::Gun { .. }));
}

#[test]
fn parked_and_dead() {
    let mut env = Env::new();
    let mut rng = DriveRng::game_reset();
    let mut ev = Vec::new();
    let mut h = commanded(2, mode::HOVER);
    h.goal = [1.0, 2.0, 3.0];
    let mut a = HeliActor::new(body([9.0; 3], [4.0; 3]));
    h.update(&mut a, &mut env, &mut rng, &mut ev);
    assert_eq!((a.body.pos, a.body.vel, rng.draws), ([1.0, 2.0, 3.0], [0.0; 3], 0));
    // Health 0 → state 16; airborne with nothing hit: the fall values.
    let mut h = commanded(1, mode::HOVER);
    let mut a = HeliActor::new(body([0.0, 30.0, 0.0], [0.0, -5.0, 10.0]));
    a.health = 0.0;
    a.pitch = -0.95;
    a.roll = 0.995;
    a.lift = 20.0;
    a.length = 6.0;
    a.firing = true;
    h.update(&mut a, &mut env, &mut rng, &mut ev);
    assert_eq!((h.state, h.mode, a.firing), (16, 1, false));
    assert!(near(a.pitch, -1.0) && near(a.roll, 1.0) && near(a.lift, 0.25) && near(a.spin, -2.0));
    assert!(ev.is_empty());
    // The sweep hits: the blast, the velocity × 0.35, the roll 0, +0xde counted …
    env.hit = true;
    h.update(&mut a, &mut env, &mut rng, &mut ev);
    assert_eq!(ev, vec![HeliEvent::Blast { pos: [0.0, 30.0, 0.0] }]);
    assert!(near(a.body.vel[2], 3.5) && near(a.roll, 0.0) && h.dying == 1);
    // … and the next update's death check despawns it.
    h.update(&mut a, &mut env, &mut rng, &mut ev);
    assert_eq!((ev.len(), ev[1], h.dying), (2, HeliEvent::Despawn, 0));
    // On the ground with contact the impact is immediate.
    let mut h = commanded(16, mode::NONE);
    let mut a = HeliActor::new(body([0.0, 0.5, 0.0], [0.0; 3]));
    a.contact = true;
    env.hit = false;
    let mut ev = Vec::new();
    h.update(&mut a, &mut env, &mut rng, &mut ev);
    assert_eq!(ev.len(), 1);
    // A proximity mine blasts within its radius of the target's box.
    let mut h = commanded(1, mode::PATH);
    let mut a = HeliActor::new(body([0.0, 0.0, 95.0], [0.0; 3]));
    a.mine_radius = Some(3.0);
    let mut ev = Vec::new();
    h.update(&mut a, &mut env, &mut rng, &mut ev); // |5| − box 2 = 3 ≤ 3
    assert_eq!((ev.len(), h.dying), (1, 1));
}

#[test]
fn the_update_gate_and_goal_tracking() {
    let mut env = Env::new();
    env.target = body([7.0, 0.0, 8.0], [0.0; 3]);
    let mut rng = DriveRng::game_reset();
    let mut ev = Vec::new();
    let mut a = HeliActor::new(body([0.0, 20.0, 0.0], [0.0; 3]));
    let mut h = Heli::reset();
    h.update(&mut a, &mut env, &mut rng, &mut ev);
    assert_eq!((h.goal, rng.draws), ([0.0; 3], 0), "not attached / commanded");
    let mut h = commanded(1, mode::HOVER);
    h.radius = 5.0;
    h.update(&mut a, &mut env, &mut rng, &mut ev);
    assert_eq!(h.goal, [7.0, 0.0, 8.0], "state 1 with +0x78 clear tracks the target");
    h.hold = 1;
    env.target.pos = [1.0; 3];
    h.update(&mut a, &mut env, &mut rng, &mut ev);
    assert_eq!(h.goal, [7.0, 0.0, 8.0], "+0x78 set keeps the goal");
}

// ---------------------------------------------------------------- the riders

#[test]
fn the_seat_classes_and_ranges() {
    let w = seat_class(3).unwrap();
    assert_eq!((w.attr, w.vtable, w.ctor, w.first_state, w.anim), ("AIC_EWIND_POS", 0x369200, 0x1224b0, 0xd, (0xd, 2)));
    assert_eq!(seat_class(4).unwrap().vtable, 0x368f08);
    assert_eq!(seat_class(0xe).unwrap().anim, (8, 7));
    assert!(!seat_class(0xf).unwrap().armed && seat_class(0x10).unwrap().anim == (1, 1));
    assert_eq!(seat_class(0x11).unwrap().ctor, 0x11aca8);
    assert!(seat_class(5).is_none() && seat_class(0x12).is_none());
    // 75², ×4 for skill 2/4, ×9 for 3/5.
    assert_eq!((range_sq(0), range_sq(1), range_sq(2), range_sq(3), range_sq(4), range_sq(5), range_sq(6)), (5625.0, 5625.0, 22500.0, 50625.0, 22500.0, 50625.0, 5625.0));
    assert_eq!(sunroof_setup(1), (25, 25, 0xb));
    assert_eq!(sunroof_setup(2), (0, 15, 10));
    assert_eq!(sunroof_setup(0), (25, 25, 0xb), "clamped to 1");
    assert_eq!(sunroof_setup(7), (0, 15, 10), "clamped to 2");
    assert!(needs_reload(25, 0) && !needs_reload(0, 0) && !needs_reload(25, 1));
}

#[test]
fn sights_and_aim() {
    let fwd = [0.0, 0.0, 1.0];
    let at = |deg: f32, r: f32| [deg.to_radians().sin() * r, 0.0, deg.to_radians().cos() * r];
    // The window: −90° … +140° (−0.25, 0.3888889 turns), 75 range.
    assert!(window_sight([0.0; 3], fwd, at(0.0, 74.0), range_sq(0)));
    assert!(!window_sight([0.0; 3], fwd, at(0.0, 76.0), range_sq(0)));
    assert!(window_sight([0.0; 3], fwd, at(139.0, 50.0), range_sq(0)));
    assert!(!window_sight([0.0; 3], fwd, at(141.0, 50.0), range_sq(0)));
    assert!(window_sight([0.0; 3], fwd, at(-89.0, 50.0), range_sq(0)));
    assert!(!window_sight([0.0; 3], fwd, at(-91.0, 50.0), range_sq(0)));
    // The snowmobile: ±120°, and the heading is stored whenever the range holds.
    let (seen, a) = snow_sight([0.0; 3], fwd, at(119.0, 50.0), range_sq(0));
    assert!(seen && near(a.unwrap(), 119.0 / 360.0));
    let (seen, a) = snow_sight([0.0; 3], fwd, at(121.0, 50.0), range_sq(0));
    assert!(!seen && a.is_some());
    assert_eq!(snow_sight([0.0; 3], fwd, at(0.0, 80.0), range_sq(0)), (false, None));
    // The sunroof holds while another rider is up; the vehicle gun is range only.
    assert!(sunroof_sight([0.0; 3], at(0.0, 70.0), range_sq(0), [false, false]));
    assert!(!sunroof_sight([0.0; 3], at(0.0, 70.0), range_sq(0), [false, true]));
    assert!(vgun_sight([0.0; 3], at(180.0, 149.0), range_sq(2)));
    // The aim test: inside ±2°, +0x84 = the side.
    assert_eq!(aim_test([0.0; 3], fwd, at(1.9, 10.0)), (true, true));
    assert_eq!(aim_test([0.0; 3], fwd, at(-1.9, 10.0)), (true, false));
    assert_eq!(aim_test([0.0; 3], fwd, at(2.1, 10.0)), (false, true));
    // The controller's target wins over the player.
    assert_eq!(pick_target(Some([1.0; 3]), [2.0; 3]), [1.0; 3]);
    assert_eq!(pick_target(None, [2.0; 3]), [2.0; 3]);
}

#[test]
fn rider_fire_cadence() {
    let mut rng = DriveRng::game_reset();
    let mut want = rng.clone();
    let mut sweeps = 0;
    let mut g = Gunner::base();
    assert!(window_acquire(&mut g, false, true, 60));
    assert_eq!((g.state, g.hold), (2, 18000));
    // Fire now: shot, next = now + 60, the animation (r·5)>>16.
    let f = window_fire(&mut g, 100, false, false, &mut || sweeps += 1, &mut || rng.next());
    let r = want.next();
    assert_eq!(f, Fire::Shot(((r as i32 * 5) >> 16) as u32));
    assert_eq!((g.state, g.shots, g.next, sweeps), (0xb, 14, 160, 1));
    // Before the next tick: only the sweep.
    assert_eq!(window_fire(&mut g, 159, false, false, &mut || sweeps += 1, &mut || rng.next()), Fire::Wait);
    assert_eq!((sweeps, rng.draws), (2, 1));
    // The hold run out and no sight: stow.
    g.hold = 0;
    assert_eq!(window_fire(&mut g, 200, false, false, &mut || sweeps += 1, &mut || rng.next()), Fire::Stow);
    assert_eq!(g.state, 0xd);
    // The per-tick count-down and the dead car.
    let mut t = Gunner::base();
    t.hold = 2;
    window_tick(&mut t, 1.0);
    assert_eq!(t.hold, 1);
    window_tick(&mut t, 0.0);
    assert_eq!(t.state, 0xf);
    // Snowmobile: off target the animation index is 0 but the draw happens.
    let mut s = Gunner::base();
    let d0 = rng.draws;
    assert_eq!(snow_fire(&mut s, 0, true, false, &mut || rng.next()), Fire::Shot(0));
    assert_eq!(rng.draws, d0 + 1);
    assert_eq!(snow_fire(&mut s, 0, false, true, &mut || rng.next()), Fire::Stow);
    // Sunroof: reload when empty, shots only on target, the index offset 6·weapon.
    let mut u = Gunner::base();
    let (clip, shots, _) = sunroof_setup(1);
    u.clip = clip;
    u.shots = 0;
    assert_eq!(sunroof_fire(&mut u, 0, false, true, false, true, 1, &mut || (), &mut || rng.next()), Fire::Reload);
    assert_eq!((u.shots, u.state), (shots, 0xc));
    assert_eq!(sunroof_fire(&mut u, 0, false, true, false, false, 1, &mut || (), &mut || rng.next()), Fire::Wait);
    let mut want = rng.clone();
    let f = sunroof_fire(&mut u, 0, false, true, false, true, 1, &mut || (), &mut || rng.next());
    assert_eq!(f, Fire::Shot(((want.next() as i32 * 6) >> 16) as u32 + 6));
}

// ---------------------------------------------------------------- the disc (NF_ISO)

const MISSIONS: [&str; 8] = ["MIS01.VIV", "MIS11.VIV", "MIS13A.VIV", "MIS13B.VIV", "MIS13C.VIV", "MIS3.VIV", "MIS4.VIV", "RACE.VIV"];

fn track(iso: &IsoImage, mission: &str) -> Vec<u8> {
    let e = iso.entries().iter().find(|e| e.path == format!("/DRIVING/{mission}")).cloned().unwrap();
    let bigf = nf_formats::disc::read_bigf(iso, &e).unwrap();
    let base = e.lba as u64 * 2048;
    let x = bigf.entries.iter().find(|x| {
        let n = x.name.to_ascii_lowercase();
        n.contains("track") && n.ends_with(".crp")
    });
    let x = x.unwrap();
    let raw = iso.read_bytes(base + x.offset as u64, x.size as u64).unwrap();
    if refpack::is_refpack(&raw) { refpack::decompress(&raw).unwrap() } else { raw }
}

fn mset(crp: &Crp) -> Option<MissionSet> {
    let map = crp.pool_find(nf_formats::crp::tag::MAP).ok()??;
    for r in crp.record_window(&map).ok()? {
        if r.tag == 0x4d53_6574 {
            return Some(MissionSet::parse(crp.payload(&r).ok()?));
        }
    }
    None
}

#[test]
fn spawn_commands_and_the_paris_helicopter() {
    let Some(path) = std::env::var_os("NF_ISO") else {
        eprintln!("NF_ISO not set — skipping");
        return;
    };
    let iso = IsoImage::open(&path).unwrap();
    for mission in MISSIONS {
        let img = track(&iso, mission);
        let crp = Crp::parse(&img).unwrap();
        let ai = MapAi::from_crp(&crp).unwrap();
        // Every element's +0xf8 resolves to an AICo record by its `{AICo}::<offset>` name.
        let mut n = 0;
        for el in &ai.elements {
            if el.command_ref != 0 {
                assert!(ai.command_index(el.command_ref).is_some(), "{mission}: {:#x} → {:?}", el.command_ref, ai.ref_name(el.command_ref));
                n += 1;
            }
        }
        // Every pattern-5 command's +0x80 names an AISp path.
        for c in &ai.commands {
            if c.pattern == 5 {
                assert!(ai.path_index(c.pattern_arg).is_some(), "{mission}: path {:#x} → {:?}", c.pattern_arg, ai.ref_name(c.pattern_arg));
            }
        }
        eprintln!("{mission}: {n} spawn commands, {} paths, {} sr names", ai.paths.len(), ai.refs.len());
        if mission == "MIS01.VIV" {
            // MIS01's element 0 → sr#1 → `{AICo}::8960` → record 56 (the old `nnnn − 1` gave 0).
            assert_eq!(ai.elements[0].command_ref, 0x7372_0001);
            assert_eq!(ai.command_index(ai.elements[0].command_ref), Some(56));
            // No AIEl helicopter: the Paris helicopter is the player's own vehicle — the MSet
            // name `FUN_00176658` hands to `FUN_002051f8` → `FUN_001ff850(·, 0, name, …)`.
            assert!(!ai.elements.iter().any(|e| e.heli));
            assert_eq!(mset(&crp).unwrap().name, "helicopter");
        }
    }
}

/// The scripted player: from `start` along +x at 15 u/s, weaving [port script].
fn player(start: [f32; 3], tick: i32) -> Body {
    let t = tick as f32 * DT;
    let w = (t * 0.5).sin() * 8.0;
    body([start[0] + 15.0 * t, start[1], start[2] + w], [15.0, 0.0, (t * 0.5).cos() * 4.0])
}

/// The disc world for the 1200-tick runs: flat ground at the spawn's height minus 30 [S]; the
/// navigator a RoadNav joined on the nearest piece by brute force with the traffic chooser
/// (the game's kind-5 chooser and the FUN_00225c48 grid query are not ported) [S]; the path
/// pose the AISp placement, held still (the EAGL track playback is not ported) [S].
struct DiscEnv<'a> {
    net: &'a RoadNet<'a>,
    ground: f32,
    target: Body,
    nav: Option<RoadNav>,
    path: Option<PathPose>,
    busy: [i32; 2],
}

impl DiscEnv<'_> {
    fn nearest(&self, p: [f32; 3]) -> Option<usize> {
        let mut best: Option<(f32, usize)> = None;
        for k in 0..self.net.segs.len() {
            if let Ok(q) = self.net.seg_point(k, 0.5) {
                let d = (q[0] - p[0]).powi(2) + (q[2] - p[2]).powi(2);
                if best.map_or(true, |b| d < b.0) {
                    best = Some((d, k));
                }
            }
        }
        best.map(|b| b.1)
    }
    fn join(&self, p: [f32; 3], dir: [f32; 3]) -> Option<RoadNav> {
        let k = self.nearest(p)?;
        let d = self.net.seg(k).ok()?.dir();
        RoadNav::join(self.net, k, 0.5, 0, d[0] * dir[0] + d[2] * dir[2] >= 0.0, NAV_KIND_TRAFFIC, LaneSet::Default).ok()
    }
}

impl HeliEnv for DiscEnv<'_> {
    fn ground(&mut self, _p: [f32; 3]) -> f32 {
        self.ground
    }
    fn target(&self) -> Body {
        self.target
    }
    fn path_pose(&mut self) -> Option<PathPose> {
        self.path
    }
    fn nav_carrot(&self) -> [f32; 3] {
        self.nav.as_ref().map_or([0.0; 3], |n| n.pos)
    }
    fn nav_dir(&self) -> [f32; 3] {
        self.nav.as_ref().map_or([0.0, 0.0, 1.0], |n| n.dir)
    }
    fn nav_road(&self) -> u16 {
        self.nav.as_ref().map_or(0xffff, |n| n.seg as u16)
    }
    fn nav_rejoin(&mut self, pos: [f32; 3], dir: [f32; 3]) {
        self.nav = self.join(pos, dir);
    }
    fn nav_reset(&mut self) {}
    fn nav_advance(&mut self, dist: f32, _dir: [f32; 3], _road: Option<u16>) {
        let net = self.net;
        if let Some(n) = self.nav.as_mut() {
            let _ = n.advance(net, dist, &mut || 0u16);
        }
    }
    fn target_road(&mut self, _dir: [f32; 3]) -> u16 {
        let p = self.target.pos;
        self.nearest(p).map_or(0xffff, |k| k as u16)
    }
    fn route_differs(&mut self, _road: u16) -> bool {
        false
    }
    fn probe_carrot(&mut self, pos: [f32; 3], fwd: [f32; 3]) -> [f32; 3] {
        self.join(pos, fwd).map_or(pos, |n| n.pos)
    }
    fn crash_hit(&mut self, _a: [f32; 3], _b: [f32; 3]) -> bool {
        false
    }
    fn launcher_busy(&self, ch: u8) -> bool {
        self.busy[(ch - 7) as usize] > 0
    }
}

struct HeliRun {
    draws: u32,
    guns: usize,
    rockets: Vec<u8>,
    start_gap: f32,
    end_gap: f32,
    min_height: f32,
    states: Vec<u32>,
}

/// One helicopter for 1200 ticks: its spawn command, then (optionally at tick 300) a second
/// command; a stand-in integration of the four flight inputs [S: the helicopter physics class
/// is not ported]: vertical acceleration lift − 9.8, heading rate yaw (clamped ±1 rad/s), the
/// flat velocity easing toward forward × roll·50.
fn run_heli(net: &RoadNet, ai: &MapAi, pos: [f32; 3], facing: [f32; 3], first: &AiCommand, second: Option<&AiCommand>) -> HeliRun {
    let mut yaw_ang = facing[0].atan2(facing[2]);
    let rows = |y: f32| {
        let (s, c) = y.sin_cos();
        ([c, 0.0, -s], [s, 0.0, c])
    };
    let (side, fwd) = rows(yaw_ang);
    let mut a = HeliActor::new(Body { pos, vel: [0.0; 3], side, up: [0.0, 1.0, 0.0], fwd, box_z: 4.0, box_x: 2.0 });
    a.length = 8.0;
    let pstart = [pos[0] - 120.0, pos[1] - 30.0, pos[2]];
    let mut env = DiscEnv { net, ground: pos[1] - 30.0, target: player(pstart, 0), nav: None, path: None, busy: [0; 2] };
    let path_of = |c: &AiCommand| -> Option<PathPose> {
        let p = &ai.paths[ai.path_index(c.pattern_arg)?];
        let m = p.placement;
        Some(PathPose { side: [m[0][0], m[0][1], m[0][2]], up: [m[1][0], m[1][1], m[1][2]], fwd: [m[2][0], m[2][1], m[2][2]], pos: [m[3][0], m[3][1], m[3][2]], vel: [0.0; 3] })
    };
    let mut h = Heli::reset();
    h.attached = true;
    let apply = |h: &mut Heli, a: &mut HeliActor, env: &mut DiscEnv, c: &AiCommand| {
        let path = if c.pattern == 5 { path_of(c) } else { None };
        h.apply_command(c, a, path.is_some());
        if c.pattern == 5 {
            env.path = path;
        }
        if h.mode == mode::ROAD {
            env.nav = env.join(a.body.pos, a.body.fwd);
        }
    };
    apply(&mut h, &mut a, &mut env, first);
    let mut rng = DriveRng::game_reset();
    let mut ev = Vec::new();
    let mut guns = 0;
    let mut rockets = Vec::new();
    let start_gap = ((a.body.pos[0] - env.target.pos[0]).powi(2) + (a.body.pos[2] - env.target.pos[2]).powi(2)).sqrt();
    let mut min_height = f32::MAX;
    let mut states = vec![h.state];
    for tick in 0..1200 {
        if tick == 300 {
            if let Some(c) = second {
                apply(&mut h, &mut a, &mut env, c);
                states.push(h.state);
            }
        }
        env.target = player(pstart, tick);
        for b in env.busy.iter_mut() {
            *b -= 1;
        }
        ev.clear();
        h.update(&mut a, &mut env, &mut rng, &mut ev);
        for e in &ev {
            match e {
                HeliEvent::Gun { aim } => {
                    assert!(aim.iter().all(|x| x.is_finite()));
                    guns += 1;
                }
                HeliEvent::Rocket { channel, .. } => {
                    rockets.push(*channel);
                    env.busy[(*channel - 7) as usize] = 30; // a launcher reloads for half a second [S]
                }
                _ => {}
            }
        }
        if h.mode != mode::PATH && h.state != 2 {
            let b = &mut a.body;
            b.vel[1] += (a.lift - 9.8) * DT;
            yaw_ang += a.yaw.clamp(-1.0, 1.0) * DT;
            let (s, f) = rows(yaw_ang);
            b.side = s;
            b.fwd = f;
            let want = [f[0] * a.roll * 50.0, f[2] * a.roll * 50.0];
            b.vel[0] += (want[0] - b.vel[0]) * DT;
            b.vel[2] += (want[1] - b.vel[2]) * DT;
            for k in 0..3 {
                b.pos[k] += b.vel[k] * DT;
            }
        }
        assert!(a.body.pos.iter().all(|x| x.is_finite()) && a.lift.is_finite() && a.yaw.is_finite());
        min_height = min_height.min(a.body.pos[1] - env.ground);
    }
    let end_gap = ((a.body.pos[0] - env.target.pos[0]).powi(2) + (a.body.pos[2] - env.target.pos[2]).powi(2)).sqrt();
    HeliRun { draws: rng.draws, guns, rockets, start_gap, end_gap, min_height, states }
}

#[test]
fn each_helicopter_for_1200_ticks() {
    let Some(path) = std::env::var_os("NF_ISO") else {
        eprintln!("NF_ISO not set — skipping");
        return;
    };
    let iso = IsoImage::open(&path).unwrap();
    let mut runs = 0;
    for mission in ["MIS11.VIV", "MIS13A.VIV", "MIS3.VIV"] {
        let img = track(&iso, mission);
        let crp = Crp::parse(&img).unwrap();
        let ai = MapAi::from_crp(&crp).unwrap();
        let net = RoadNet::parse_crp(&crp).unwrap();
        let helis: Vec<_> = ai.elements.iter().enumerate().filter(|(_, e)| e.kind == TYPE_VEHICLE && e.heli).collect();
        assert!(!helis.is_empty(), "{mission} has helicopter elements");
        // The only AICo record on the three tracks with a helicopter chase state: MIS3's state-3
        // road approach (index 14) [D]. Which trigger issues it to which element was not
        // traced; it is applied at tick 300 to the first mode-2 helicopter [S].
        let approach = ai.commands.iter().find(|c| c.profile == 3);
        let mut done_approach = false;
        for (k, el) in helis {
            let c = ai.spawn_command(el).unwrap_or_else(|| panic!("{mission} element {k}: no spawn command"));
            // The spawn commands put every helicopter in state 1 or 10, mode 2 or 5 [D].
            assert!(matches!(c.profile, 1 | 10) && matches!(c.pattern, 2 | 5), "{mission} {k}: state {} mode {}", c.profile, c.pattern);
            let second = if !done_approach && c.pattern == 2 { approach } else { None };
            done_approach |= second.is_some();
            let r = run_heli(&net, &ai, el.pos, el.facing, c, second);
            eprintln!(
                "{mission} el {k} {}: states {:?} mode {} → draws {}, guns {}, rockets {:?}, gap {:.0} → {:.0}, min height {:.1}",
                el.name, r.states, c.pattern, r.draws, r.guns, r.rockets, r.start_gap, r.end_gap, r.min_height
            );
            if c.pattern == 5 {
                // A path helicopter never runs the flight control: no draws, and it sits on its
                // (held) path pose.
                assert_eq!(r.draws as usize, 3 * r.guns, "{mission} {k}");
            } else {
                // Hover/road: one flight-control draw every tick, plus three per gun shot.
                assert_eq!(r.draws as usize, 1200 + 3 * r.guns, "{mission} {k}");
            }
            if second.is_some() {
                // MIS3's approach command carries 0.1 rockets → +0xdc = 0: the approach never
                // guns [M 0x139be8 case 3 + D].
                assert_eq!((r.states.clone(), r.guns), (vec![1, 3], 0), "{mission} {k}");
            }
            if c.profile == 10 {
                // A rocket post: launchers alternate 8, 7, 8, …; with +0x64 = 5 the timer is
                // 300 ticks, so at most 1200/301 + 1 shots.
                for (i, ch) in r.rockets.iter().enumerate() {
                    assert_eq!(*ch, if i % 2 == 0 { 8 } else { 7 });
                }
                assert!(r.rockets.len() <= 4, "{:?}", r.rockets);
            } else {
                assert!(r.rockets.is_empty(), "{mission} {k}: state {} fired rockets", c.profile);
            }
            runs += 1;
        }
        if mission == "MIS3.VIV" {
            assert!(done_approach, "MIS3's approach command ran");
        }
    }
    assert!(runs >= 30, "{runs} helicopters");
}
