//! The driving missions' AI car: the per-frame brain of a civilian traffic car (AIVehicle state 4,
//! profile 0xc, pattern 3 — the set the respawn `FUN_00132808` 0x132808 gives every live traffic
//! car), its road navigator (WRoadNav kind 1) and the drive-control applier, headless. Every item
//! was checked against the DRIVING.ELF decompile; the per-function review table with the verdicts
//! is docs/systems/driving-ai.md §9.
//!
//! Frame order [M FUN_0012e350 0x12e350 case 4]: `FUN_00132d80` (route bookkeeping + respawn) →
//! `FUN_0012e808` (target speed → +0x8c) → `FUN_00130508` (pattern 3: lookahead
//! `FUN_00132f80`, carrot `FUN_00130140` → road advance `FUN_00224e00`, then the drive control
//! `FUN_00133ab0` → the actor's SetSteer/SetThrottle/SetBrake/SetHandbrake).
//!
//! The car is a physics body steered at a carrot point: the navigator keeps a point on the road
//! (lane-offset) at least `lookahead` ahead, and the control pass turns and pedals toward it. The
//! old port drove the car on rails at the segment's speed limit; the game does neither (see §9).

use super::physics::{Car, CarInput, CarParams, FlatWorld, Rigid};
use super::road::{lane_offset, LaneSet, RoadNet, RSF_TURN_RESTR_A, RSF_TURN_RESTR_B};

/// The AIVehicle states (+0x70) [M 0x12e350's switch].
pub const STATE_STANDBY: u32 = 1;
pub const STATE_RETARGET: u32 = 2;
pub const STATE_TELEPORT: u32 = 3;
pub const STATE_TRAFFIC: u32 = 4;
pub const STATE_CHASE: u32 = 5;

/// The speed profiles (+0x84) `FUN_0012e808` switches on [M 0x12e808]. 1 = parked
/// (`FUN_00132ba8` 0x132ba8:41), 0xc = live traffic (`FUN_00132808` 0x132808:130,
/// `FUN_0015c080` 0x15c080:67), 0xf = road-limit follower (`FUN_00130e88` 0x130e88:34).
pub const PROFILE_PARKED: u32 = 0x1;
pub const PROFILE_TRAFFIC: u32 = 0xc;
pub const PROFILE_ROAD_LIMIT: u32 = 0xf;
/// The drive pattern (+0x78) `FUN_00130508` switches on; 3 = road follow [M 0x12e468 / 0x130508].
pub const PATTERN_ROAD: u32 = 3;
/// The road navigator kinds (nav+0x00) [M FUN_0012e468 0x12e468: a state-4 car with a pattern
/// other than 1 gets kind 1; FUN_00224e00 0x224e00:127-141 dispatches the next-segment chooser on
/// it — 4 → 0x2235c8, 3/5 → 0x223a30, 1 → 0x223f98, 2 → 0x2247b0].
pub const NAV_KIND_TRAFFIC: u32 = 1;

/// Profile 0xc's target speed: the constant 0x40f00000 [M 0x12e808 `iVar2 == 0xc`].
pub const TRAFFIC_SPEED: f32 = 7.5;

