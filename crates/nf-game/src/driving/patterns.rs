//! The driving engine's per-pattern hostile steering (docs/systems/driving-ai.md §11).
//!
//! `hostile.rs` (§10) ports the AI element lifecycle, the chase speeds, the ram roll, sight and
//! the fire decisions. This file ports the rest of the chase brain, one DRIVING.ELF function per
//! item (EE = file offset + 0x107080):
//!
//! - the aim point and the pursue/road switch of `FUN_00131770` (+0x1c0 / +0x1d0 / +0x1e0 /
//!   +0x210, the pattern-3 lead, the profile-0xd pattern 2 <-> 3 switch, the +0x27c latch);
//! - the pattern dispatch `FUN_00130508` (patterns 1 hold, 2 direct pursuit, 3 road chase,
//!   4 road follower, 5 scripted path) — which arm runs and where it aims;
//! - the chase drive control `FUN_00133f60` (ground) with the reverse logic and the bump probe
//!   `FUN_00133238`, and the flying arm (`+0x118`: the altitude block of `FUN_00133f60` and
//!   `FUN_001349c0`);
//! - the chase avoidance `FUN_00137310` with its histogram build `FUN_001369a0`, the
//!   collision-course test `FUN_00135b10` and the slot pick `FUN_00135cf0`;
//! - the turret aim `FUN_0012f170`;
//! - the world vehicle cap `FUN_002012d0` (36 cars, 48 with its flag);
//! - the helicopter controller's leaf rules: the update gate `FUN_00138ef0`, the speed clamp
//!   `FUN_0013b4c8` and the lift/roll part of the flight control `FUN_0013b4f8` (the one driving
//!   random draw per helicopter per tick);
//! - one rider gunner's aim sweep `FUN_00122a68`.
//!
//! Pure functions: the caller supplies the sim values (positions, rows, velocities, ray hits)
//! and applies the returned commands. The only random draws are `heli_lift`'s, from the
//! driving generator (`ai::DriveRng`). What is not ported is listed in §11.4.

use crate::driving::ai::{DriveCommand, DriveRng};
use crate::driving::hostile::RAND_SCALE;

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn scale(a: [f32; 3], k: f32) -> [f32; 3] {
    [a[0] * k, a[1] * k, a[2] * k]
}
fn len(a: [f32; 3]) -> f32 {
    let s = dot(a, a);
    if s <= 0.0 { 0.0 } else { s.sqrt() }
}
/// The VU normalize; a zero vector stays zero [S, as in `ai.rs`].
fn normalize(a: [f32; 3]) -> [f32; 3] {
    let l = len(a);
    if l > 0.0 { [a[0] / l, a[1] / l, a[2] / l] } else { [0.0; 3] }
}
/// `a × b` — the VU `vopmula`/`vopmsub` pair (`vopmula ACC, a, b; vopmsub d, b, a`).
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
/// `FUN_00210510` 0x210510 as (x, 1, 1) [M].
fn clamp_unit(x: f32) -> f32 {
    if x > 1.0 {
        1.0
    } else if x < -1.0 {
        -1.0
    } else {
        x
    }
}
/// `(int)(x ± 0.5)` — the game's round-half-away [M 0x135d88 / 0x137310 / 0x12f170].
fn round_i(x: f32) -> i32 {
    let h = if x < 0.0 { -0.5 } else { 0.5 };
    (h + x) as i32
}

/// `FUN_002cbd50` 0x2cbd50: sin of `x` turns — reduce to [−0.5, 0.5], then the VU odd
/// polynomial with the coefficients 0xbe2aaaa4 / 0x3c08873e / 0xb94fb21f / 0x362e9c14 [M; the
/// VU accumulation order is the plain sum here, S].
pub fn sin_turns(x: f32) -> f32 {
    let mut p = x;
    if 1.0 < p {
        p -= p as i32 as f32;
    } else if p < 0.0 {
        p -= (p as i32 - 1) as f32;
    }
    if p < -0.5 {
        p += 1.0;
    } else if 0.5 < p {
        p -= 1.0;
    }
    let t = p * 6.283_185_5;
    let t2 = t * t;
    let t3 = t2 * t;
    let t5 = t3 * t2;
    let t7 = t5 * t2;
    let t9 = t7 * t2;
    t + t3 * f32::from_bits(0xbe2a_aaa4) + t5 * f32::from_bits(0x3c08_873e) + t7 * f32::from_bits(0xb94f_b21f) + t9 * f32::from_bits(0x362e_9c14)
}

// ------------------------------------------------------------------------------------------
// Constants
// ------------------------------------------------------------------------------------------

/// The drive patterns AI +0x78 [M 0x130508 switch].
pub const PATTERN_HOLD: u32 = 1;
pub const PATTERN_PURSUE: u32 = 2;
pub const PATTERN_ROAD: u32 = 3;
pub const PATTERN_ROAD_FOLLOW: u32 = 4;
pub const PATTERN_PATH: u32 = 5;
/// The pattern-3 lead floor 0.2 (0x3e4ccccd) and the non-road velocity lead 0.2 s [M 0x131c60 /
/// the `_qmtc2(0x3e4ccccd)` arm of 0x131770]; the lead needs a target moving at ≥ 5 [M 0x131c44].
pub const LEAD_FLOOR: f32 = 0.2;
pub const LEAD_MIN_TARGET_SPEED: f32 = 5.0;
/// The pursue/road switch re-arms 60 ticks after a change (`DAT_00339f54 + 0x3c`) [M 0x131770].
pub const SWITCH_HOLD_TICKS: i32 = 0x3c;
/// FUN_00133f60's ramp span 15 and floor 0.05; the flying arm's span 5 [M 0x133f60 / 0x1349c0].
pub const CHASE_RAMP_SPAN: f32 = 15.0;
pub const FLY_RAMP_SPAN: f32 = 5.0;
pub const RAMP_FLOOR: f32 = 0.05;
/// The avoidance histogram: 64 one-unit lateral slots around the aim line, centre 32
/// (DAT_00399c70 = 32.0 [D the disc ELF]); a car's width byte +0xd8 is 3 at construction
/// [M 0x12ded8, 0x13f748].
pub const SLOTS: usize = 64;
pub const SLOT_CENTRE: f32 = 32.0;
pub const DEFAULT_WIDTH: u8 = 3;
/// The world vehicle cap of `FUN_002012d0`: 36 (0x24), 48 (0x30) with the third argument [M].
pub const WORLD_CAP: i32 = 36;
pub const WORLD_CAP_BIG: i32 = 48;
/// The turret slew per tick in turns (0.005) [M 0x12f170].
pub const TURRET_STEP: f32 = 0.005;

