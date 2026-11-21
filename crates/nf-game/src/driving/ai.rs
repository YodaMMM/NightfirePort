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
    /// `t' = |1 − t|`; lane offset `±table[class][|lane|]`; P0/P1 through FUN_002267d8; the carrot
    /// at `lerp(P0, P1, t')` [M 0x2257c0:58-180]. Curved pieces (flags 0x4000) evaluate straight
    /// here: the curve object (FUN_001a8380/0x1a85d0) is not ported [S].
    pub fn join(net: &RoadNet, seg: usize, t: f32, lane: i8, forward: bool, kind: u32, lane_set: LaneSet) -> Result<RoadNav, String> {
        let s = net.seg(seg)?;
        let d = s.dir();
        let (end, dir, tt) = if kind == 4 || forward {
            (1u8, d, t)
        } else {
            (0u8, [-d[0], -d[1], -d[2]], (1.0 - t).abs())
        };
        let off = lane_offset(lane_set, s.road_class(), lane);
        let p0 = lane_point(net, seg, end, true, off)?;
        let p1 = lane_point(net, seg, end, false, off)?;
        Ok(RoadNav {
            kind,
            pos: lerp(p0, p1, tt),
            dir,
            end,
            seg,
            t: tt,
            p0,
            p1,
            lane,
            lane_off: off,
            repeated: false,
            lane_set,
        })
    }

    /// `FUN_00224e00` 0x224e00 — move the carrot `dist` along the road [M]: `t += dist/|P0−P1|`;
    /// past 1 the rest of the segment is consumed, the kind-1 chooser picks the next piece
    /// (`next_segment_traffic`), the direction is re-read (negated toward A), P0/P1 are rebuilt
    /// (the old offset, or on a lane change P0 = old P1 and P1 with the new offset), t restarts at
    /// 0 and the leftover distance `(dt − (1 − t))·len` carries into the new piece (the game
    /// recurses); else t is clamped to [0, 1] and the carrot is `lerp(P0, P1, t)`
    /// [M 0x224e00:91-207]. A zero-length piece crosses with nothing left over (the game's
    /// `dist/0` → the PS2 FPU's max float) [S].
    pub fn advance(&mut self, net: &RoadNet, dist: f32, rng: &mut dyn FnMut() -> u16) -> Result<(), String> {
        if self.kind != NAV_KIND_TRAFFIC {
            return Err(format!("nav kind {} chooser not ported (kind 1 only)", self.kind));
        }
        let mut dist = dist;
        for _ in 0..256 {
            let l = len(sub(self.p0, self.p1));
            let dt = if l > 0.0 { dist / l } else if dist > 0.0 { f32::INFINITY } else { 0.0 };
            let t1 = self.t + dt;
            if t1 >= 1.0 {
                let rest = if l > 0.0 { (dt - (1.0 - self.t)) * l } else { 0.0 };
                let pick = next_segment_traffic(net, self, rng)?;
                if pick.seg == self.seg {
                    self.repeated = true;
                }
                self.seg = pick.seg;
                self.end = pick.end;
                self.lane = pick.lane;
                let d = net.seg(pick.seg)?.dir();
                self.dir = if pick.end == 0 { [-d[0], -d[1], -d[2]] } else { d };
                if !pick.lane_change {
                    self.p0 = lane_point(net, pick.seg, pick.end, true, self.lane_off)?;
                    self.p1 = lane_point(net, pick.seg, pick.end, false, self.lane_off)?;
                } else {
                    self.p0 = self.p1;
                    self.p1 = lane_point(net, pick.seg, pick.end, false, pick.lane_off)?;
                    self.lane_off = pick.lane_off;
                }
                self.pos = self.p0;
                self.t = 0.0;
                dist = rest;
                continue;
            }
            self.t = t1.max(0.0).min(1.0);
            if self.t.is_nan() {
                self.t = 0.0;
            }
            self.pos = lerp(self.p0, self.p1, self.t);
            return Ok(());
        }
        Err("road advance did not settle in 256 pieces".into())
    }
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
        // FUN_00132f80 → FUN_00130140: keep the carrot `lookahead` ahead [M 0x130140:106-245].
        let la = lookahead(sim.speed(), self.box_z);
        let gap = len(sub(sim.pos, self.nav.pos));
        if gap < la {
            self.nav.advance(net, (la - gap).abs(), rng)?;
        }
        // nav+0x94 set → the aim is the car itself (stop) [M 0x130508:297-303].
        let aim = if self.nav.repeated { sim.pos } else { self.nav.pos };
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
        let sim = SimCar::of_car(&self.car);
        let rng = &mut self.rng;
        let cmd = match self.brain.step(net, &sim, &mut || rng.next())? {
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