/// The FUN_00133ab0 constants [M — the literals in the decompile and the disassembly].
pub const THROTTLE_RAMP_SPAN: f32 = 7.5;
pub const THROTTLE_FLOOR: f32 = 0.05;
pub const TARGET_FLOOR: f32 = 0.5;
pub const ARRIVAL_MIN_RADIUS: f32 = 5.0;
pub const ARRIVAL_SPEED_FACTOR: f32 = 0.5;
/// The aim-behind test: `dot(forward, dir) ≤ −0.2` → full lock (0xbe4ccccd) [M 0x133ab0:131-133].
pub const FULL_LOCK_DOT: f32 = -0.2;
/// The brake-side hold: `min(|Δ|, 7.5) ≤ 1.0` with `target ≥ 2.5` → no pedal [M 0x133ab0:158-160].
pub const BRAKE_HOLD_SPAN: f32 = 1.0;
pub const BRAKE_HOLD_TARGET: f32 = 2.5;
/// FUN_00132f80's lookahead: `max(|v|·0.35, box_z + 7.5)` [M 0x132f80:56-58].
pub const LOOKAHEAD_SPEED_FACTOR: f32 = 0.35;
pub const LOOKAHEAD_PAD: f32 = 7.5;
/// The sim box extent floor FUN_001f6e98 writes into +0x4c0/4c4/4c8 [M 0x1f6e98:157-163].
pub const BOX_FLOOR: f32 = 0.001;
/// FUN_00130508's gated stop: the creep pedal `min(|v|, 0.25)` once `|v| ≥ 0.1` [M 0x130508:119,171].
pub const GATED_PEDAL_MAX: f32 = 0.25;
pub const GATED_MIN_SPEED: f32 = 0.1;
/// FUN_00223f98's junction take chance: `rand/65536 < 0.3` (0x3fd3333333333333) [M 0x223f98].
pub const JUNCTION_TAKE_CHANCE: f64 = 0.3;

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn len(a: [f32; 3]) -> f32 {
    let s = dot(a, a);
    if s <= 0.0 {
        0.0
    } else {
        s.sqrt()
    }
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// The VU normalize `v · rsqrt(|v|²)`; a zero vector stays zero [S — the VU's RSQRT of 0 is the
/// largest float, times 0 gives 0].
fn normalize(a: [f32; 3]) -> [f32; 3] {
    let l = len(a);
    if l > 0.0 {
        [a[0] / l, a[1] / l, a[2] / l]
    } else {
        [0.0; 3]
    }
}

fn lerp(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

// ------------------------------------------------------------------------------------------
// The driving world's random generator
// ------------------------------------------------------------------------------------------

/// The driving world's random generator — the object at *0x339190 the traffic choices draw from.
/// `FUN_0017dc28` 0x17dc28: `p = state·mult; state = p & 0xffff; return (p >> 8) & 0xffff`;
/// ctor `FUN_0017db68` sets mult = 0x075bcd15; the world reset `FUN_001feda0` → `FUN_0017dbb0`
/// sets state = 0xf874af01 and draws `mult % 50` (= 39) times [M]. Every other driving system
/// draws from the same object, so the stream position at a given traffic choice is unproven;
/// this generator is the driving engine's own — it shares nothing with the on-foot stream.
#[derive(Debug, Clone)]
pub struct DriveRng {
    pub state: u32,
    pub mult: u32,
    pub draws: u32,
}

impl DriveRng {
    /// The state after `FUN_001feda0`'s reset [M 0x17db68 + 0x17dbb0].
    pub fn game_reset() -> DriveRng {
        let mut r = DriveRng { state: 0xf874_af01, mult: 0x075b_cd15, draws: 0 };
        for _ in 0..(r.mult % 50) {
            r.next();
        }
        r.draws = 0; // FUN_0017dbb0 zeroes +0x0c before the warm-up; the port counts from here
        r
    }

    /// One `FUN_0017dc28` draw [M 0x17dc28].
    pub fn next(&mut self) -> u16 {
        let p = self.state.wrapping_mul(self.mult);
        self.draws = self.draws.wrapping_add(1);
        self.state = p & 0xffff;
        ((p >> 8) & 0xffff) as u16
    }
}

// ------------------------------------------------------------------------------------------
// The sim-object view
// ------------------------------------------------------------------------------------------

/// The sim-object fields the AI step reads (`FUN_001ff088` 0x1ff088's object).
pub struct SimCar {
    /// sim +0x10: the position [M 0x133ab0:41].
    pub pos: [f32; 3],
    /// sim +0x20: the velocity [M 0x133ab0:67,104 — the speed is its length].
    pub vel: [f32; 3],
    /// sim+0x5c → row at +0x20: the facing the forward/velocity dots use [M 0x133ab0:34-36].
    pub forward: [f32; 3],
    /// sim+0x5c → row at +0x00: the side row the steer dot uses [M 0x133ab0:38,57-63].
    pub right: [f32; 3],
    /// The actor's FUN_00195a30 value (the alive gates) [M 0x12e808:12-27].
    pub alive: f32,
}

impl SimCar {
    pub fn speed(&self) -> f32 {
        len(self.vel)
    }

    /// The sim view of the port's physics car. forward = `[sin yaw, 0, cos yaw]` (the row
    /// `Car::step` accelerates along); right = `[cos yaw, 0, −sin yaw]` = d(forward)/d(yaw), the
    /// side a positive steer turns toward in `Car::step` (yaw += speed·tan(steer)/wheelbase).
    /// The game's row 0 / SetSteer sign pairing is not read here; the pairing is the one under
    /// which FUN_00133ab0's `steer = dot(dir, row0)` closes on the aim, which the game's cars
    /// do [S]. `alive` = 1.0 (FUN_00195a30 is the actor's health layer).
    pub fn of_car(car: &Car) -> SimCar {
        let (sin, cos) = car.yaw.sin_cos();
        SimCar { pos: car.pos, vel: car.vel, forward: [sin, 0.0, cos], right: [cos, 0.0, -sin], alive: 1.0 }
    }
}

/// One drive-control output — the four actor-vtbl calls [M 0x133ab0:135-180].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DriveCommand {
    pub steer: f32,
    pub throttle: f32,
    pub brake: f32,
    pub handbrake: bool,
}

impl DriveCommand {
    pub const IDLE: DriveCommand = DriveCommand { steer: 0.0, throttle: 0.0, brake: 0.0, handbrake: false };
}

/// `FUN_00210510` 0x210510 called as (x, 1, 1): clamp to ±1 [M].
fn clamp_unit(x: f32) -> f32 {
    if x > 1.0 {
        1.0
    } else if x < -1.0 {
        -1.0
    } else {
        x
    }
}

/// `FUN_00133ab0` 0x133ab0 — the drive-control applier [M 0x133ab0:31-181 + the disassembly of
/// the pedal tail 0x133e5c-0x133f10]. `aim` = *param_3 (the carrot), `target` = AI +0x8c as
/// passed by FUN_00130508 (no multiplier on this path), `forced_stop` = AI +0x19c.
///
/// - `dir = normalize(aim − pos)`; `lat = dot(dir, right)`; `fa = dot(forward, dir)`;
///   `va = dot(normalize(vel), forward)` [:40-95].
/// - `target' = max(target·|fa|, 0.5)`; `speed = |vel|`; inside `max(speed·0.5, 5)` of the aim
///   the target is 0 [:96-130].
/// - steer = clamp(lat); with the aim behind (`fa ≤ −0.2`) full lock to lat's side [:131-137].
/// - forced stop: throttle 0, brake 0, handbrake 1 [:173-180].
/// - else `span = min(|target' − speed|, 7.5)`, `ramp = max(span/7.5, 0.05)`: `speed < target'`
///   → throttle(ramp); else `va ≤ 0` (stopped or rolling backwards) → throttle(ramp)
///   [0x133ea8 `c.olt.s 0, va` / bc1fl → 0x133f00]; else `span ≤ 1` and `target' ≥ 2.5` → no
///   pedal; else brake(ramp) [:144-172].
pub fn drive_control(sim: &SimCar, aim: [f32; 3], target: f32, forced_stop: bool) -> DriveCommand {
    let d = sub(aim, sim.pos);
    let dir = normalize(d);
    let lat = dot(dir, sim.right);
    let va = dot(normalize(sim.vel), sim.forward);
    let fa = dot(sim.forward, dir);

    let mut tgt = (target * fa.abs()).max(TARGET_FLOOR);
    let speed = sim.speed();
    let radius = (speed * ARRIVAL_SPEED_FACTOR).max(ARRIVAL_MIN_RADIUS);
    if len(d) <= radius {
        tgt = 0.0;
    }

    let mut steer_in = lat;
    if fa <= FULL_LOCK_DOT {
        steer_in = if lat < 0.0 { -1.0 } else { 1.0 };
    }
    let steer = clamp_unit(steer_in);

    if forced_stop {
        return DriveCommand { steer, throttle: 0.0, brake: 0.0, handbrake: true };
    }
    let span = (tgt - speed).abs().min(THROTTLE_RAMP_SPAN);
    let ramp = (span / THROTTLE_RAMP_SPAN).max(THROTTLE_FLOOR);
    if speed < tgt {
        return DriveCommand { steer, throttle: ramp, brake: 0.0, handbrake: false };
    }
    if !(0.0 < va) {
        return DriveCommand { steer, throttle: ramp, brake: 0.0, handbrake: false };
    }
    if span <= BRAKE_HOLD_SPAN && tgt >= BRAKE_HOLD_TARGET {
        return DriveCommand { steer, throttle: 0.0, brake: 0.0, handbrake: false };
    }
    DriveCommand { steer, throttle: 0.0, brake: ramp, handbrake: false }
}

/// `FUN_00130508`'s gated stop [M 0x130508:74-176]: when the alive gates or the stop requests
/// hold, steer/throttle/brake/handbrake go to 0, then a car still moving at ≥ 0.1 gets the creep
/// pedal `min(|v|, 0.25)` — throttle if it rolls backwards (`dot(normalize(v), forward) < 0`),
/// else brake. The `vtbl+0x1d4 → +0xc0` and `vtbl+0x17c` pre-checks are the actor layer; the port
/// takes them as clear [S].
pub fn gated_stop(sim: &SimCar) -> DriveCommand {
    let speed = sim.speed();
    if speed < GATED_MIN_SPEED {
        return DriveCommand::IDLE;
    }
    let pedal = speed.min(GATED_PEDAL_MAX);
    if dot(normalize(sim.vel), sim.forward) < 0.0 {
        DriveCommand { steer: 0.0, throttle: pedal, brake: 0.0, handbrake: false }
    } else {
        DriveCommand { steer: 0.0, throttle: 0.0, brake: pedal, handbrake: false }
    }
}

/// The inputs `FUN_0012e808` reads for the profiles the port carries.
#[derive(Debug, Clone, Copy)]
pub struct SpeedInputs {
    pub state: u32,
    /// AI +0x84.
    pub profile: u32,
    /// FUN_00195a30(actor).
    pub alive: f32,
    /// The actor's vtbl+0x144 stop query ≠ 0.
    pub stop_query: bool,
    /// AI +0x170 ≠ 0.
    pub stop_byte: bool,
    /// The current segment's rs+0x4c.
    pub seg_limit: f32,
    /// AI +0x134 (1.0 at construction [M 0x12ded8]).
    pub speed_mul: f32,
    /// AI +0x158 (profile 0xf's follow-a-target arm).
    pub follow_target: bool,
}

/// `FUN_0012e808` 0x12e808 → AI +0x8c [M]. The gates: state 5 needs alive > 0, state 4 alive ≥ 0.5,
/// then the stop query and the +0x170 byte; any gate → 0 [:12-52]. Profile 0xc → 7.5; profile
/// 0xf → `rs+0x4c · +0x134` when +0x158 is clear; any profile outside {1, 0xc, 0xd, 0xe, 0xf} → 0.
/// `None` = a profile arm the port does not carry (1, 0xd, 0xe and 0xf with +0x158 — the chase
/// arms that read the target's motion, docs/systems/driving-ai.md §9).
pub fn target_speed(i: &SpeedInputs) -> Option<f32> {
    if i.state == STATE_CHASE && i.alive <= 0.0 {
        return Some(0.0);
    }
    if i.state == STATE_TRAFFIC && i.alive < 0.5 {
        return Some(0.0);
    }
    if i.stop_query || i.stop_byte {
        return Some(0.0);
    }
    match i.profile {
        PROFILE_TRAFFIC => Some(TRAFFIC_SPEED),
        PROFILE_ROAD_LIMIT if !i.follow_target => Some(i.seg_limit * i.speed_mul),
        0x1 | 0xd | 0xe | PROFILE_ROAD_LIMIT => None,
        _ => Some(0.0),
    }
}

/// `FUN_00132f80` 0x132f80: the carrot distance `max(|v|·0.35, box_z + 7.5)`, box_z = the sim's
/// +0x4c8 [M 0x132f80:56-58].
pub fn lookahead(speed: f32, box_z: f32) -> f32 {
    let pad = box_z + LOOKAHEAD_PAD;
    let by_speed = speed * LOOKAHEAD_SPEED_FACTOR;
    if by_speed >= pad {
        by_speed
    } else {
        pad
    }
}

/// `FUN_0013c5e8` 0x13c5e8 — the spawn lane and direction for segment `seg` [M]:
/// `n = lanes− + lanes+`; `k = (route occupancy rr+0x2e + count) % n` (count alone when the
/// segment has no route, rs+0x3c = −1); `c = k + (k ≥ lanes+)`; `lane = lanes+ − c`; lane slot k
/// drives A→B when bit k of rs+0x58 is set, else the lane is negated and the car drives B→A.
/// `count` = DAT_00327d64, the spawn counter `FUN_0013ca60` bumps per spawn. `None` for a
/// segment with no lanes (the game divides by zero there). The random start t the game also draws
/// here is the caller's.
pub fn spawn_lane(net: &RoadNet, seg: usize, count: u32) -> Result<Option<(i8, bool)>, String> {
    let s = net.seg(seg)?;
    let n = s.lanes_neg() as i32 + s.lanes_pos() as i32;
    if n <= 0 {
        return Ok(None);
    }
    let route = i16::from_le_bytes([s.data[0x3c], s.data[0x3d]]);
    let occ = if route == -1 {
        0
    } else {
        net.routes.get(route as usize).map(|r| r.occupancy().0 as u32).unwrap_or(0)
    };
    let k = ((occ + count) % n as u32) as i32;
    let mut c = k as i8;
    if s.lanes_pos() as i32 <= k {
        c += 1;
    }
    let lane = s.lanes_pos() - c;
    let bit = (s.lane_dir_mask() as u32 >> (k & 0x1f)) & 1;
    if bit != 0 {
        Ok(Some((lane, true)))
    } else {
        Ok(Some((-lane, false)))
    }
}

// ------------------------------------------------------------------------------------------
// The road navigator (WRoadNav, kind 1)
// ------------------------------------------------------------------------------------------

/// The navigator state the traffic path uses — WRoadNav at AI +0x54 [M offsets in each field].
#[derive(Debug, Clone)]
pub struct RoadNav {
    /// +0x00: kind (1 = traffic) [M 0x12e468].
    pub kind: u32,
    /// +0x20 (and +0x10): the carrot point on the road [M 0x224e00:176-206].
    pub pos: [f32; 3],
    /// +0x30: the travel direction (rs+0x40, negated toward A) [M 0x2257c0:62-80, 0x224e00:151-159].
    pub dir: [f32; 3],
    /// +0x3c: 1 = toward node B (the exit node is rs node[end]) [M 0x2257c0:65,77; 0x2235c8:92].
    pub end: u8,
    /// +0x3e: the current segment [M 0x2257c0:59].
    pub seg: usize,
    /// +0x40: progress from P0 to P1 [M 0x224e00:120-187].
    pub t: f32,
    /// +0x50 / +0x60: the lane-offset start and end points [M 0x2267d8].
    pub p0: [f32; 3],
    pub p1: [f32; 3],
    /// +0x95: the lane [M 0x2257c0:131, 0x223f98's tail].
    pub lane: i8,
    /// +0x98: the lane offset (the +0x9c/+0xa0 blend pair equals it outside a lane change)
    /// [M 0x2257c0:142-148, 0x227058].
    pub lane_off: f32,
    /// +0x94: the chooser returned the same segment (dead end / no pick) [M 0x224e00:145-147].
    pub repeated: bool,
    /// The track's lane table set [M 0x220c28:21-61].
    pub lane_set: LaneSet,
    /// +0x70 / +0x80: the Bezier control points of a curved piece at this lane offset — P1 beside
    /// P0 and P2 beside P1 (`FUN_00226788` → `FUN_00226538`) [M 0x224e00, 0x2257c0, 0x226f18].
    pub c1: [f32; 3],
    pub c2: [f32; 3],
    /// +0x9c / +0xa0 / +0xa4 / +0xa8 / +0xac: a lane change in progress — the offset it started
    /// from, the offset it goes to, the blend fraction, the distance done and the blend length
    /// (`FUN_00226f18` starts it, `FUN_00227058` steps it; idle while +0x98 == +0xa0) [M].
    pub lane_from: f32,
    pub lane_to: f32,
    pub blend_frac: f32,
    pub blend_done: f32,
    pub blend_len: f32,
    /// +0xb0: a junction link slot to take next (−1 none; kinds 3/4/5) [M 0x223a30, 0x2235c8].
    pub slot: i8,
    /// +0xb1 / +0xb2: steer the junction pick by the hint direction (kind 4) and where the carrot
    /// takes the hint from (0x130140: +0xb1 toward the target, +0xb2 away) [M 0x2235c8, 0x130140].
    pub hint_toward: bool,
    pub hint_away: bool,
}

/// The road the chase choosers steer by: the direction hint and the target piece the carrot
/// passes into `FUN_00224e00` (its a1 / a2, read in the disassembly 0x224e2c / 0x224e54) [M].
#[derive(Debug, Clone, Copy)]
pub struct NavHint {
    pub dir: [f32; 3],
    /// AI +0x1fc: the target's piece, −1 for none [M 0x130140:237].
    pub target: i16,
}

impl NavHint {
    pub const NONE: NavHint = NavHint { dir: [0.0; 3], target: -1 };
}

/// `FUN_00226538` 0x226538 — one lane control point of a curved piece. `start` (param_4 = 1)
/// gives P1 (beside P0), else P2 (beside P1). Toward B: P1 from rs+0x00 / node A, P2 from rs+0x10
/// / node B; toward A: P1 from rs+0x10 / node B, P2 from rs+0x00 / node A. With `ctrl` and
/// `node` those, `d = |ctrl − node|`, `u = normalize(ctrl − node)`,
/// `k = max(rs+0x2c / max(|P0 − P1|, 0.01), 0.01)`: the point is `end_point + u·(d / k)` where
/// end_point is P0 (start) or P1. A straight piece leaves the point unchanged [M].
fn lane_ctrl_point(net: &RoadNet, nav: &RoadNav, seg: usize, start: bool, keep: [f32; 3]) -> Result<[f32; 3], String> {
    let s = net.seg(seg)?;
    if s.flags() & super::road::RSF_CURVED == 0 {
        return Ok(keep);
    }
    let (ctrl, node) = match (nav.end, start) {
        (0, false) => (s.curve_ctrl_a(), s.node_a()),
        (0, true) => (s.curve_ctrl(), s.node_b()),
        (_, true) => (s.curve_ctrl_a(), s.node_a()),
        (_, false) => (s.curve_ctrl(), s.node_b()),
    };
    let n = net.node(node as usize)?.pos();
    let chord = len(sub(nav.p0, nav.p1));
    let k = s.chord_len() / if chord < 0.01 { 0.01 } else { chord };
    let k = if k < 0.01 { 0.01 } else { k };
    let d = len(sub(n, ctrl));
    let u = normalize(sub(ctrl, n));
    let base = if start { nav.p0 } else { nav.p1 };
    let f = d / k;
    Ok([base[0] + u[0] * f, base[1] + u[1] * f, base[2] + u[2] * f])
}

/// `FUN_002267d8` 0x2267d8: one lane-offset end point. `start` = P0 (param_5 = 1). Toward B
/// (end 1): P0 = A + side_a·off, P1 = B + side_b·off; toward A: P0 = B − side_b·off,
/// P1 = A − side_a·off [M 0x2267d8:17-56].
fn lane_point(net: &RoadNet, seg: usize, end: u8, start: bool, off: f32) -> Result<[f32; 3], String> {
    let s = net.seg(seg)?;
    let (node, side, sign) = match (end, start) {
        (1, true) => (s.node_a(), s.side_a(), 1.0),
        (1, false) => (s.node_b(), s.side_b(), 1.0),
        (_, true) => (s.node_b(), s.side_b(), -1.0),
        (_, false) => (s.node_a(), s.side_a(), -1.0),
    };
    let n = net.node(node as usize)?.pos();
    Ok([n[0] + side[0] * off * sign, n[1] + side[1] * off * sign, n[2] + side[2] * off * sign])
}

/// `FUN_002270e8` 0x2270e8: the first link of `node` other than `exclude` whose flag bit 0 is
/// clear (a plain road piece, not a junction connector) [M].
fn first_plain_link(net: &RoadNet, node: u16, exclude: Option<u16>) -> Result<Option<usize>, String> {
    let n = net.node(node as usize)?;
    for i in 0..n.link_count() {
        let Some(l) = n.link(i) else { break };
        let s = net.seg(l as usize)?;
        if Some(s.own_id()) != exclude && s.flags() & 1 == 0 {
            return Ok(Some(l as usize));
        }
    }
    Ok(None)
}

/// `FUN_00222300` 0x222300: the node of `seg` that is not `node` [M].
fn other_node(net: &RoadNet, seg: usize, node: u16) -> Result<u16, String> {
    let s = net.seg(seg)?;
    Ok(if s.node_a() == node { s.node_b() } else { s.node_a() })
}

/// The outcome of the traffic next-segment pick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NextPick {
    pub seg: usize,
    pub end: u8,
    pub lane: i8,
    pub lane_off: f32,
    /// *param_4: the lane-change flag (P0 keeps the old P1, P1 takes the new offset).
    pub lane_change: bool,
}