// ------------------------------------------------------------------------------------------
// The sim view
// ------------------------------------------------------------------------------------------

/// The sim-object fields the chase functions read (`FUN_001ff088`'s object).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Body {
    /// +0x10 position, +0x20 velocity.
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    /// sim +0x5c → row 0 (+0x00, the side the steer dot uses), row 1 (+0x10), row 2 (+0x20, forward).
    pub side: [f32; 3],
    pub up: [f32; 3],
    pub fwd: [f32; 3],
    /// sim +0x5c → +0x4c8 (z extent) and +0x4c0 (x extent) [M 0x1f6e98].
    pub box_z: f32,
    pub box_x: f32,
}

impl Body {
    pub fn speed(&self) -> f32 {
        len(self.vel)
    }
}

// ------------------------------------------------------------------------------------------
// The aim point (FUN_00131770)
// ------------------------------------------------------------------------------------------

/// What the aim block of `FUN_00131770` 0x131770 reads (AI offsets).
#[derive(Debug, Clone, Copy)]
pub struct AimInputs {
    pub profile: u32,
    pub pattern: u32,
    /// +0x80: 1 = drive to the goal point +0x1a8.
    pub target_mode: u32,
    pub goal: [f32; 3],
    /// +0x1a4 ≠ 0, and its sim position / velocity (+0x10 / +0x20) and the controller position
    /// `FUN_0013e0b8` (the sim position for a car).
    pub has_target: bool,
    pub target_pos: [f32; 3],
    pub target_vel: [f32; 3],
    /// +0x208 as the sight block left it (1 = lost).
    pub lost: bool,
    pub self_body: Body,
    /// +0x150 lead, +0x220 follow distance, +0x1d0 last frame's distance.
    pub lead: f32,
    pub follow: f32,
    pub prev_dist: f32,
    /// +0x1c0 as last frame left it (kept when this block writes nothing before the distance).
    pub prev_aim: [f32; 3],
    /// nav +0x20 (the carrot) and nav +0x10.
    pub nav_carrot: [f32; 3],
    pub nav_10: [f32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aim {
    /// +0x1c0 the aim point, +0x1d0 the distance to it, +0x1e0 the unit direction.
    pub point: [f32; 3],
    pub dist: f32,
    pub dir: [f32; 3],
    /// +0x210 = n(nav+0x10 − own pos).
    pub nav_dir: [f32; 3],
    /// +0x208 after the block (no target and not mode 1 → lost).
    pub lost: bool,
    /// The flag that asks for a road re-join toward the target (the `FUN_001351d0` /
    /// `FUN_00225c48` arm; profiles 0xe/0xf and patterns ≠ 3 never ask) [M].
    pub rejoin: bool,
}

/// The aim block of `FUN_00131770` [M 0x131770, disassembly 0x131c44-0x131d5c]:
///
/// - profile 0xf with a target: aim = the target's position;
/// - else, not lost: no target (or mode 1) → mode 1 aims at the goal, otherwise lost = 1;
///   with a target the aim is its position, and for a target moving at ≥ 5: pattern 3 adds
///   `n(v_t)·k`, `k = min(max(lead, 0.2), |v_t|)` (and asks for a re-join when `|v_t|·k ≤
///   follow` while last frame's distance exceeds `max(follow, 15)`); other patterns add
///   `v_t·0.2`;
/// - the distance and direction to that point are stored;
/// - lost: pattern 2 aims at the target (or `own pos + forward` without one); other patterns at
///   the carrot; the stored distance and direction are not refreshed (the game computes them
///   first). A lost car with a target asks for a re-join.
pub fn aim_point(i: &AimInputs) -> Aim {
    let me = i.self_body.pos;
    let mut lost = i.lost;
    let mut point = i.prev_aim;
    let mut rejoin = false;
    if i.profile == 0xf && i.has_target {
        point = i.target_pos;
    } else if !lost {
        if !i.has_target || i.target_mode == 1 {
            if i.target_mode == 1 {
                point = i.goal;
                    } else {
                lost = true;
            }
        } else {
            point = i.target_pos;
                let ts = len(i.target_vel);
            if LEAD_MIN_TARGET_SPEED <= ts {
                if i.pattern == PATTERN_ROAD {
                    let k = i.lead.max(LEAD_FLOOR).min(ts);
                    point = add(point, scale(normalize(i.target_vel), k));
                    rejoin = ts * k <= i.follow && !(i.prev_dist <= i.follow.max(15.0));
                } else {
                    point = add(point, scale(i.target_vel, LEAD_FLOOR));
                }
            }
        }
    }
    let dist = len(sub(me, point));
    let dir = normalize(sub(point, me));
    let nav_dir = normalize(sub(i.nav_10, me));
    if lost {
        point = if i.pattern == PATTERN_PURSUE {
            if i.has_target { i.target_pos } else { add(me, i.self_body.fwd) }
        } else {
            i.nav_carrot
        };
        rejoin = i.has_target;
    }
    if matches!(i.profile, 0xe | 0xf) || i.pattern != PATTERN_ROAD {
        rejoin = false;
    }
    Aim { point, dist, dir, nav_dir, lost, rejoin }
}

/// The profile-0xd pursue/road switch of `FUN_00131770` (after the aim block, when the hold
/// timer +0x74 has run out and the car is on a road network) [M 0x131770].
#[derive(Debug, Clone, Copy)]
pub struct SwitchInputs {
    pub profile: u32,
    pub pattern: u32,
    pub lost: bool,
    pub now: i32,
    /// +0x74: 0, or the tick the switch is held until.
    pub hold_until: i32,
    /// nav +0x94 (the navigator repeated its piece).
    pub nav_repeat: bool,
    /// The aim: +0x1d0, +0x1e0; +0x220 follow.
    pub dist: f32,
    pub dir: [f32; 3],
    pub follow: f32,
    /// nav +0x30 (the road direction) and the target's speed.
    pub nav_dir_30: [f32; 3],
    pub target_speed: f32,
    /// Own forward row and speed.
    pub self_fwd: [f32; 3],
    pub self_speed: f32,
    /// Pattern 2 only: the road under the target (`FUN_001351d0`): its lane offset nav +0x98,
    /// the road's half width on that side (`FUN_00221c70` / `FUN_00221a88`) and its direction
    /// nav +0x30.
    pub target_lane: f32,
    pub target_road_half_width: f32,
    pub target_road_dir: [f32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SwitchOut {
    /// The new +0x78 / +0x7c when they changed.
    pub pattern: Option<u32>,
    pub mode_7c: Option<u32>,
    /// The new +0x74.
    pub hold_until: Option<i32>,
    /// Re-join the road at the own position (nav kind 3, then advance 7.5); after it the caller
    /// passes `dot(fwd, n(nav+0x10 − pos))` to `finish_rejoin`.
    pub rejoin: bool,
}

/// Runs the switch. Returns `None` while the hold timer runs or the navigator repeats.
pub fn pursue_switch(i: &SwitchInputs) -> Option<SwitchOut> {
    if i.hold_until != 0 && i.now < i.hold_until {
        return None;
    }
    if i.nav_repeat || i.profile != 0xd {
        return Some(SwitchOut { hold_until: if i.hold_until != 0 { Some(0) } else { None }, ..Default::default() });
    }
    let mut o = SwitchOut { hold_until: if i.hold_until != 0 { Some(0) } else { None }, ..Default::default() };
    if i.pattern == PATTERN_ROAD {
        if dot(i.nav_dir_30, i.dir) < 0.5 && !i.lost {
            o.pattern = Some(PATTERN_PURSUE);
            o.mode_7c = Some(3);
            o.hold_until = Some(i.now + SWITCH_HOLD_TICKS);
        }
        if i.lost || LEAD_MIN_TARGET_SPEED <= i.target_speed {
            return Some(o);
        }
        if i.dist < i.follow {
            o.pattern = Some(PATTERN_PURSUE);
            o.mode_7c = Some(3);
            o.hold_until = Some(i.now + SWITCH_HOLD_TICKS);
        }
    } else if i.pattern == PATTERN_PURSUE {
        let along = dot(i.target_road_dir, i.self_fwd);
        let on_road = 10.0 <= i.self_speed && i.target_lane.abs() < i.target_road_half_width && 0.7 < along;
        if on_road || i.lost {
            o.rejoin = true;
        }
        o.hold_until = Some(i.now + SWITCH_HOLD_TICKS);
    }
    Some(o)
}

/// The end of the pattern-2 re-join: `dot(fwd, +0x210) ≥ 0.7` → pattern 3, +0x7c = 2, +0x27c = 0 [M].
pub fn finish_rejoin(o: &mut SwitchOut, fwd_dot_nav: f32) -> bool {
    if 0.7 <= fwd_dot_nav {
        o.pattern = Some(PATTERN_ROAD);
        o.mode_7c = Some(2);
        true
    } else {
        false
    }
}

/// The +0x27c latch for patterns 3/4 [M 0x131770 tail]: clear → set once `dot(fwd, +0x210) ≥
/// 0.7`; set → when the road runs backwards (`< −0.15`) re-join reversed (the `+0x154 ^ 1`
/// argument) and clear. Returns (new latch, re-join).
pub fn road_heading_latch(latch: bool, fwd_dot_nav: f32) -> (bool, bool) {
    if !latch {
        (0.7 <= fwd_dot_nav, false)
    } else if fwd_dot_nav < -0.15 {
        (false, true)
    } else {
        (true, false)
    }
}

// ------------------------------------------------------------------------------------------
// The pattern dispatch (FUN_00130508)
// ------------------------------------------------------------------------------------------

/// Which drive arm `FUN_00130508` 0x130508 runs this tick and where it aims (+0x1f0).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Arm {
    /// The gated stop (dead, stopped, +0x170) → `ai::gated_stop`.
    Gated,
    /// Nothing driven (pattern 1 outside state 5, unknown pattern).
    Idle,
    /// `FUN_00133f60(speed, aim)` (or the flying arm when +0x118).
    Chase { speed: f32, aim: [f32; 3], avoid: Avoid },
    /// Pattern 3 in state 4: the traffic path (`ai::AiCar::step`).
    Traffic,
    /// Pattern 5: the sim follows the scripted path object (transform + velocity copy).
    Path,
}

/// The avoidance arm the dispatch asks for before the drive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Avoid {
    None,
    /// `FUN_00137310` (state 5, pattern 2).
    Chase,
    /// `FUN_001377c8` (pattern 2 outside state 5) — the traffic yield, not ported.
    Traffic,
    /// `FUN_00135f70` (patterns 3/4) — the road lane pick, not ported.
    Road,
}

#[derive(Debug, Clone, Copy)]
pub struct DispatchInputs {
    pub state: u32,
    pub pattern: u32,
    /// The gates: alive (FUN_00195a30), the vtbl+0x144 stop query, +0x170.
    pub alive: f32,
    pub stop_query: bool,
    pub stop_byte: bool,
    /// +0x8c the target speed, +0x154 avoidance on, nav +0x94.
    pub target_speed: f32,
    pub avoid_on: bool,
    pub nav_repeat: bool,
    pub self_pos: [f32; 3],
    /// +0x1e0 and the lookahead `FUN_00132f80(+0x8c)`.
    pub dir: [f32; 3],
    pub lookahead: f32,
    /// The carrot after `FUN_00130140` (patterns 3/4).
    pub carrot: [f32; 3],
}

/// `FUN_00130508` 0x130508 [M]: the gates as `ai::target_speed`; pattern 1 aims at the own
/// position and drives (speed 0) only in state 5; pattern 2 aims `pos + dir·lookahead`, runs
/// the avoidance (state 5 → `FUN_00137310`, else `FUN_001377c8`) when +0x154 is set, then
/// `FUN_00133f60(+0x8c)`; pattern 3 in state 4 is the traffic path, in other states the carrot
/// + `FUN_00135f70` + `FUN_00133f60`; pattern 4 the carrot, `FUN_00135f70` unless nav +0x94,
/// `FUN_00133f60`; pattern 5 the path copy.
pub fn dispatch(i: &DispatchInputs) -> Arm {
    if (i.state == 5 && i.alive <= 0.0) || (i.state == 4 && i.alive < 0.5) || i.stop_query || i.stop_byte {
        return Arm::Gated;
    }
    match i.pattern {
        PATTERN_HOLD => {
            if i.state == 5 {
                Arm::Chase { speed: 0.0, aim: i.self_pos, avoid: Avoid::None }
            } else {
                Arm::Idle
            }
        }
        PATTERN_PURSUE => {
            let aim = add(i.self_pos, scale(i.dir, i.lookahead));
            let avoid = if !i.avoid_on {
                Avoid::None
            } else if i.state == 5 {
                Avoid::Chase
            } else {
                Avoid::Traffic
            };
            Arm::Chase { speed: i.target_speed, aim, avoid }
        }
        PATTERN_ROAD => {
            if i.state == 4 {
                Arm::Traffic
            } else {
                // FUN_00135f70 returns at once when +0x154 is clear [M 0x135f70:14].
                Arm::Chase { speed: i.target_speed, aim: i.carrot, avoid: if i.avoid_on { Avoid::Road } else { Avoid::None } }
            }
        }
        PATTERN_ROAD_FOLLOW => {
            let avoid = if !i.nav_repeat && i.avoid_on { Avoid::Road } else { Avoid::None };
            Arm::Chase { speed: i.target_speed, aim: i.carrot, avoid }
        }
        PATTERN_PATH => Arm::Path,
        _ => Arm::Idle,
    }
}

// ------------------------------------------------------------------------------------------
// The chase drive control (FUN_00133f60) and its bump probe (FUN_00133238)
// ------------------------------------------------------------------------------------------

/// The reverse state: +0x140 (reversing), +0x25c (timed-reverse ticks), +0x260 (its direction).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReverseState {
    pub reversing: bool,
    pub timer: i16,
    pub timed: bool,
}

/// What `FUN_00133f60` reads besides the body.
#[derive(Debug, Clone, Copy)]
pub struct ChaseDriveInputs {
    pub profile: u32,
    pub pattern: u32,
    /// +0x164 spawn delay, +0x160 / +0x15c (steer holds), nav +0x94.
    pub spawn_delay: i16,
    pub hold_160: bool,
    pub hold_15c: bool,
    pub nav_repeat: bool,
    /// +0x1a4 and whether it is the player (actor vtbl+0x9c == 1, read on the target [M
    /// disassembly 0x1343b0, a0 = +0x1a4]); its sim position, velocity and forward row.
    pub has_target: bool,
    pub target_is_player: bool,
    pub target_pos: [f32; 3],
    pub target_vel: [f32; 3],
    pub target_fwd: [f32; 3],
    /// +0x1e0.
    pub dir: [f32; 3],
    /// The actor's vtbl+0x134 first float (the steer it holds), read only while the spawn
    /// delay runs outside pattern 4.
    pub held_steer: f32,
    /// DAT_00339f4c.
    pub fps: i32,
}

/// The chase command: the four pedal calls plus the pitch call vtbl+0xdc the flying arm uses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChaseCommand {
    /// `None` when SetSteer was not called this tick (the +0x160/+0x15c holds).
    pub steer: Option<f32>,
    pub throttle: f32,
    pub brake: f32,
    pub handbrake: bool,
    pub pitch: f32,
}