/// The far-junction lane rule (link slot 2 of 3, or 3 of 4) [M 0x223f98:214-262, the
/// `DAT_003c827c != 0` arm]. `None` = skip the candidate.
fn lane_rule_far(bc: bool, cf: u16, r1000: bool, lane: i8, cur_pos: i8, r_pos: i8) -> Option<i8> {
    if bc || !(cf != 0x20 || r1000) {
        return Some(lane);
    }
    let i13: i8 = if cf == 0x1020 && !r1000 {
        1
    } else {
        if cf != 0x20 || !r1000 {
            if lane == 0 {
                return Some(r_pos);
            }
            if lane != cur_pos {
                return None;
            }
            return Some(r_pos);
        }
        -1
    };
    if lane == 0 {
        return Some(i13);
    }
    if lane == cur_pos {
        return Some(r_pos);
    }
    Some(i13)
}

/// The near-junction lane rule (link slot 1) [M 0x223f98:263-286].
fn lane_rule_near(bc: bool, cf: u16, r1000: bool, lane: i8) -> Option<i8> {
    if bc {
        return Some(lane);
    }
    if cf == 0x20 {
        if !r1000 {
            Some(lane)
        } else {
            None
        }
    } else if lane == 1 {
        Some(1)
    } else {
        None
    }
}

fn take(rng: &mut dyn FnMut() -> u16) -> bool {
    let r = rng() as f32 * 1.525_878_9e-5; // (float)r · (1/65536) [M]
    (r as f64) < JUNCTION_TAKE_CHANCE
}

/// `FUN_00223f98` 0x223f98 (2,072 B) — the next segment for a kind-1 (traffic) navigator at the
/// exit node rs node[end] [M]:
/// - fewer than 2 links: stay on the segment and turn around (end flips);
/// - a plain link exists (`FUN_002270e8`, flag bit 0 clear, not the current one): take it, end =
///   (exit is its node A);
/// - else (a junction: every other link is a connector, bit 0 set): walk the links; a candidate
///   needs flag 0x2 clear, the plain road past its far node with 0x2 clear and a lane slot in
///   rs+0x58 that matches the entry parity (A/B entry, flipped by 0x1000); the slot rules pick
///   which links may be taken with which lane (slot 2 of 4 or any of 2 straight, slot 1 near,
///   slot 2 of 3 / 3 of 4 far, the 0x1020 flag lane rules); each eligible candidate is taken with
///   `rand/65536 < 0.3`, else the walk goes on and the last eligible one stands;
/// - the new lane offset is `±table[class][|lane|]` of the picked segment (of the plain road past
///   it when the pick is a connector); a changed offset on a new segment raises the lane-change
///   flag.
/// The DAT_003c827c = 0 arms are dead (its only write sets 1 [M 0x220c28:63]). A junction road
/// lookup that finds no plain link reads address 0x50 in the game; the port skips the
/// candidate [S].
pub fn next_segment_traffic(net: &RoadNet, nav: &RoadNav, rng: &mut dyn FnMut() -> u16) -> Result<NextPick, String> {
    let cur_idx = nav.seg;
    let cur = net.seg(cur_idx)?;
    let cur_id = cur.own_id();
    let mut lane = nav.lane;
    let mut pick = cur_idx;
    let mut end = nav.end;
    let mut flag = false;
    let exit_idx = if nav.end == 0 { cur.node_a() } else { cur.node_b() };
    let exit = net.node(exit_idx as usize)?;
    let count = exit.link_count();
    let plain = first_plain_link(net, exit_idx, Some(cur_id))?;
    let link = |i: usize| -> Result<usize, String> {
        exit.link(i).map(|l| l as usize).ok_or_else(|| format!("node {exit_idx} link {i} missing"))
    };

    let mut bc = false; // uStack_bc
    if count == 3 {
        let mut n = 0usize;
        for i in 0..count {
            let l = link(i)?;
            if net.seg(l)?.own_id() != cur_id {
                let far = other_node(net, l, exit_idx)?;
                if let Some(r) = first_plain_link(net, far, None)? {
                    if net.seg(r)?.flags() & 2 != 0 {
                        n += 1;
                    }
                }
            }
        }
        bc = n == count - 2;
    }

    if count < 2 {
        pick = cur_id as usize;
        end = (end == 0) as u8;
    } else if let Some(p) = plain {
        pick = p;
        end = (exit_idx == net.seg(p)?.node_a()) as u8;
    } else {
        let cf = cur.flags() & 0x1020;
        for i in 0..count {
            let l = link(i)?;
            if l == cur_idx {
                continue;
            }
            let c = net.seg(l)?;
            if c.flags() & 2 != 0 {
                continue;
            }
            let far = if exit_idx == c.node_a() { c.node_b() } else { c.node_a() };
            let Some(ri) = first_plain_link(net, far, None)? else { continue };
            let r = net.seg(ri)?;
            if r.flags() & 2 != 0 {
                continue;
            }
            let r1000 = r.flags() & 0x1000 != 0;
            let mut par: u32 = if far == r.node_a() { 1 } else { 0 };
            if r1000 {
                par = (par == 0) as u32;
            }
            let n = r.lanes_neg() as i32 + r.lanes_pos() as i32;
            let mut open = false;
            for k in 0..n.max(0) {
                if par == (r.lane_dir_mask() as u32 & (1u32 << (k as u32 & 0x1f))) {
                    open = true;
                }
            }
            if !open {
                continue;
            }
            if (count == 4 && i == 2) || count == 2 {
                pick = l;
                lane = nav.lane;
                end = (exit_idx == c.node_a()) as u8;
                flag = true;
                if take(rng) {
                    break;
                }
            }
            let rule = if (count == 3 && i == 2) || (count == 4 && i == 3) {
                lane_rule_far(bc, cf, r1000, nav.lane, cur.lanes_pos(), r.lanes_pos())
            } else if i == 1 {
                lane_rule_near(bc, cf, r1000, nav.lane)
            } else {
                None
            };
            let Some(nl) = rule else { continue };
            lane = nl;
            pick = l;
            end = (exit_idx == c.node_a()) as u8;
            flag = true;
            if take(rng) {
                break;
            }
        }
    }

    let new = net.seg(pick)?;
    let class = if new.flags() & 1 == 0 {
        new.road_class()
    } else {
        let far = other_node(net, pick, exit_idx)?;
        match first_plain_link(net, far, None)? {
            Some(r) => net.seg(r)?.road_class(),
            None => 0, // [S] the game reads the null record
        }
    };
    let off = lane_offset(nav.lane_set, class, lane);
    if off != nav.lane_off && pick != cur_idx {
        flag = true;
    }
    Ok(NextPick { seg: pick, end, lane, lane_off: off, lane_change: flag })
}

impl RoadNav {
    /// `FUN_002257c0` 0x2257c0 for a non-pedestrian navigator (kind ≠ 2): join `seg` at the
    /// A→B parameter `t`, driving A→B when `forward` (or kind 4), else B→A with the travel-space
    /// `t' = |1 − t|`; lane offset `±table[class][|lane|]` into +0x98/+0x9c/+0xa0; P0/P1 through
    /// FUN_00226a28, the curve controls through FUN_00226788; the carrot is `lerp(P0, P1, t')` on
    /// a straight piece and `bezier([P0, c1, c2, P1], t')` on a curved one (FUN_001a8380 +
    /// FUN_001a85d0) [M 0x2257c0:58-180].
    pub fn join(net: &RoadNet, seg: usize, t: f32, lane: i8, forward: bool, kind: u32, lane_set: LaneSet) -> Result<RoadNav, String> {
        let s = net.seg(seg)?;
        let d = s.dir();
        let (end, dir, tt) = if kind == 4 || forward {
            (1u8, d, t)
        } else {
            (0u8, [-d[0], -d[1], -d[2]], (1.0 - t).abs())
        };
        let off = lane_offset(lane_set, s.road_class(), lane);
        let mut nav = RoadNav {
            kind,
            pos: [0.0; 3],
            dir,
            end,
            seg,
            t: tt,
            p0: [0.0; 3],
            p1: [0.0; 3],
            lane,
            lane_off: off,
            repeated: false,
            lane_set,
            c1: [0.0; 3],
            c2: [0.0; 3],
            lane_from: off,
            lane_to: off,
            blend_frac: 0.0,
            blend_done: 0.0,
            blend_len: 0.0,
            slot: -1,
            hint_toward: false,
            hint_away: false,
        };
        nav.rebuild_points(net, off)?;
        nav.pos = nav.eval(net)?;
        Ok(nav)
    }

    /// `FUN_00226a28` (P0/P1 at offset `off`) then `FUN_00226788` (the curve controls) [M].
    fn rebuild_points(&mut self, net: &RoadNet, off: f32) -> Result<(), String> {
        self.p0 = lane_point(net, self.seg, self.end, true, off)?;
        self.p1 = lane_point(net, self.seg, self.end, false, off)?;
        self.rebuild_ctrl(net)
    }

    /// `FUN_00226788` 0x226788: c1 = FUN_00226538(…, 1), c2 = FUN_00226538(…, 0) [M].
    fn rebuild_ctrl(&mut self, net: &RoadNet) -> Result<(), String> {
        self.c1 = lane_ctrl_point(net, self, self.seg, true, self.c1)?;
        self.c2 = lane_ctrl_point(net, self, self.seg, false, self.c2)?;
        Ok(())
    }

    /// The carrot at the current t: FUN_00222468's straight lerp, or the curve object's Bezier on
    /// a curved piece [M 0x224e00 tail].
    pub fn eval(&self, net: &RoadNet) -> Result<[f32; 3], String> {
        if net.seg(self.seg)?.flags() & super::road::RSF_CURVED != 0 {
            Ok(super::road::bezier([self.p0, self.c1, self.c2, self.p1], self.t))
        } else {
            Ok(lerp(self.p0, self.p1, self.t))
        }
    }

    /// `FUN_00227058` 0x227058 — one step of a lane change: idle (false) while +0x98 == +0xa0;
    /// with a blend length the distance accumulates into +0xa8, `frac = done / len`, and +0x98
    /// moves from +0x9c toward +0xa0 (landing on it at frac ≥ 1, +0x9c too); with no length it
    /// jumps to +0xa0 [M].
    pub fn blend_step(&mut self, dist: f32) -> bool {
        let to = self.lane_to;
        if self.lane_off == to {
            return false;
        }
        if 0.0 < self.blend_len {
            let done = self.blend_done + dist;
            let f = done / self.blend_len;
            self.blend_done = done;
            self.blend_frac = f;
            if 1.0 <= f {
                self.lane_from = to;
                self.lane_off = to;
            } else {
                self.lane_off = self.lane_from + (to - self.lane_from) * f;
            }
        } else {
            self.lane_off = to;
        }
        true
    }