impl ChaseCommand {
    pub fn drive(&self) -> DriveCommand {
        DriveCommand { steer: self.steer.unwrap_or(0.0), throttle: self.throttle, brake: self.brake, handbrake: self.handbrake }
    }
}

/// `FUN_00133238` 0x133238, the bump probe: only after a contact (+0xdc set, +0xe0 == 6; the
/// contact flag is cleared by the call). Forward: one ray from the position to `pos + fwd ·
/// 2·box_z`. Reversing: the ray runs backwards, from `pos ± side·box_x` (the right corner
/// first, the left if it is clear). Blocked = the ray reports ≥ 2 hits (`FUN_00215cc8`) [M].
/// `ray(from, to)` is the world ray's hit count.
pub fn bump_probe(contact: &mut bool, kind: u8, reversing: bool, b: &Body, ray: &mut dyn FnMut([f32; 3], [f32; 3]) -> u32) -> bool {
    if !*contact {
        return false;
    }
    if kind != 6 {
        return false;
    }
    *contact = false;
    let reach = b.box_z + b.box_z;
    if !reversing {
        let to = add(b.pos, scale(b.fwd, reach));
        return ray(b.pos, to) >= 2;
    }
    let back = scale(b.fwd, -1.0);
    // The corners: `(back.z, back.y, −back.x)·box_x` then its mirror `(−x, y, −z)` [M 0x133238].
    let side = [back[2], back[1], -back[0]];
    let a = add(add(b.pos, scale(side, b.box_x)), scale(back, reach));
    if ray(b.pos, a) >= 2 {
        return true;
    }
    let side2 = [-side[0], side[1], -side[2]];
    let c = add(add(b.pos, scale(side2, b.box_x)), scale(back, reach));
    ray(b.pos, c) >= 2
}