    /// `FUN_00226f18` 0x226f18 — start a lane change to offset `to` over `length` units: with a
    /// length > 0 the blend restarts from the old target (+0x9c = +0x98 = old +0xa0, done and
    /// frac 0); otherwise, when the offset differs, the offset jumps at once and P0/P1, the curve
    /// controls and the carrot are rebuilt [M].
    pub fn start_lane_change(&mut self, net: &RoadNet, to: f32, length: f32) -> Result<(), String> {
        if 0.0 < length {
            let old = self.lane_to;
            self.blend_done = 0.0;
            self.lane_from = old;
            self.lane_to = to;
            self.blend_len = length;
            self.lane_off = old;
            self.blend_frac = 0.0;
        } else if self.lane_off != to {
            self.lane_from = to;
            self.lane_off = to;
            self.lane_to = to;
            self.rebuild_points(net, to)?;
            self.pos = self.eval(net)?;
        }
        Ok(())
    }

    /// `FUN_00224e00` 0x224e00 — move the carrot `dist` along the road [M]: `t += dist/|P0−P1|`;
    /// past 1 the lane blend takes the rest of the piece, the chooser for the navigator's kind
    /// picks the next piece (1 → `next_segment_traffic`, 3/5 → `next_segment_chase`, 4 →
    /// `next_segment_follow`), the direction is re-read (negated toward A), P0/P1 are rebuilt
    /// (at +0x98, or on a lane change P0 = old P1 and P1 at the new offset, which +0x98/+0x9c/
    /// +0xa0 all take), the curve controls follow, t restarts at 0 and the leftover distance
    /// `(dt − (1 − t))·len` carries into the new piece (the game recurses); else t is clamped to
    /// [0, 1], a running lane blend rebuilds the points, and the carrot is the straight lerp or
    /// the Bezier [M 0x224e00:91-207]. A zero-length piece crosses with nothing left over (the
    /// game's `dist/0` → the PS2 FPU's max float) [S]. Kind 2 (0x2247b0) is not ported.
    pub fn advance(&mut self, net: &RoadNet, dist: f32, rng: &mut dyn FnMut() -> u16) -> Result<(), String> {
        self.advance_with(net, dist, NavHint::NONE, rng)
    }

    pub fn advance_with(&mut self, net: &RoadNet, dist: f32, hint: NavHint, rng: &mut dyn FnMut() -> u16) -> Result<(), String> {
        if !matches!(self.kind, 1 | 3 | 4 | 5) {
            return Err(format!("nav kind {} chooser not ported (kinds 1, 3, 4, 5)", self.kind));
        }
        let mut dist = dist;
        let mut hint = hint;
        for _ in 0..256 {
            let l = len(sub(self.p0, self.p1));
            let dt = if l > 0.0 { dist / l } else if dist > 0.0 { f32::INFINITY } else { 0.0 };
            let t1 = self.t + dt;
            if t1 >= 1.0 {
                let rest = if l > 0.0 { (dt - (1.0 - self.t)) * l } else { 0.0 };
                self.blend_step((1.0 - self.t) * l);
                let pick = match self.kind {
                    NAV_KIND_TRAFFIC => next_segment_traffic(net, self, rng)?,
                    4 => {
                        let seg = next_segment_follow(net, self, &mut hint.dir, rng)?;
                        NextPick { seg, end: self.end, lane: self.lane, lane_off: self.lane_off, lane_change: false }
                    }
                    _ => {
                        let (seg, end) = next_segment_chase(net, self, &mut hint.dir, hint.target)?;
                        NextPick { seg, end, lane: self.lane, lane_off: self.lane_off, lane_change: false }
                    }
                };
                if pick.seg == self.seg {
                    self.repeated = true;
                }
                self.seg = pick.seg;
                self.end = pick.end;
                self.lane = pick.lane;
                let d = net.seg(pick.seg)?.dir();
                self.dir = if pick.end == 0 { [-d[0], -d[1], -d[2]] } else { d };
                if !pick.lane_change {
                    let off = self.lane_off;
                    self.p0 = lane_point(net, pick.seg, pick.end, true, off)?;
                    self.p1 = lane_point(net, pick.seg, pick.end, false, off)?;
                } else {
                    self.p0 = self.p1;
                    self.p1 = lane_point(net, pick.seg, pick.end, false, pick.lane_off)?;
                    self.lane_off = pick.lane_off;
                    self.lane_from = pick.lane_off;
                    self.lane_to = pick.lane_off;
                }
                self.rebuild_ctrl(net)?;
                self.pos = self.p0;
                self.t = 0.0;
                dist = rest;
                continue;
            }
            self.t = t1.max(0.0).min(1.0);
            if self.t.is_nan() {
                self.t = 0.0;
            }
            if self.blend_step(dist) {
                let off = self.lane_off;
                self.rebuild_points(net, off)?;
            }
            self.pos = self.eval(net)?;
            return Ok(());
        }
        Err("road advance did not settle in 256 pieces".into())
    }
}

/// `FUN_00223a30` 0x223a30 (1,380 B) — the next piece for a kind-3/5 navigator, the road chasers'
/// chooser (pattern 3 off state 4 joins kind 3, `FUN_0012e468`) [M]:
/// - a pending link slot (+0xb0 ≥ 0) at a node with more than 2 links: take that link, clear the
///   slot; the end byte is left as it was;
/// - fewer than 2 links: stay, turn round;
/// - a plain link (flag bit 0 clear, not the current piece): take it;
/// - a junction with a target piece (≥ 0) and a junction record at the exit end (rs+0x1c+end·2):
///   for each neighbour junction other than the one behind, that is an end of the target piece,
///   the first road of the exit junction that also ends at that neighbour is reached through the
///   exit link whose far node lists it (the last such link wins);
/// - otherwise the connector whose road past it points best along the hint (`dot(normalize(road
///   dir away from the junction), normalize(hint))`, first best above −2);
/// The end byte follows the pick (1 = the exit node is the pick's node A). A connector with no
/// plain road past its far node makes the game read a null record; the port skips it [S].
pub fn next_segment_chase(net: &RoadNet, nav: &mut RoadNav, hint: &mut [f32; 3], target: i16) -> Result<(usize, u8), String> {
    let cur = net.seg(nav.seg)?;
    let cur_id = cur.own_id();
    let exit_idx = if nav.end == 0 { cur.node_a() } else { cur.node_b() };
    let exit = net.node(exit_idx as usize)?;
    let count = exit.link_count();
    let link = |i: usize| -> Result<usize, String> {
        exit.link(i).map(|l| l as usize).ok_or_else(|| format!("node {exit_idx} link {i} missing"))
    };
    let end_of = |seg: usize| -> Result<u8, String> { Ok((exit_idx == net.seg(seg)?.node_a()) as u8) };
    let plain = first_plain_link(net, exit_idx, Some(cur_id))?;
    if nav.slot >= 0 && count > 2 {
        let s = link(nav.slot as usize)?;
        nav.slot = -1;
        return Ok((s, nav.end));
    }
    if count < 2 {
        return Ok((cur_id as usize, (nav.end == 0) as u8));
    }
    if let Some(p) = plain {
        return Ok((p, end_of(p)?));
    }
    let mut pick = nav.seg;
    let mut end = nav.end;
    if target >= 0 {
        let jn = cur.junction_at(nav.end);
        let behind = cur.junction_at(nav.end ^ 1);
        let tgt = net.seg(target as usize)?;
        if jn >= 0 {
            let j = net.junction(jn as usize)?;
            for nb in j.neighbours() {
                if nb == behind {
                    continue;
                }
                if tgt.junction_at(0) != nb && tgt.junction_at(1) != nb {
                    continue;
                }
                for r in j.roads() {
                    let road = net.seg(r as usize)?;
                    if road.junction_at(0) != nb && road.junction_at(1) != nb {
                        continue;
                    }
                    for k in 0..count {
                        let l = link(k)?;
                        let far = other_node(net, l, exit_idx)?;
                        let fnode = net.node(far as usize)?;
                        for u in 0..fnode.link_count() {
                            if fnode.link(u).map(|x| x == road.own_id()).unwrap_or(false) {
                                pick = net.seg(l)?.own_id() as usize;
                            }
                        }
                    }
                    return Ok((pick, end_of(pick)?));
                }
            }
        }
    }
    *hint = normalize(*hint);
    let mut best = -2.0f32;
    for k in 0..count {
        let l = link(k)?;
        if l == nav.seg {
            continue;
        }
        let c = net.seg(l)?;
        if c.flags() & 1 == 0 {
            return Ok((l, end_of(l)?));
        }
        let far = other_node(net, l, exit_idx)?;
        let Some(ri) = first_plain_link(net, far, None)? else { continue };
        let r = net.seg(ri)?;
        let mut d = r.dir();
        if far != r.node_a() {
            d = [-d[0], -d[1], -d[2]];
        }
        let dd = dot(normalize(d), *hint);
        if best < dd {
            pick = l;
            end = end_of(l)?;
            best = dd;
        }
    }
    Ok((pick, end))
}

/// `FUN_002235c8` 0x2235c8 (1,128 B) — the next piece for a kind-4 navigator (pattern 4, the
/// road followers; it joins toward B and keeps that orientation) [M]:
/// - a pending link slot at a node with 3+ links: that link (link 0 when the slot is past the
///   list), slot cleared;
/// - fewer than 2 links: stay, turn round;
/// - no hint flag (+0xb1/+0xb2 clear), 2 links: the first link other than the current whose node
///   at the travel end differs from the exit node (else the last non-current link);
/// - no hint flag, 3+ links: one draw; of the n links that pass the same test the
///   `(n·rand) >> 16`-th is taken (the last passing one, or the current piece, if the count runs
///   out);
/// - a hint flag: the link whose direction away from the exit node (rs+0x40, negated when its
///   node B is the exit) best matches the hint; the end byte follows that pick.
pub fn next_segment_follow(net: &RoadNet, nav: &mut RoadNav, hint: &mut [f32; 3], rng: &mut dyn FnMut() -> u16) -> Result<usize, String> {
    let cur = net.seg(nav.seg)?;
    let cur_id = cur.own_id();
    let exit_idx = if nav.end == 0 { cur.node_a() } else { cur.node_b() };
    let exit = net.node(exit_idx as usize)?;
    let count = exit.link_count();
    let link = |i: usize| -> Result<usize, String> {
        exit.link(i).map(|l| l as usize).ok_or_else(|| format!("node {exit_idx} link {i} missing"))
    };
    let node_at = |s: &super::road::RoadSeg, e: u8| if e == 0 { s.node_a() } else { s.node_b() };
    if nav.slot >= 0 && count >= 3 {
        let i = if (nav.slot as usize) < count { nav.slot as usize } else { 0 };
        nav.slot = -1;
        return link(i);
    }
    if count < 2 {
        nav.end = (nav.end == 0) as u8;
        return Ok(cur_id as usize);
    }
    let cur_exit = node_at(&cur, nav.end);
    if !(nav.hint_toward || nav.hint_away) {
        if count < 3 {
            let mut last = None;
            for i in 0..count {
                let l = link(i)?;
                if l == cur_id as usize {
                    continue;
                }
                last = Some(l);
                if cur_exit != node_at(&net.seg(l)?, nav.end) {
                    return Ok(l);
                }
            }
            return Ok(last.unwrap_or(cur_id as usize));
        }
        let mut n = count as u32;
        for i in 0..count {
            let l = link(i)?;
            if l == cur_id as usize || cur_exit == node_at(&net.seg(l)?, nav.end) {
                n -= 1;
            }
        }
        let r = rng() as u32;
        let want = ((n.wrapping_mul(r)) as i32) >> 16;
        let mut k = 0i32;
        let mut last = cur_id as usize;
        for i in 0..count {
            let l = link(i)?;
            if l != cur_id as usize && cur_exit != node_at(&net.seg(l)?, nav.end) {
                if k == want {
                    return Ok(l);
                }
                k += 1;
                last = l;
            }
        }
        return Ok(last);
    }
    *hint = normalize(*hint);
    let exit_id = exit.own_id();
    let mut best = -2.0f32;
    let mut pick = cur_id as usize;
    for i in 0..count {
        let l = link(i)?;
        if l == cur_id as usize {
            continue;
        }
        let c = net.seg(l)?;
        let mut d = c.dir();
        if c.node_b() == exit_id {
            d = [-d[0], -d[1], -d[2]];
        }
        let dd = dot(normalize(d), *hint);
        if best < dd {
            pick = l;
            nav.end = (exit_idx == c.node_a()) as u8;
            best = dd;
        }
    }
    Ok(pick)
}