/// The reverse decision shared by `FUN_00133f60` (with the +0x140 hysteresis) and
/// `FUN_001349c0` (without). `ahead` = dot(fwd, dir to aim), `vel_fwd` = dot(fwd, n(v)).
/// Returns whether this tick drives in reverse and whether the head-on rule (`bVar2`) fired.
fn reverse_decision(st: &mut ReverseState, hysteresis: bool, ahead: f32, vel_fwd: f32, head_on: &dyn Fn() -> bool, nav_repeat: bool, fps: i32, probe: &mut dyn FnMut(bool) -> bool) -> (bool, bool) {
    let mut t = st.timer as i32;
    if hysteresis && 0 < t && t <= fps / 2 && 0.7 < ahead {
        st.timer = 0;
        t = 0;
    }
    if 0 < t {
        st.timer = (t - 1).max(0) as i16;
        return (st.timed, false);
    }
    if hysteresis {
        if st.reversing {
            if !(ahead <= 0.7 || 0.0 <= vel_fwd) {
                st.reversing = false;
            } else if -0.15 <= ahead && 0.0 < vel_fwd {
                st.reversing = false;
            }
        } else if ahead < -0.15 {
            st.reversing = true;
        }
    }
    let mut rev = st.reversing && hysteresis;
    let blocked_head_on = head_on();
    if blocked_head_on {
        return (rev, true);
    }
    if !nav_repeat && probe(st.reversing) {
        rev = !st.reversing;
        st.timer = fps as i16;
        st.timed = rev;
    }
    (rev, false)
}