// ------------------------------------------------------------------------------------------
// The AI car
// ------------------------------------------------------------------------------------------

/// The AIVehicle subset a live traffic car runs.
#[derive(Debug, Clone)]
pub struct AiCar {
    /// +0x70 [M 0x12e350].
    pub state: u32,
    /// +0x84 [M 0x12e808].
    pub profile: u32,
    /// +0x78 [M 0x130508].
    pub pattern: u32,
    /// +0x54: the road navigator.
    pub nav: RoadNav,
    /// +0x8c: the target speed [M 0x12e808].
    pub target_speed: f32,
    /// +0x19c: the forced stop [M 0x133ab0:144].
    pub forced_stop: bool,
    /// +0x170: the stop byte [M 0x12e808:48-52].
    pub force_stop_byte: i8,
    /// The actor's vtbl+0x144 stop query [M 0x12e808:41-45].
    pub stop_query: i32,
    /// +0x134: the speed multiplier (1.0 at construction) [M 0x12ded8].
    pub speed_mul: f32,
    /// sim+0x5c → +0x4c8: the box z extent FUN_00132f80 pads the lookahead with [M 0x1f6e98:162].
    /// The extent's source (FUN_001f6e98's param_10) is not traced; the floor is the default.
    pub box_z: f32,
    /// +0x178: the scripted-path flag; FUN_00133ab0 issues nothing while it is set [M 0x133ab0:31].
    pub scripted: bool,
    /// +0x154: the avoidance enable — the AIVehicle ctor sets it (FUN_0012ded8:58) and the
    /// state-4 drive runs the yield FUN_001377c8 while it is set [M 0x130508:292-296].
    pub avoid: bool,
    /// +0x194 / +0x198: the yield found a car or a red light ahead / the red light alone
    /// (rewritten by every FUN_001377c8 call; +0x198 holds the carrot next frame) [M 0x1377c8].
    pub yielding: bool,
    pub junction_stop: bool,
    /// +0xe4: the stop-and-wait flag the actor's vtbl+0x17c query raises [M 0x132d80].
    pub stop_wait: bool,
    /// +0x90: the route the car is counted on (−1 none) [M 0x132d80, 0x132ba8].
    pub route_seen: i16,
    /// +0x58: the controller enable [M 0x132808:21].
    pub enabled: bool,
    /// +0x28c / +0x280: a respawn spot is waiting / the spot [M 0x132808].
    pub respawn_pending: bool,
    pub respawn_at: [f32; 3],
    /// +0x290: the tick before which an out-of-range car is left alone (now + 5 s on a respawn)
    /// [M 0x132808:132].
    pub respawn_hold: i32,
}

impl AiCar {
    /// A live traffic car (state 4, profile 0xc, pattern 3, nav kind 1 — what FUN_00132808 leaves
    /// a respawned car in [M 0x132808:93-131 + 0x12e468]) joined onto `seg`.
    pub fn traffic(net: &RoadNet, seg: usize, t: f32, lane: i8, forward: bool, lane_set: LaneSet) -> Result<AiCar, String> {
        Ok(AiCar {
            state: STATE_TRAFFIC,
            profile: PROFILE_TRAFFIC,
            pattern: PATTERN_ROAD,
            nav: RoadNav::join(net, seg, t, lane, forward, NAV_KIND_TRAFFIC, lane_set)?,
            target_speed: 0.0,
            forced_stop: false,
            force_stop_byte: 0,
            stop_query: 0,
            speed_mul: 1.0,
            box_z: BOX_FLOOR,
            scripted: false,
            avoid: true,
            yielding: false,
            junction_stop: false,
            stop_wait: false,
            route_seen: -1,
            enabled: true,
            respawn_pending: false,
            respawn_at: [0.0; 3],
            respawn_hold: 0,
        })
    }

    fn gated(&self, sim: &SimCar) -> bool {
        (self.state == STATE_CHASE && sim.alive <= 0.0)
            || (self.state == STATE_TRAFFIC && sim.alive < 0.5)
            || self.stop_query != 0
            || self.force_stop_byte != 0
    }

    /// One frame of the brain for states 2/4/5 → the actor command; `None` when the game issues
    /// none (standby/teleport states, the +0x178 scripted path) [M 0x12e350 → 0x12e808 →
    /// 0x130508 → 0x133ab0]. Pattern 3 in state 4 only; the +0x198 / +0xe4 / +0x154 arms of
    /// FUN_00130508 (scripted stop, stop-and-wait, avoidance FUN_001377c8) are taken as clear [S].
    pub fn step(&mut self, net: &RoadNet, sim: &SimCar, rng: &mut dyn FnMut() -> u16) -> Result<Option<DriveCommand>, String> {
        self.step_with(net, sim, None, rng)
    }

    /// `step` with the traffic world: given `world`, the state-4 drive runs FUN_00130508's
    /// +0x198 carrot hold and the +0x154 yield (`traffic_yield`) [M 0x130508:256-309]. The
    /// actor's vtbl+0x17c query (which, nonzero, releases the hold and raises +0xe4) is taken as 0
    /// [S: its implementation was not read].
    pub fn step_with(&mut self, net: &RoadNet, sim: &SimCar, world: Option<&YieldWorld>, rng: &mut dyn FnMut() -> u16) -> Result<Option<DriveCommand>, String> {
        if !matches!(self.state, STATE_RETARGET | STATE_TRAFFIC | STATE_CHASE) {
            return Ok(None);
        }
        let seg_limit = net.seg(self.nav.seg)?.speed_limit();
        let si = SpeedInputs {
            state: self.state,
            profile: self.profile,
            alive: sim.alive,
            stop_query: self.stop_query != 0,
            stop_byte: self.force_stop_byte != 0,
            seg_limit,
            speed_mul: self.speed_mul,
            follow_target: false,
        };
        self.target_speed = target_speed(&si)
            .ok_or_else(|| format!("profile {:#x} target speed not ported", self.profile))?;

        if self.gated(sim) {
            return Ok(Some(gated_stop(sim)));
        }
        if self.pattern != PATTERN_ROAD || self.state != STATE_TRAFFIC {
            return Err(format!("pattern {} in state {} not ported", self.pattern, self.state));
        }
        // FUN_00132f80 → FUN_00130140: keep the carrot `lookahead` ahead [M 0x130140:106-245];
        // a red-light stop from last frame (+0x198) holds it [M 0x130508:256-262].
        let hold = world.is_some() && self.junction_stop;
        if !hold {
            let la = lookahead(sim.speed(), self.box_z);
            let gap = len(sub(sim.pos, self.nav.pos));
            if gap < la {
                self.nav.advance(net, (la - gap).abs(), rng)?;
            }
        }
        let mut aim = self.nav.pos;
        if let Some(w) = world {
            if self.avoid {
                let (y, j) = traffic_yield(net, &self.nav, sim, self.box_z, w, &mut aim, &mut self.forced_stop)?;
                self.yielding = y;
                self.junction_stop = j;
            }
        }
        // nav+0x94 set → the aim is the car itself (stop) [M 0x130508:297-303].
        if self.nav.repeated {
            aim = sim.pos;
        }
        if self.scripted {
            return Ok(None);
        }
        Ok(Some(drive_control(sim, aim, self.target_speed, self.forced_stop)))
    }
}

/// The turn-restriction flags — kept for the old tests.
pub fn is_turn_restricted(flags: u16) -> bool {
    flags & (RSF_TURN_RESTR_A | RSF_TURN_RESTR_B) != 0
}

/// The AI brain on the port's physics car: the brain's command becomes the `CarInput`
/// (SetSteer/SetThrottle/SetBrake/SetHandbrake 1:1 [M 0x133ab0:135-180]) and `Car::step`
/// integrates it. The car is a free body steered at the carrot, as in the game.
pub struct AiDriver {
    pub brain: AiCar,
    pub car: Car,
    pub world: FlatWorld,
    pub rigid: Rigid,
    /// The driving world's generator the junction picks draw from.
    pub rng: DriveRng,
}

impl AiDriver {
    /// A traffic car spawned on `segment` the way `FUN_0013ca60` → `FUN_0013c5e8` →
    /// `FUN_002257c0` → `FUN_00132808`'s teleport do it [M]: lane and direction from
    /// `spawn_lane(segment, spawn_index)`, the car placed on the carrot (nav +0x20) facing the
    /// travel direction (nav +0x30) [M 0x132808:95-140]. The start t is 0 (the game draws it at
    /// random [M 0x13c5e8:22]); `end` (1 = toward B) is used only on a segment with no lanes.
    pub fn on_segment(params: CarParams, net: &RoadNet, segment: usize, end: u8, spawn_index: i8) -> Result<AiDriver, String> {
        let (lane, forward) = spawn_lane(net, segment, spawn_index as u8 as u32)?.unwrap_or((0, end == 1));
        let brain = AiCar::traffic(net, segment, 0.0, lane, forward, LaneSet::Default)?;
        let yaw = brain.nav.dir[0].atan2(brain.nav.dir[2]);
        let car = Car::new(params, brain.nav.pos, yaw);
        Ok(AiDriver { brain, car, world: FlatWorld::paved(), rigid: Rigid::game_defaults(), rng: DriveRng::game_reset() })
    }

    /// One 60 Hz step: brain (FUN_0012e808 → FUN_00130508 → FUN_00133ab0) → `Car::step`.
    /// Two stand-ins for gaps in the port's `Car::step` [S]: a brake cannot drive the car
    /// backwards through zero (the game's reverse is a separate gear path), and the velocity
    /// follows the heading (the tyre lateral force FUN_00190f88 is not integrated by `Car::step`,
    /// so without it a turning car keeps sliding along its old line).
    pub fn step(&mut self, net: &RoadNet, dt: f32) -> Result<DriveCommand, String> {
        let mut rng = std::mem::replace(&mut self.rng, DriveRng { state: 0, mult: 0, draws: 0 });
        let out = self.step_shared(net, dt, None, &mut rng);
        self.rng = rng;
        out
    }

    /// `step` drawing from a generator shared by the whole world (the game has one, at
    /// *0x339190) and, given `world`, running the yield (`AiCar::step_with`).
    pub fn step_shared(&mut self, net: &RoadNet, dt: f32, world: Option<&YieldWorld>, rng: &mut DriveRng) -> Result<DriveCommand, String> {
        let sim = SimCar::of_car(&self.car);
        let cmd = match self.brain.step_with(net, &sim, world, &mut || rng.next())? {
            Some(c) => c,
            None => return Ok(DriveCommand::IDLE),
        };
        let input = CarInput { throttle: cmd.throttle, brake: cmd.brake, steer: cmd.steer, handbrake: cmd.handbrake };
        let (pre_sin, pre_cos) = self.car.yaw.sin_cos();
        self.car.step(&input, &self.world, &self.rigid, dt);
        if input.throttle == 0.0 && input.brake > 0.0 && self.car.vel[0] * pre_sin + self.car.vel[2] * pre_cos < 0.0 {
            self.car.vel = [0.0, self.car.vel[1], 0.0];
        }
        let (s, c) = self.car.yaw.sin_cos();
        let along = self.car.vel[0] * s + self.car.vel[2] * c;
        self.car.vel[0] = s * along;
        self.car.vel[2] = c * along;
        Ok(cmd)
    }
}

// ------------------------------------------------------------------------------------------
// Junction lights and the traffic yield (FUN_0013ef40 / FUN_0013efc8 / FUN_001377c8)
// ------------------------------------------------------------------------------------------

/// `FUN_0013ef40`'s light period: every 300 world ticks the green road index steps by one
/// (`DAT_003c8290 = now + 300; DAT_003c828c += 1` once `DAT_003c8290 < now`) [M 0x13ef40].
pub const LIGHT_PERIOD: i32 = 300;
/// `FUN_0013efc8`'s all-red tail: no road may go once `deadline − 150 < now` (0x96) [M 0x13efc8].
pub const LIGHT_ALL_RED: i32 = 0x96;
/// `FUN_0013efc8`'s clear check: an object on the `DAT_003339a8` list within 50 units (squared
/// 2500) of the junction and less than 5 below it holds every road [M 0x13efc8].
pub const LIGHT_CLEAR_DIST2: f32 = 2500.0;
pub const LIGHT_CLEAR_RISE: f32 = 5.0;

/// The junction lights: `DAT_003c828c` (the green road counter) and `DAT_003c8290` (the tick the
/// period ends). "Init AI Controllers" `FUN_0013e688` and `FUN_0013e840` reset them to 0 and
/// `now + 300` [M 0x13e688:43-44, 0x13e840:11-13]; the world frame callback `FUN_001472e0`
/// ticks them once per frame through `FUN_0013ef40` [M 0x1472e0:11].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JunctionLights {
    pub phase: i32,
    pub deadline: i32,
}

impl JunctionLights {
    pub fn new(now: i32) -> JunctionLights {
        JunctionLights { phase: 0, deadline: now + LIGHT_PERIOD }
    }

    /// `FUN_0013ef40` [M].
    pub fn tick(&mut self, now: i32) {
        if self.deadline < now {
            self.deadline = now + LIGHT_PERIOD;
            self.phase += 1;
        }
    }

    /// `FUN_0013efc8` 0x13efc8 — may road `idx` of the `n` open roads at `junction` go? Any
    /// object of the `DAT_003339a8` list (`objects`; positions = entry +0x30) with
    /// `junction.y − obj.y < 5` and within 50 holds it; then the all-red tail; then green iff
    /// `idx == phase % n` [M]. n = 0 traps in the game (`trap(7)`); the port answers false.
    pub fn may_go(&self, junction: [f32; 3], idx: i32, n: i32, now: i32, objects: &[[f32; 3]]) -> bool {
        for o in objects {
            if junction[1] - o[1] < LIGHT_CLEAR_RISE {
                let d = sub(junction, *o);
                if dot(d, d) < LIGHT_CLEAR_DIST2 {
                    return false;
                }
            }
        }
        if self.deadline - LIGHT_ALL_RED < now {
            return false;
        }
        if n == 0 {
            return false;
        }
        idx == self.phase % n
    }
}

/// `FUN_001f8908` 0x1f8908: the sim object's grid cell — each coordinate `·0.2`, rounded half
/// away from zero by `trunc(x ± 0.5)` (stored at sim+0x5c → +0x4f0/+0x4f4/+0x4f8) [M].
pub fn cell_of(p: [f32; 3]) -> [i32; 3] {
    let c = |v: f32| {
        let s = v * 0.2;
        (s + if s < 0.0 { -0.5 } else { 0.5 }) as i32
    };
    [c(p[0]), c(p[1]), c(p[2])]
}

/// One entry of the world's vehicle list (`DAT_00339f5c`, the player first) as the traffic code
/// reads it [M 0x1377c8 / 0x13ca60 field reads].
#[derive(Debug, Clone, Copy)]
pub struct Vehicle {
    /// sim +0x10.
    pub pos: [f32; 3],
    /// sim +0x5c → +0x4c0 / +0x4c8: the extents the yield's radius takes the larger of.
    pub box_x: f32,
    pub box_z: f32,
    /// sim +0x6d (< 4 is considered by the yield) and +0x6e (3 = parked, 2 = live traffic).
    pub kind: i8,
    pub status: i8,
    /// The controller's +0x58 enable and +0x70 state (the spawn spacing reads them).
    pub enabled: bool,
    pub state: u32,
    /// FUN_00195a30 of the actor (the spawn spacing's alive ≥ 1 test).
    pub alive: f32,
    /// The navigator's lane (+0x95) and end (+0x3c) (the spawn spacing's same-lane test).
    pub lane: i8,
    pub end: u8,
}

impl Vehicle {
    pub fn cell(&self) -> [i32; 3] {
        cell_of(self.pos)
    }
}

/// What the yield sees of the world.
pub struct YieldWorld<'a> {
    /// `DAT_00339f54`, the world tick.
    pub now: i32,
    pub lights: JunctionLights,
    /// Every vehicle (the player included).
    pub vehicles: &'a [Vehicle],
    /// The index of this car in `vehicles` (the game skips its own sim id), if listed.
    pub me: Option<usize>,
    /// The `DAT_003339a8` objects the lights check (positions). What that list holds was not
    /// traced (FUN_001b1988 builds it; FUN_0013da80 adds timed path objects) [S].
    pub light_objects: &'a [[f32; 3]],
}

/// `FUN_00135530` 0x135530 — does `other` block the drive toward `aim`? It does when it lies in
/// the 0.9 cone of the aim direction (`dot(n(aim − me), n(other − me)) ≥ 0.9`) and either sits
/// within `r` of the aim or nearer to the car than the aim [M].
pub fn path_blocked(r: f32, me: [f32; 3], aim: [f32; 3], other: [f32; 3]) -> bool {
    let u = normalize(sub(aim, me));
    let w = normalize(sub(other, me));
    if dot(u, w) < 0.9 {
        return false;
    }
    if r <= len(sub(other, aim)) {
        let d_other = len(sub(me, other));
        let d_aim = len(sub(me, aim));
        return d_aim > d_other;
    }
    true
}

/// `FUN_00135978` 0x135978 — pull the aim back to stop short of `point`:
/// `aim = me + u·max(dot(u, point − me) − r − 1.5·box_z, 0)`, u = n(aim − me), box_z = this
/// car's sim +0x4c8 [M].
pub fn stop_short(r: f32, me: [f32; 3], box_z: f32, aim: &mut [f32; 3], point: [f32; 3]) {
    let u = normalize(sub(*aim, me));
    let proj = dot(u, sub(point, me));
    let s = proj - r - box_z * 1.5;
    let s = if 0.0 < s { s } else { 0.0 };
    *aim = [me[0] + u[0] * s, me[1] + u[1] * s, me[2] + u[2] * s];
}

/// `FUN_001377c8` 0x1377c8 (1,432 B) — the traffic yield, run on the aim (AI +0x1f0) after the
/// carrot step for a state-4 car with +0x154 [M 0x130508:292-296]. Returns (+0x194, +0x198) and
/// updates `aim` and `forced_stop` (+0x19c):
/// 1. the red light: on a piece with flag 0x80 whose exit node has 3+ links, the exit end's
///    junction (rs+0x1c+end·2) counts its roads without flag 0x2 (n) and this piece's index among
///    them; on a piece without flag 0x20 with n ≥ 3, `JunctionLights::may_go` false → +0x194 =
///    +0x198 = 1, the aim stops short (r = 0) of the piece's other node, and at speed ≤ 1 the
///    forced stop is raised;
/// 2. otherwise the car ahead: every listed vehicle but this one with sim +0x6d < 4, +0x6e ≠ 3,
///    within 4/3/4 grid cells (x/y/z) and `path_blocked(max(box_x, box_z), …)` against the
///    current aim → +0x194 = 1, a working aim (carried from car to car) stops short of it, the
///    forced stop at speed ≤ 1, and the aim takes the working aim nearest the car;
/// 3. +0x194 clear → the forced stop is cleared.
/// The store of +0x154 = 1 onto this car when the blocker is not a traffic car changes nothing
/// (it is already set on this path). A junction lookup with no record (rs+0x1c = −1) reads
/// `DAT_0033a604[-1]` in the game; the port takes no light there [S].
pub fn traffic_yield(net: &RoadNet, nav: &RoadNav, sim: &SimCar, box_z: f32, w: &YieldWorld, aim: &mut [f32; 3], forced_stop: &mut bool) -> Result<(bool, bool), String> {
    let mut yielding = false;
    let mut junction_stop = false;
    let cur = net.seg(nav.seg)?;
    if cur.flags() & 0x80 != 0 {
        let exit_idx = if nav.end == 0 { cur.node_a() } else { cur.node_b() };
        let exit = net.node(exit_idx as usize)?;
        let jn = cur.junction_at(nav.end);
        if exit.link_count() >= 3 && jn >= 0 {
            let entry = other_node(net, nav.seg, exit_idx)?;
            let j = net.junction(jn as usize)?;
            let roads = j.roads();
            let mut open = roads.len() as i32;
            let mut seen = 0i32;
            let mut mine = 0i32;
            for r in &roads {
                if *r as i32 == cur.own_id() as i32 {
                    mine = seen;
                }
                if net.seg(*r as usize)?.flags() & 2 == 0 {
                    seen += 1;
                } else {
                    open -= 1;
                }
            }
            if cur.flags() & 0x20 == 0 && open >= 3 && !w.lights.may_go(j.pos(), mine, open, w.now, w.light_objects) {
                yielding = true;
                junction_stop = true;
                let p = net.node(entry as usize)?.pos();
                stop_short(0.0, sim.pos, box_z, aim, p);
                if sim.speed() <= 1.0 {
                    *forced_stop = true;
                }
            }
        }
    }
    if !yielding {
        let me = cell_of(sim.pos);
        let mut work = *aim;
        let mut best: Option<f32> = None;
        for (i, v) in w.vehicles.iter().enumerate() {
            if Some(i) == w.me {
                continue;
            }
            if !(v.kind < 4 && v.status != 3) {
                continue;
            }
            let c = v.cell();
            if (me[0] - c[0]).abs() >= 4 || (me[1] - c[1]).abs() >= 3 || (me[2] - c[2]).abs() >= 4 {
                continue;
            }
            let r = if v.box_z < v.box_x { v.box_x } else { v.box_z };
            if !path_blocked(r, sim.pos, *aim, v.pos) {
                continue;
            }
            yielding = true;
            stop_short(r, sim.pos, box_z, &mut work, v.pos);
            if sim.speed() <= 1.0 {
                *forced_stop = true;
            }
            let d = len(sub(sim.pos, work));
            if best.is_none_or(|b| d < b) {
                *aim = work;
                best = Some(d);
            }
        }
        if !yielding {
            *forced_stop = false;
        }
    }
    Ok((yielding, junction_stop))
}

// ------------------------------------------------------------------------------------------
// Traffic spawn and respawn (FUN_0013d618 / FUN_0013ca60 / FUN_0013c860 / FUN_0013c798 /
// FUN_00132808 / FUN_00132ba8 / FUN_00132d80)
// ------------------------------------------------------------------------------------------

/// The spawn ring `FUN_0013ca60` and `FUN_0013d3f0` ask `FUN_0013d618` for: 75 (0x42960000,
/// `DAT_00327d68`, the nearest a spawn may be and the keep radius) and 150 (0x43160000,
/// `DAT_00327d6c`, the box half-size and the far radius) [M 0x13ca60:33].
pub const SPAWN_INNER: f32 = 75.0;
pub const SPAWN_OUTER: f32 = 150.0;
/// `FUN_0013d618` rebuilds its list at most every 0x59 ticks, keeping up to 50 pieces [M].
pub const SPAWN_REFRESH: i32 = 0x59;
pub const SPAWN_LIST_MAX: usize = 50;
/// `FUN_0013c860`: a car beyond the keep radius is in view while `dot(n(pos − eye), eye fwd) ≥
/// 0.65` [M].
pub const VIEW_DOT: f32 = 0.65;
/// `FUN_00132808`: a respawned car is left alone for 5 s (`DAT_00339f4c · 5`) [M 0x132808:132].
pub const RESPAWN_HOLD_SECONDS: i32 = 5;