/// `FUN_00133f60` 0x133f60 on the ground (+0x118 clear) [M; the actor of the vtbl+0x9c query
/// and the final handbrake argument from the disassembly 0x1343b0 / 0x134950]:
///
/// - `side = dot(d, row0)`, `vel_fwd = dot(fwd, n(v))`, `ahead = dot(fwd, d)`, `s = ±|v|` (minus
///   when rolling backwards); `ramp = clamp(min(|target − s|, 15)/15, 0.05, 1)`;
/// - the reverse: a timed reverse (+0x25c) counts down (cut short once the aim is ahead, `>
///   0.7`, in its second half); else the +0x140 hysteresis (on below `ahead < −0.15`, off when
///   the aim is well ahead while rolling backwards or ahead with forward motion); profile 0xd
///   against the player, within 10 and head-on (`dot(target heading, dir) ≤ −0.7`, the heading
///   = n(v) above speed 1 else the forward row) skips the probe and cancels the handbrake;
///   otherwise a bump (`FUN_00133238`) flips the direction for one second;
/// - steer (when the spawn delay is out or pattern 4, and no +0x160/+0x15c hold): reversing →
///   full lock opposite the side; forward at `|s| ≥ 1` with forward motion → `side`; else 0.
///   During the spawn delay (not pattern 4): `clamp(2·held)` (0.1 when held is 0);
/// - pedals: target < 0.05 → below speed 1 release with the handbrake on at ≥ 0.1, else
///   throttle 1 when rolling backwards, brake 1 when forwards; reversing → rolling forwards
///   brake 1, else brake 0.5 up to speed 7.5; forward → `s < target` throttle(ramp); a near
///   match (`≤ 2.5` off, target ≥ 5) holds; rolling forwards brake(ramp), else throttle(ramp).
pub fn chase_drive(b: &Body, aim: [f32; 3], target: f32, st: &mut ReverseState, i: &ChaseDriveInputs, probe: &mut dyn FnMut(bool) -> bool) -> ChaseCommand {
    let d = normalize(sub(aim, b.pos));
    let side = dot(d, b.side);
    let vel_fwd = dot(b.fwd, normalize(b.vel));
    let ahead = dot(b.fwd, d);
    let mut s = len(b.vel);
    if vel_fwd < 0.0 {
        s = -s;
    }
    let span = (target - s).abs().min(CHASE_RAMP_SPAN);
    let ramp = (span / CHASE_RAMP_SPAN).max(RAMP_FLOOR).min(1.0);

    let head_on = || {
        if !(i.has_target && i.profile == 0xd && i.target_is_player) {
            return false;
        }
        let heading = if 1.0 < len(i.target_vel) { normalize(i.target_vel) } else { i.target_fwd };
        len(sub(b.pos, i.target_pos)) <= 10.0 && dot(heading, i.dir) <= -0.7
    };
    let (rev, head) = reverse_decision(st, true, ahead, vel_fwd, &head_on, i.nav_repeat, i.fps, probe);

    let mut cmd = ChaseCommand { steer: None, throttle: 0.0, brake: 0.0, handbrake: false, pitch: 0.0 };
    if i.spawn_delay == 0 || i.pattern == PATTERN_ROAD_FOLLOW {
        if !i.hold_160 && !i.hold_15c {
            let st_in = if rev {
                if side < 0.0 { 1.0 } else { -1.0 }
            } else if 1.0 <= s.abs() && 0.0 <= vel_fwd {
                side
            } else {
                0.0
            };
            cmd.steer = Some(clamp_unit(st_in));
        }
    } else {
        let h = if i.held_steer == 0.0 { 0.1 } else { i.held_steer };
        cmd.steer = Some(clamp_unit(h + h));
    }
    if target < 0.05 {
        let a = s.abs();
        if a < 1.0 {
            cmd.steer = Some(0.0);
            cmd.handbrake = 0.1 <= a;
        } else if vel_fwd < 0.0 {
            cmd.throttle = 1.0;
        } else {
            cmd.brake = 1.0;
        }
    } else if rev {
        if vel_fwd <= 0.0 {
            if s.abs() <= 7.5 {
                cmd.brake = 0.5;
            }
        } else {
            cmd.brake = 1.0;
        }
    } else if s < target {
        cmd.throttle = ramp;
    } else if span <= 2.5 && 5.0 <= target {
    } else if 0.0 < vel_fwd {
        cmd.brake = ramp;
    } else {
        cmd.throttle = ramp;
    }
    if head {
        cmd.handbrake = false;
    }
    cmd
}

/// The flying car state (+0x118 set: the actor's +0xbc byte at controller creation [M
/// 0x13f748]): +0x119 altitude hold, +0x11c ceiling, +0x120 floor, +0x124 bob phase, +0x128
/// bob height (10, or 2.5 for an `IS_HELI` car, set by pattern 4 [M 0x12e468]).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FlyState {
    pub hold: bool,
    pub ceiling: f32,
    pub floor: f32,
    pub phase: f32,
    pub bob: f32,
}

/// The +0x118 block of `FUN_00133f60` [M 0x133f60:22-46]: with the altitude hold, the aim's
/// height is the target's (plus `sin(phase)·bob`, phase += dt·0.05 wrapping at 1, when bob >
/// 0), clamped to [floor, ceiling]; then `FUN_001349c0`.
pub fn fly_aim(fs: &mut FlyState, aim: &mut [f32; 3], target_y: f32, dt: f32) {
    if !fs.hold {
        return;
    }
    if 0.0 < fs.bob {
        fs.phase += dt * 0.05;
        if 1.0 < fs.phase {
            fs.phase -= 1.0;
        }
        aim[1] = target_y + sin_turns(fs.phase) * fs.bob;
    } else {
        aim[1] = target_y;
    }
    let y = aim[1].max(fs.floor);
    aim[1] = if fs.ceiling < y { fs.ceiling } else { y };
}