/// The `CDat` collision grid the road code shares (`DAT_0033a5c8`): the `CGrd` header (min x at
/// +0x00, min z at +0x08, 1/cell at +0x14, rows (z) at +0x18, columns (x) at +0x1c) and the
/// `cn  ` cells (record count = the cell index; payload +0x07 the road-piece count, +0x0e its byte
/// offset from +0x10, s16 entries) [M FUN_0021cf80 reads the header, FUN_0013d618 / FUN_00225490
/// the cells; D Paris: 108 × 162 cells of 24 units, 1/24 stored].
#[derive(Debug, Clone, Default)]
pub struct RoadGrid {
    pub min_x: f32,
    pub min_z: f32,
    pub inv_cell: f32,
    pub rows: u32,
    pub cols: u32,
    pub cells: std::collections::HashMap<u32, Vec<i16>>,
}

impl RoadGrid {
    /// Read the grid out of the track `.crp`'s pool `CDat` article.
    pub fn parse_crp(crp: &nf_formats::crp::Crp) -> Result<RoadGrid, String> {
        use nf_formats::crp::Record;
        const CDAT: u32 = 0x4344_6174;
        const CGRD: u32 = 0x4347_7264; // "CGrd"
        const CN: u32 = 0x636e_2020;
        let mut g = RoadGrid::default();
        let mut found = false;
        for rec in crp.pool().map_err(|e| e.to_string())? {
            if rec.tag != CDAT {
                continue;
            }
            let (s, e) = rec.window(crp.data).map_err(|e| e.to_string())?;
            for o in (s..e).step_by(0x10) {
                let r = Record::parse(crp.data, o).map_err(|e| e.to_string())?;
                let code = r.tag_code();
                if code != CGRD && code != CN {
                    continue;
                }
                let p = crp.payload(&r).map_err(|e| e.to_string())?;
                let f = |o: usize| f32::from_le_bytes(p[o..o + 4].try_into().unwrap());
                let u = |o: usize| u32::from_le_bytes(p[o..o + 4].try_into().unwrap());
                if code == CGRD {
                    if p.len() < 0x20 {
                        return Err("CGrd payload too small".into());
                    }
                    g.min_x = f(0x00);
                    g.min_z = f(0x08);
                    g.inv_cell = f(0x14);
                    g.rows = u(0x18);
                    g.cols = u(0x1c);
                    found = true;
                } else {
                    if p.len() < 0x10 {
                        continue;
                    }
                    let n = p[0x07] as usize;
                    let off = u16::from_le_bytes([p[0x0e], p[0x0f]]) as usize;
                    let list = (0..n)
                        .filter_map(|k| {
                            let a = 0x10 + off + k * 2;
                            p.get(a..a + 2).map(|b| i16::from_le_bytes([b[0], b[1]]))
                        })
                        .collect();
                    g.cells.insert(r.count as u32, list);
                }
            }
        }
        if !found {
            return Err("no CGrd in the track's CDat".into());
        }
        Ok(g)
    }

    /// `FUN_0021cf30` → `FUN_0021cf80` 0x21cf80: the cells under the box `centre ± r` (x, z):
    /// `(coord − min)·inv` truncated, clamped to the grid, ordered, a span past 20 cells collapses
    /// to its first cell; then columns (x) outer, rows (z) inner, cell = z·cols + x [M].
    pub fn cells_in_box(&self, centre: [f32; 3], r: f32) -> Vec<u32> {
        let clamp = |v: f32, n: u32| -> u32 {
            let i = v as i32;
            if (i as u32) < n {
                i as u32
            } else if i < 0 {
                0
            } else {
                n.saturating_sub(1)
            }
        };
        let mut x0 = clamp((centre[0] - r - self.min_x) * self.inv_cell, self.cols);
        let mut z0 = clamp((centre[2] - r - self.min_z) * self.inv_cell, self.rows);
        let mut x1 = clamp((centre[0] + r - self.min_x) * self.inv_cell, self.cols);
        let mut z1 = clamp((centre[2] + r - self.min_z) * self.inv_cell, self.rows);
        if z1 < z0 {
            std::mem::swap(&mut z0, &mut z1);
        }
        if x1 < x0 {
            std::mem::swap(&mut x0, &mut x1);
        }
        if z1 - z0 > 0x14 {
            z1 = z0;
        }
        if x1 - x0 > 0x14 {
            x1 = x0;
        }
        let mut out = Vec::new();
        for x in x0..=x1 {
            for z in z0..=z1 {
                out.push(z * self.cols + x);
            }
        }
        out
    }
}

/// The camera the traffic code reads (`*DAT_00333ed8`: +0x30 eye, +0x20 forward) and the two
/// world flags (`DAT_0033a024` +0x350: the view test is on; +0x3cc: traffic spawning is off) [M
/// 0x13c860, 0x13ca60, 0x132808]. What sets the two flags was not traced [S].
#[derive(Debug, Clone, Copy)]
pub struct TrafficView {
    pub eye: [f32; 3],
    pub forward: [f32; 3],
    pub view_test: bool,
    pub spawning_off: bool,
}

/// A spawn `FUN_0013ca60` hands back: the piece, lane, start t and direction (`FUN_0013c5e8`)
/// [M].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpawnPick {
    pub seg: usize,
    pub lane: i8,
    pub t: f32,
    pub forward: bool,
}

/// The traffic director state (the AI controller manager's globals) [M].
#[derive(Debug, Clone)]
pub struct TrafficDirector {
    /// `DAT_00327d68` / `DAT_00327d6c`: the ring set by the last list build.
    pub inner: f32,
    pub outer: f32,
    /// `DAT_00327d70`: the list was built around the camera for a script (FUN_0014eea8); the
    /// range tests are off while set.
    pub scripted: bool,
    /// `DAT_003c8278`: the next tick the list may be rebuilt.
    pub refresh_at: i32,
    /// `DAT_00423000` / `DAT_00327d5c`: the candidate pieces.
    pub list: Vec<i16>,
    /// `DAT_00327d64`: successful picks (the lane rotation in FUN_0013c5e8).
    pub spawn_count: u32,
    /// `DAT_0032b752` / `DAT_0032b750`: the live traffic count and its cap (`MAX_TRAFFIC`).
    pub live: i16,
    pub cap: i16,
    /// rr +0x2e per route: the cars counted on it (zeroed at load, 0x220c28).
    pub occupancy: Vec<u8>,
    /// `DAT_00327c3c`: the parking slot offset (z = 40000 + it, +50 per park up to 5000).
    pub park_slot: f32,
    pub lights: JunctionLights,
    /// `DAT_00339f4c`: ticks per second (60 in play [S: not read live]).
    pub fps: i32,
    /// The track's lane table set (the scratch navigator's join).
    pub lane_set: LaneSet,
}

impl TrafficDirector {
    pub fn new(net: &RoadNet, cap: i16, now: i32) -> TrafficDirector {
        TrafficDirector {
            inner: SPAWN_INNER,
            outer: SPAWN_OUTER,
            scripted: false,
            refresh_at: 0,
            list: Vec::new(),
            spawn_count: 0,
            live: 0,
            cap,
            occupancy: vec![0; net.routes.len()],
            park_slot: 0.0,
            lights: JunctionLights::new(now),
            fps: 60,
            lane_set: LaneSet::Default,
        }
    }

    /// `FUN_0013d618` 0x13d618 with param_4 = 0 (the traffic callers): once `refresh_at ≤ now`
    /// (then `refresh_at = now + 0x59`), the list is rebuilt from the grid cells under the box
    /// `player ± outer`: each cell's road pieces in order, each piece once, below the piece
    /// count, skipping flags 0x2, 0x1, 0x8, 0x8000, 0x80, (flags & 0x1020) == 0x1020, and 0x100
    /// pieces whose trigger id (rs+0x59) is set; up to 50. Nothing is built on a track with no
    /// plain road (DAT_003c825c) [M].
    pub fn refresh(&mut self, net: &RoadNet, grid: &RoadGrid, player: [f32; 3], now: i32) -> Result<(), String> {
        if self.refresh_at > now {
            return Ok(());
        }
        self.refresh_at = now + SPAWN_REFRESH;
        self.scripted = false;
        if !net.has_plain_road {
            return Ok(());
        }
        self.inner = SPAWN_INNER;
        self.outer = SPAWN_OUTER;
        self.list.clear();
        let mut seen = vec![false; net.segs.len()];
        'cells: for c in grid.cells_in_box(player, SPAWN_OUTER) {
            let Some(list) = grid.cells.get(&c) else { continue };
            for &s in list {
                if s < 0 || s as usize >= net.segs.len() {
                    continue;
                }
                if self.list.len() > 0x31 {
                    break 'cells;
                }
                if seen[s as usize] {
                    continue;
                }
                seen[s as usize] = true;
                let rs = net.seg(s as usize)?;
                let f = rs.flags();
                if f & (0x2 | 0x1 | 0x8 | 0x8000 | 0x80) != 0 || f & 0x1020 == 0x1020 {
                    continue;
                }
                if f & 0x100 != 0 && rs.trigger_id() != 0 {
                    continue;
                }
                self.list.push(s);
                if self.list.len() >= SPAWN_LIST_MAX {
                    break 'cells;
                }
            }
        }
        Ok(())
    }

    /// `FUN_0013ca60` 0x13ca60 (1,188 B) — pick a spawn [M]: refresh the list; with spawning on
    /// and a list, draw `list[(n·rand) >> 16]`; a piece with no route, or whose route is at its
    /// cap (rr+0x2e ≥ rr+0x2c), fails; `FUN_0013c5e8` gives lane, direction and a second draw
    /// for t; a scratch kind-1 navigator joined there gives the spot; any enabled vehicle whose
    /// grid cell is within 4 (x and z; 8 for one not in state 4) of the spot's cell fails it
    /// (the same-lane branch of the decompile tests the same 4-cell box); the spot must lie at
    /// least `inner` from the player; on success `spawn_count += 1`.
    pub fn pick(&mut self, net: &RoadNet, grid: &RoadGrid, player: [f32; 3], now: i32, view: &TrafficView, vehicles: &[Vehicle], rng: &mut dyn FnMut() -> u16) -> Result<Option<(SpawnPick, [f32; 3])>, String> {
        self.refresh(net, grid, player, now)?;
        if view.spawning_off || self.list.is_empty() {
            return Ok(None);
        }
        let n = self.list.len() as i32;
        let r = rng() as i32;
        let s = self.list[((n * r) >> 16) as usize];
        if s < 0 {
            return Ok(None);
        }
        let seg = s as usize;
        let rs = net.seg(seg)?;
        let route = rs.route();
        if route == -1 {
            return Ok(None);
        }
        let rr = net.routes.get(route as usize).ok_or("route out of range")?;
        let occ = self.occupancy.get(route as usize).copied().unwrap_or(0);
        if occ >= rr.traffic_cap() {
            return Ok(None);
        }
        let t = rng() as f32 * 1.525_878_9e-5;
        let Some((lane, forward)) = spawn_lane_occ(net, seg, occ, self.spawn_count)? else {
            return Ok(None);
        };
        let probe = RoadNav::join(net, seg, t, lane, forward, NAV_KIND_TRAFFIC, self.lane_set)?;
        let c = cell_of(probe.pos);
        for v in vehicles {
            if !v.enabled {
                continue;
            }
            let vc = v.cell();
            let (dx, dz) = ((c[0] - vc[0]).abs(), (c[2] - vc[2]).abs());
            let lim = if v.state == STATE_TRAFFIC { 4 } else { 8 };
            if dx <= lim && dz <= lim {
                return Ok(None);
            }
        }
        if len(sub(player, probe.pos)) < self.inner {
            return Ok(None);
        }
        self.spawn_count += 1;
        Ok(Some((SpawnPick { seg, lane, t, forward }, probe.pos)))
    }

    /// `FUN_0013c860` 0x13c860 — is a car at `pos` out of range (to be respawned)? Never while
    /// spawning is off; never within `inner` of the player; with the view test off, beyond
    /// `outer`; with it on, beyond `inner` and outside the 0.65 view cone. While the list is a
    /// script's (`scripted`) everything is out of range [M].
    pub fn out_of_range(&self, player: [f32; 3], view: &TrafficView, pos: [f32; 3]) -> bool {
        if self.scripted {
            return true;
        }
        if view.spawning_off {
            return false;
        }
        let d = len(sub(player, pos));
        if d <= self.inner {
            return false;
        }
        if !view.view_test {
            return self.outer < d;
        }
        let dir = normalize(sub(pos, view.eye));
        dot(dir, view.forward) < VIEW_DOT
    }

    /// `FUN_0013c798` 0x13c798 — may a car be placed at `pos`? Only beyond `outer + box_z` of the
    /// player (always while `scripted`) [M; the disassembly 0x13c828 subtracts the float-argument box
    /// extent].
    pub fn placement_ok(&self, player: [f32; 3], box_z: f32, pos: [f32; 3]) -> bool {
        if self.scripted {
            return true;
        }
        len(sub(player, pos)) - box_z > self.outer
    }

    /// `FUN_00132ba8` 0x132ba8 — park a traffic car: profile 1, pattern 1, moved to
    /// `(0, 0, 40000 + slot)` (slot += 50, capped at 5000), sim +0x6e = 3, the live count − 1
    /// (floored at 0), its route's occupancy − 1, +0x90 = −1, +0x290 = 0. Returns the park spot
    /// [M].
    pub fn park(&mut self, car: &mut AiCar) -> [f32; 3] {
        let s = self.park_slot + 50.0;
        let s = if 0.0 <= s { s } else { 0.0 };
        self.park_slot = if 5000.0 < s { 5000.0 } else { s };
        car.profile = PROFILE_PARKED;
        car.pattern = 1;
        car.nav.kind = 0;
        self.live = (self.live - 1).max(0);
        if car.route_seen != -1 {
            if let Some(o) = self.occupancy.get_mut(car.route_seen as usize) {
                *o = o.wrapping_sub(1);
            }
            car.route_seen = -1;
        }
        car.respawn_hold = 0;
        [0.0, 0.0, self.park_slot + 40000.0]
    }

    /// `FUN_00132d80`'s route count for a profile-0xc car: entering a piece with another route
    /// moves the car's count from +0x90 to it; a piece with no route keeps the old count [M].
    pub fn count_route(&mut self, net: &RoadNet, car: &mut AiCar) -> Result<(), String> {
        if car.profile != PROFILE_TRAFFIC {
            return Ok(());
        }
        let r = net.seg(car.nav.seg)?.route();
        if car.route_seen != -1 {
            if r == -1 || r == car.route_seen {
                return Ok(());
            }
            if let Some(o) = self.occupancy.get_mut(car.route_seen as usize) {
                *o = o.wrapping_sub(1);
            }
        }
        if r >= 0 {
            if let Some(o) = self.occupancy.get_mut(r as usize) {
                *o = o.wrapping_add(1);
            }
        }
        car.route_seen = r;
        Ok(())
    }
}