/// `FUN_001349c0` 0x1349c0, the flying drive [M]: the up row is world up turned square to the
/// nose (`fwd × (up × fwd)`, side `up × fwd`) unless the nose is steep (`|fwd.y| > 0.707`: the
/// body rows 1 and 0); yaw = `dot(d, side)`, pitch = `dot(d, up')`. The reverse as
/// `FUN_00133f60` without the hysteresis (bump flips only); reversing zeroes yaw and pitch.
/// Off the nose (`ahead < 0.7`) the target becomes `0.75·s`. With no target (or the altitude
/// hold) the pitch is dropped when it would steepen a nose already past ±0.4. SetSteer(yaw),
/// pitch call(−pitch); ramp over 5: target < 0.05 → below speed 1 nothing, else throttle 1
/// rolling backwards / brake 1; forward → `s < target` throttle(ramp), stopped/backwards
/// throttle(ramp), else brake(ramp); reversing → brake 0.5.
pub fn fly_drive(b: &Body, aim: [f32; 3], target: f32, hold: bool, st: &mut ReverseState, nav_repeat: bool, fps: i32, probe: &mut dyn FnMut(bool) -> bool) -> ChaseCommand {
    let fy = b.fwd[1];
    let (side_row, up_row) = if fy.abs() <= 0.707 {
        let side = cross([0.0, 1.0, 0.0], b.fwd);
        (side, cross(b.fwd, side))
    } else {
        (b.side, b.up)
    };
    let d = normalize(sub(aim, b.pos));
    let mut yaw = dot(d, side_row);
    let mut pitch = dot(d, up_row);
    let vel_fwd = dot(b.fwd, normalize(b.vel));
    let ahead = dot(b.fwd, d);
    let mut s = len(b.vel);
    if vel_fwd < 0.0 {
        s = -s;
    }
    let (rev, _) = reverse_decision(st, false, ahead, vel_fwd, &|| false, nav_repeat, fps, probe);
    if rev {
        yaw = 0.0;
        pitch = 0.0;
    }
    let mut t = target;
    if 0.05 <= target && ahead < 0.7 {
        t = s * 0.75;
    }
    if t == 0.0 || hold {
        if pitch < 0.0 && fy < -0.4 {
            pitch = 0.0;
        } else if 0.0 < pitch && 0.4 < fy {
            pitch = 0.0;
        }
    }
    let mut cmd = ChaseCommand { steer: Some(clamp_unit(yaw)), throttle: 0.0, brake: 0.0, handbrake: false, pitch: -clamp_unit(pitch) };
    let ramp = ((t - s).abs().min(FLY_RAMP_SPAN) / FLY_RAMP_SPAN).max(RAMP_FLOOR).min(1.0);
    if t < 0.05 {
        if s.abs() < 1.0 {
            return cmd;
        }
        if vel_fwd < 0.0 {
            cmd.throttle = 1.0;
        } else {
            cmd.brake = 1.0;
        }
    } else if !rev {
        if s < t || vel_fwd <= 0.0 {
            cmd.throttle = ramp;
        } else {
            cmd.brake = ramp;
        }
    } else {
        cmd.brake = 0.5;
    }
    cmd
}

// ------------------------------------------------------------------------------------------
// The chase avoidance (FUN_00137310, FUN_001369a0, FUN_00135b10, FUN_00135cf0)
// ------------------------------------------------------------------------------------------

/// AI +0x98..+0xd7, the 64 lateral slots, and +0xd8 the own width in slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slots {
    pub cost: [u8; SLOTS],
    pub width: u8,
}

impl Default for Slots {
    fn default() -> Slots {
        Slots { cost: [0; SLOTS], width: DEFAULT_WIDTH }
    }
}

/// `FUN_001369a0` 0x1369a0 [M]: clear the slots; the world rays from the aim point 32 units to
/// each side (side = `(d.z, d.y, −d.x)` of `d = n(aim − pos)`): a wall at distance `w` on the
/// + side fills slots above `round(w) + 32` with 0xff, on the − side the slots below `32 −
/// round(w)`; then every slot gets `+ round(|32 − i|·5)` (saturating at 0xff); the three centre
/// slots 31..33 are cleared. `walls` = the hit distances (+ side, − side) from aim.
pub fn slots_init(sl: &mut Slots, walls: (Option<f32>, Option<f32>)) {
    sl.cost = [0; SLOTS];
    if let Some(w) = walls.0 {
        let lim = round_i(w) + 32;
        let mut i: i32 = 63;
        loop {
            if lim < 0 {
                if i < 1 {
                    break;
                }
            } else if i <= lim {
                break;
            }
            sl.cost[i as usize] = 0xff;
            i -= 1;
        }
    }
    if let Some(w) = walls.1 {
        let lim = 32 - round_i(w);
        let mut i: i32 = 0;
        loop {
            if lim < 0x41 {
                if lim <= i {
                    break;
                }
            } else if 0x3f < i {
                break;
            }
            sl.cost[i as usize] = 0xff;
            i += 1;
        }
    }
    for (i, c) in sl.cost.iter_mut().enumerate() {
        let add = round_i((SLOT_CENTRE - i as f32).abs() * 5.0);
        let v = *c as i32 + add;
        *c = if v < 0x100 { v.max(0) as u8 } else { 0xff };
    }
    sl.cost[0x1f] = 0;
    sl.cost[0x20] = 0;
    sl.cost[0x21] = 0;
}

/// `FUN_00135b10` 0x135b10: is `other` on the course? Same height band (±5, unless `any_height`),
/// within `clamp(2·speed, 20, 35)` (15 when `other_speed > 0` and `≥ speed + 1`), and ahead
/// (`dot(n(dir), n(other − me)) ≥ 0`) [M].
pub fn on_course(speed: f32, other_speed: f32, me: [f32; 3], other: [f32; 3], dir: [f32; 3], any_height: bool) -> bool {
    if !(any_height || (me[1] - other[1]).abs() <= 5.0) {
        return false;
    }
    let dist = len(sub(me, other));
    let mut r = (speed + speed).max(20.0).min(35.0);
    if 0.0 < other_speed && speed + 1.0 <= other_speed {
        r = 15.0;
    }
    if !(dist <= r) {
        return false;
    }
    !(dot(normalize(dir), normalize(sub(other, me))) < 0.0)
}

/// `FUN_00135cf0` 0x135cf0: the window of `width` slots with the smallest sum (first wins;
/// start below `64 − width`), returned as its centre `start + width/2`; 32 when none is under
/// `width·255` [M, disassembly 0x135cf0-0x135d84].
pub fn best_slot(sl: &Slots) -> i32 {
    let w = sl.width as i32;
    let mut best = w * 0x100 - w;
    let mut out = 0x20;
    if 0 < 0x40 - w {
        let mut i = 0;
        while i < 0x40 - w {
            let sum: i32 = (i..i + w).map(|k| sl.cost[k as usize] as i32).sum();
            if sum < best {
                out = i + (w >> 1);
                best = sum;
            }
            i += 1;
        }
    }
    out
}

/// One other car for the avoidance pass.
#[derive(Debug, Clone, Copy)]
pub struct OtherCar {
    pub id: i16,
    pub pos: [f32; 3],
    /// sim +0x6d < 4 and sim +0x6e ≠ 3 (not traffic).
    pub sim_6d: i8,
    pub sim_6e: i8,
    /// Its row 0 and box z; its controller's width byte +0xd8.
    pub side: [f32; 3],
    pub box_z: f32,
    pub width: u8,
    /// It is this car's target (+0x1a4).
    pub is_target: bool,
}

/// `FUN_00137310` 0x137310 (pattern 2 in state 5, avoidance on) [M; arguments from the
/// disassembly 0x137388-0x1374e0]: without a navigator repeat the slots are rebuilt
/// (`slots_init`) and every other non-traffic car that is not the target and is on the course
/// (`on_course(own speed, 0, own pos, its pos, +0x1e0, false)`) marks `+0x80` over its slots:
/// lateral `32 + dot(side, its pos − own)`, side = `(d.z, d.y, −d.x)`, width `max(its width
/// byte, round(its box_z·|dot(its row 0, d)|))`, from `round(lateral) − width/2` (≥ 0) to
/// `min(|start + width|, 64)`. Then the aim moves sideways by `best_slot − 32` (− 0.5 when the
/// own width is even) along `n(d.z, d.y, −d.x)`.
pub fn avoid_chase(sl: &mut Slots, me: &Body, shooter: i16, dir: [f32; 3], nav_repeat: bool, walls: (Option<f32>, Option<f32>), others: &[OtherCar], aim: &mut [f32; 3]) {
    let side = [dir[2], dir[1], -dir[0]];
    if !nav_repeat {
        slots_init(sl, walls);
        let speed = me.speed();
        for o in others {
            if o.id == shooter || !(o.sim_6d < 4 && o.sim_6e != 3) || o.is_target {
                continue;
            }
            if !on_course(speed, 0.0, me.pos, o.pos, dir, false) {
                continue;
            }
            let lat = SLOT_CENTRE + dot(side, sub(o.pos, me.pos));
            let mut w = o.width as i32;
            let span = o.box_z * dot(o.side, dir).abs();
            if w <= round_i(span) {
                w = round_i(span);
            }
            let start = (round_i(lat) - w / 2).max(0);
            let end = (start + w).abs().min(0x40);
            for k in start..end {
                let v = sl.cost[k as usize] as i32 + 0x80;
                sl.cost[k as usize] = v.min(0xff) as u8;
            }
        }
    }
    let mut off = (best_slot(sl) - 0x20) as f32;
    if sl.width & 1 == 0 {
        off -= 0.5;
    }
    *aim = add(*aim, scale(normalize(side), off));
}

// ------------------------------------------------------------------------------------------
// The turret (FUN_0012f170)
// ------------------------------------------------------------------------------------------

/// AI +0x234 (turret angle in turns) and +0x238 (on target).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Turret {
    pub angle: f32,
    pub on_target: bool,
}

/// `FUN_0012f170` 0x12f170 [M; the cross product order from the disassembly 0x12f26c]: skipped
/// while lost unless target mode 1. The wanted angle `a = acos(clamp(dot(+0x1e0, fwd)))/2π`,
/// `1 − a` when `(dir × fwd).y < 0`. `TURRET_RANGE` (four degrees: two arcs) limits where the
/// turret turns; outside both arcs it holds. It slews 0.005 turns per tick the short way
/// (snapping within 0.005), wrapping at 0/1. Returns the bone angle in degrees
/// (`round(angle·360)`, channel 0x10 frame 8) or `None` when the update was skipped.
pub fn turret_step(t: &mut Turret, lost: bool, target_mode: u32, dir: [f32; 3], fwd: [f32; 3], range: Option<[f32; 4]>) -> Option<i32> {
    if lost && target_mode != 1 {
        return None;
    }
    let c = dot(dir, fwd).max(-1.0).min(1.0);
    let mut a = c.acos() * 0.159_154_94;
    if cross(dir, fwd)[1] < 0.0 {
        a = 1.0 - a;
    }
    let ok = match range {
        None => true,
        Some(r) => {
            let deg = a * 360.0;
            (r[0] <= deg && deg <= r[1]) || (r[2] <= deg && deg <= r[3])
        }
    };
    if ok {
        let cur = t.angle;
        if (a - cur).abs() <= TURRET_STEP {
            t.angle = a;
        } else {
            let mut n;
            if a <= cur || a < 0.75 {
                if a < cur {
                    if 0.25 < a {
                        n = cur - TURRET_STEP;
                    } else if 0.75 <= cur {
                        n = cur + TURRET_STEP;
                    } else {
                        n = cur - TURRET_STEP;
                    }
                } else if cur < a {
                    n = cur + TURRET_STEP;
                } else {
                    n = cur - TURRET_STEP;
                }
            } else if cur <= 0.25 {
                n = cur - TURRET_STEP;
            } else {
                n = cur + TURRET_STEP;
            }
            if 1.0 < n {
                n -= 1.0;
            } else if n < 0.0 {
                n += 1.0;
            }
            t.angle = n;
        }
    }
    t.on_target = t.angle == a;
    Some(round_i(t.angle * 360.0))
}

// ------------------------------------------------------------------------------------------
// The world vehicle cap (FUN_002012d0)
// ------------------------------------------------------------------------------------------