/// `FUN_0013c5e8` with the director's route occupancy (the port keeps rr+0x2e as runtime state;
/// the disc bytes are the load-time zero) [M].
pub fn spawn_lane_occ(net: &RoadNet, seg: usize, occ: u8, count: u32) -> Result<Option<(i8, bool)>, String> {
    let s = net.seg(seg)?;
    let n = s.lanes_neg() as i32 + s.lanes_pos() as i32;
    if n <= 0 {
        return Ok(None);
    }
    let base = if s.route() == -1 { 0 } else { occ as u32 };
    let k = ((base + count) % n as u32) as i32;
    let mut c = k as i8;
    if s.lanes_pos() as i32 <= k {
        c += 1;
    }
    let lane = s.lanes_pos() - c;
    if (s.lane_dir_mask() as u32 >> (k & 0x1f)) & 1 != 0 {
        Ok(Some((lane, true)))
    } else {
        Ok(Some((-lane, false)))
    }
}

/// What `FUN_00132808` did to a car this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Respawn {
    /// Nothing (in range, held, or no spot).
    None,
    /// Parked at the spot (FUN_00132ba8).
    Parked([f32; 3]),
    /// Placed: the sim moves to `pos` facing `dir` (vtbl+0x164), sim +0x6e = 2.
    Placed { pos: [f32; 3], dir: [f32; 3] },
}

/// `FUN_00132808` 0x132808 (924 B) — the state-4 respawn, called every frame from
/// `FUN_00132d80` with the cap `DAT_0032b750` [M]:
/// - an enabled car out of range (`out_of_range` of its position) and past its hold: a live
///   profile-0xc car (sim +0x6e = 2) is parked; then, below the cap, `pick` a spot, join the car's
///   navigator there, keep the spot (+0x280) and mark it pending unless the view test sees it
///   (FUN_001c65b0, the `visible` input: a sphere of the car's box_z);
/// - a pending spot that `placement_ok` accepts: profile 0xc, pattern 3 (nav kind 1), the car
///   moves there facing the navigator direction, +0xe4 = 0, live + 1, hold = now + 5 s; the
///   pending flag clears either way.
/// `status` is the car's sim +0x6e.
#[allow(clippy::too_many_arguments)]
pub fn respawn_step(
    dir: &mut TrafficDirector,
    car: &mut AiCar,
    status: &mut i8,
    pos: [f32; 3],
    net: &RoadNet,
    grid: &RoadGrid,
    player: [f32; 3],
    now: i32,
    view: &TrafficView,
    vehicles: &[Vehicle],
    visible: &dyn Fn([f32; 3], f32) -> bool,
    rng: &mut dyn FnMut() -> u16,
) -> Result<Respawn, String> {
    if !car.enabled {
        return Ok(Respawn::None);
    }
    let here = if car.scripted { car.nav.pos } else { pos };
    let mut out = Respawn::None;
    if dir.out_of_range(player, view, here) {
        if now < car.respawn_hold {
            return Ok(Respawn::None);
        }
        if car.profile == PROFILE_TRAFFIC && *status == 2 {
            out = Respawn::Parked(dir.park(car));
            *status = 3;
        }
        if dir.live < dir.cap {
            if let Some((p, _spot)) = dir.pick(net, grid, player, now, view, vehicles, rng)? {
                let kind = car.nav.kind;
                let lane_set = car.nav.lane_set;
                car.nav = RoadNav::join(net, p.seg, p.t, p.lane, p.forward, kind, lane_set)?;
                car.respawn_at = car.nav.pos;
                car.respawn_pending = !(view.view_test && visible(car.respawn_at, car.box_z));
            }
        }
    }
    if car.respawn_pending {
        if dir.placement_ok(player, car.box_z, car.respawn_at) {
            car.profile = PROFILE_TRAFFIC;
            car.pattern = PATTERN_ROAD;
            car.nav.kind = NAV_KIND_TRAFFIC;
            car.nav.repeated = false;
            *status = 2;
            car.stop_wait = false;
            dir.live = (dir.live + 1).max(0);
            car.respawn_hold = now + dir.fps * RESPAWN_HOLD_SECONDS;
            out = Respawn::Placed { pos: car.respawn_at, dir: car.nav.dir };
        }
        car.respawn_pending = false;
    }
    Ok(out)
}

// ------------------------------------------------------------------------------------------
// A headless traffic world (the per-frame order of the pieces above)
// ------------------------------------------------------------------------------------------

/// One traffic car: the driver and its sim +0x6e status (2 live, 3 parked).
pub struct TrafficCar {
    pub drv: AiDriver,
    pub status: i8,
}

/// The traffic of one track run headless: the director, the grid, the cars and the world tick.
/// Per frame [M order where read, S elsewhere]: the lights tick (FUN_001472e0 → FUN_0013ef40;
/// before or after the cars is not proven, the port ticks first), then each car in list order
/// runs FUN_00132d80 (respawn + route count) and, as a live profile-0xc car, the brain with the
/// yield and the physics step. Parked cars (profile 1, pattern 1) are not driven [S: pattern 1's
/// drive is not ported]. The yield reads the other cars as they stand at that point of the frame
/// (earlier cars already moved), as the game's list walk does.
pub struct TrafficWorld {
    pub director: TrafficDirector,
    pub grid: RoadGrid,
    pub cars: Vec<TrafficCar>,
    pub now: i32,
    pub rng: DriveRng,
    pub light_objects: Vec<[f32; 3]>,
}

impl TrafficWorld {
    /// `count` traffic cars built from `params`, all parked (status 3, profile 1) until the
    /// respawn places them [S: the cars' first placement at mission start was not traced].
    pub fn new(net: &RoadNet, grid: RoadGrid, params: CarParams, count: usize, cap: i16, now: i32) -> Result<TrafficWorld, String> {
        let mut director = TrafficDirector::new(net, cap, now);
        let seg = (0..net.segs.len()).find(|&s| net.seg(s).map(|r| r.flags() & 1 == 0).unwrap_or(false)).unwrap_or(0);
        let mut cars = Vec::new();
        for _ in 0..count {
            let mut drv = AiDriver::on_segment(params.clone(), net, seg, 1, 0)?;
            let spot = director.park(&mut drv.brain);
            director.live = 0;
            drv.car.pos = spot;
            drv.car.vel = [0.0; 3];
            cars.push(TrafficCar { drv, status: 3 });
        }
        Ok(TrafficWorld { director, grid, cars, now, rng: DriveRng::game_reset(), light_objects: Vec::new() })
    }

    fn vehicle_of(c: &TrafficCar) -> Vehicle {
        Vehicle {
            pos: c.drv.car.pos,
            box_x: BOX_FLOOR,
            box_z: c.drv.brain.box_z,
            kind: 3,
            status: c.status,
            enabled: c.drv.brain.enabled,
            state: c.drv.brain.state,
            alive: 1.0,
            lane: c.drv.brain.nav.lane,
            end: c.drv.brain.nav.end,
        }
    }

    /// One world frame. `player` is the list's first vehicle; `visible` is the respawn's view
    /// test (FUN_001c65b0, a sphere against the view frustum, not ported) [S].
    pub fn step(&mut self, net: &RoadNet, player: Vehicle, view: &TrafficView, visible: &dyn Fn([f32; 3], f32) -> bool, dt: f32) -> Result<(), String> {
        self.director.lights.tick(self.now);
        let mut vehicles: Vec<Vehicle> = std::iter::once(player).chain(self.cars.iter().map(Self::vehicle_of)).collect();
        for i in 0..self.cars.len() {
            let car = &mut self.cars[i];
            let pos = car.drv.car.pos;
            let r = respawn_step(
                &mut self.director,
                &mut car.drv.brain,
                &mut car.status,
                pos,
                net,
                &self.grid,
                player.pos,
                self.now,
                view,
                &vehicles,
                visible,
                &mut || self.rng.next(),
            )?;
            match r {
                Respawn::Parked(p) => {
                    car.drv.car.pos = p;
                    car.drv.car.vel = [0.0; 3];
                    car.drv.car.speed = 0.0;
                }
                Respawn::Placed { pos, dir } => {
                    car.drv.car.pos = pos;
                    car.drv.car.vel = [0.0; 3];
                    car.drv.car.speed = 0.0;
                    car.drv.car.yaw = dir[0].atan2(dir[2]);
                }
                Respawn::None => {}
            }
            self.director.count_route(net, &mut car.drv.brain)?;
            if car.drv.brain.profile == PROFILE_TRAFFIC && car.status == 2 {
                let lights = self.director.lights;
                let w = YieldWorld { now: self.now, lights, vehicles: &vehicles, me: Some(i + 1), light_objects: &self.light_objects };
                car.drv.step_shared(net, dt, Some(&w), &mut self.rng)?;
            }
            vehicles[i + 1] = Self::vehicle_of(&self.cars[i]);
        }
        self.now += 1;
        Ok(())
    }
}