/// `FUN_002012d0` 0x2012d0, asked before a vehicle is created at `at` [M]: cap 36 (48 with
/// `big`). Over the cap, the farthest car (squared distance, strictly farther than the last
/// pick, skipping sim +0x6e == 2) is removed (`FUN_0019e610`); then the create may go ahead
/// when the count is below the cap. A count equal to the cap removes nothing and refuses.
/// `cars` = (sim position, sim +0x6e). Returns (index removed, may create).
pub fn world_cap(count: i32, cars: &[([f32; 3], i8)], at: [f32; 3], big: bool) -> (Option<usize>, bool) {
    let cap = if big { WORLD_CAP_BIG } else { WORLD_CAP };
    if !(cap < count) {
        return (None, count < cap);
    }
    let mut best = 0.0f32;
    let mut pick = None;
    for (k, (p, kind)) in cars.iter().enumerate() {
        if *kind == 2 {
            continue;
        }
        let d = sub(*p, at);
        let d2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
        if best < d2 {
            pick = Some(k);
            best = d2;
        }
    }
    match pick {
        None => (None, count < cap),
        Some(k) => (Some(k), count - 1 < cap),
    }
}

// ------------------------------------------------------------------------------------------
// The helicopter controller's leaf rules
// ------------------------------------------------------------------------------------------

/// `FUN_00138ef0` 0x138ef0, the AIHelicopter update: only with +0x58 and +0x5c set; outside
/// states 2, 9, 10 and with +0x78 clear the goal +0xa0 tracks the target's position; then the
/// death check `FUN_0013b2a8` and, when it does not end the tick, the state machine
/// `FUN_00139be8` [M]. Returns whether the goal follows the target and whether the state
/// machine runs (`death_handled` = FUN_0013b2a8's verdict).
pub fn heli_update_gate(enabled: (bool, bool), state: u32, flag_78: bool, death_handled: bool) -> (bool, bool) {
    if !(enabled.0 && enabled.1) {
        return (false, false);
    }
    let follow = !matches!(state, 2 | 9 | 10) && !flag_78;
    (follow, !death_handled)
}

/// `FUN_0013b4c8` 0x13b4c8: `clamp(x·0.5, 10, 20)` (the helicopter's advance per tick) [M].
pub fn heli_advance(x: f32) -> f32 {
    (x * 0.5).max(10.0).min(20.0)
}

/// The lift and roll outputs of `FUN_0013b4f8` 0x13b4f8 [M]. `ground` = the ground height
/// under the helicopter (`FUN_002327a0`); `alt` (param_1) ≠ 0 puts the goal height at `ground
/// + alt`, else `goal.y + alt`; `max_speed` (param_2, zeroed while +0xd6 counts down); `flat_speed`
/// = |velocity in the ground plane|; `roll_dot` = dot(flat direction to the goal, the body row
/// `FUN_001fd598` returns). Higher than 2 above the ground: roll (+0x94) approaches
/// `roll_dot·max_speed/50` by 0.02 per tick while slower than `max_speed`, else 0. Lift (+0xa0)
/// = `2·(min(goal.y − y, 10) − v.y) + 9.8`, × `clamp(rand·4/65536, 0.5, 1)` higher than 2 above
/// the ground. One driving-generator draw per call, always. The full flight control, with the
/// yaw +0x98, is `heli::Heli::flight`.
pub struct HeliLift {
    pub goal_y: f32,
    pub roll: f32,
    pub lift: f32,
}

#[allow(clippy::too_many_arguments)]
pub fn heli_lift(pos_y: f32, vel_y: f32, ground: f32, goal_y: f32, alt: f32, max_speed: f32, flat_speed: f32, roll_dot: f32, roll: f32, rng: &mut DriveRng) -> HeliLift {
    let mut gy = goal_y;
    if alt != 0.0 {
        gy = ground;
    }
    gy += alt;
    let height = pos_y - ground;
    let mut r = roll;
    if 2.0 < height {
        r = if flat_speed < max_speed { approach(roll, roll_dot * (max_speed / 50.0), f32::from_bits(0x3ca3_d70a)) } else { 0.0 };
    }
    let e = (gy - pos_y).min(10.0);
    let k = (rng.next() as i32 as f32 * RAND_SCALE * 4.0).max(0.5).min(1.0);
    let mut lift = (e - vel_y) * 2.0 + 9.8;
    // The game computes `e − v.y` then doubles it: `fVar11 = fVar11 - v.y; fVar11 = fVar11 +
    // fVar11 + 9.8` [M 0x13b9c0-0x13bb6c].
    if 2.0 < height {
        lift *= k;
    }
    HeliLift { goal_y: gy, roll: r, lift }
}

/// `FUN_002104a8` 0x2104a8: step `cur` toward `target` by `rate`, landing on it [M].
pub fn approach(cur: f32, target: f32, rate: f32) -> f32 {
    if target < cur {
        let n = cur - rate;
        if n < target { target } else { n }
    } else {
        let n = cur + rate;
        if target < n { target } else { n }
    }
}

// ------------------------------------------------------------------------------------------
// One rider gunner's aim sweep (FUN_00122a68)
// ------------------------------------------------------------------------------------------

/// `FUN_00122a68` 0x122a68, the rider class with vtable 0x369200 (ctor `FUN_001224b0`): unless
/// its vtbl+0x124 query holds, the yaw +0x154 steps +2 (when `+0x84 ^ +0x160` = 0) or −2 (= 1)
/// degrees per tick, the bone angle is `clamp(yaw − 55, −80, 30)` (computed before the yaw's
/// own clamp to [−60, 80]); the bone takes a y-axis rotation by that angle [M]. Which seat the
/// class sits in and what drives +0x84 / +0x160 were not traced. Returns the bone angle.
pub fn gunner_sweep(yaw: &mut f32, held: bool, a: u32, b: u32) -> Option<f32> {
    if held {
        return None;
    }
    let step = match a ^ b {
        0 => 2.0,
        1 => -2.0,
        _ => 0.0,
    };
    *yaw += step;
    let mut bone = -80.0f32;
    if -80.0 <= *yaw - 55.0 {
        bone = *yaw - 55.0;
    }
    if 30.0 < bone {
        bone = 30.0;
    }
    if *yaw < -60.0 {
        *yaw = -60.0;
    }
    if 80.0 < *yaw {
        *yaw = 80.0;
    }
    Some(bone)
}
