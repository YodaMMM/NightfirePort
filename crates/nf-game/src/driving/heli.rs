//! The driving engine's AI helicopter controller (docs/systems/driving-ai.md §12).
//!
//! One DRIVING.ELF function per item (EE = file offset + 0x107080; the class vtable is
//! 0x36b888: 0x13bd30 type, 0x138280 dtor, 0x1382f0 reset, 0x138ef0 update, 0x13b468 set path,
//! 0x13b498 release):
//!
//! - the reset `FUN_001382f0` and the command apply `FUN_00138fa8` (an AICo record handed over
//!   by `FUN_00145ff8` when the actor's controller is an AIHelicopter, kind 2);
//! - the update `FUN_00138ef0` (the gate is `patterns::heli_update_gate`), the death check
//!   `FUN_0013b2a8`, the state machine `FUN_00139be8` (states 1-5, 9-11, 16; 6/7/8 reviewed);
//! - the movement `FUN_00139380` (mode 2 hover to the goal, mode 3 along the road, mode 5 the
//!   scripted path);
//! - the flight control `FUN_0013b4f8` in full (goal height, the yaw +0x98, the roll +0x94,
//!   the lift +0xa0 with its one driving-generator draw);
//! - the face helper `FUN_001385d8`, the gun `FUN_001387a0` (cone, cadence, the three-draw
//!   spread) and the rockets `FUN_00138b88` (two launchers alternating, cone, cadence).
//!
//! The controller is pure logic: [`HeliEnv`] supplies the world (ground height, the road
//! navigators, the scripted-path pose, the crash sweep, the launchers) and the target's body;
//! the actor-side values the game keeps on the helicopter actor (+0x94 roll, +0x98 yaw, +0x9c
//! pitch, +0xa0 lift, +0x74 spin) live in [`HeliActor`]. How the helicopter physics class
//! (`FUN_002f1428`, chosen by `FUN_00186878` for `helicopter`/`helivanquish` sims) turns those
//! four inputs into motion was not read: callers bring their own integration.
//!
//! Random draws: the flight control draws once per call; a gun shot draws three times. All are
//! `FUN_0017dc28(DAT_00339190)`, the driving generator (`ai::DriveRng`).

use crate::driving::ai::DriveRng;
use crate::driving::hostile::RAND_SCALE;
use crate::driving::patterns::{approach, heli_advance, Body};
use nf_formats::driveai::AiCommand;

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
    dot(a, a).max(0.0).sqrt()
}
/// The VU normalize (`vrsqrt` of the dot); a zero vector stays zero [S, as `patterns.rs`].
fn norm(a: [f32; 3]) -> [f32; 3] {
    let l = len(a);
    if l > 0.0 { scale(a, 1.0 / l) } else { a }
}
fn flat(a: [f32; 3]) -> [f32; 3] {
    [a[0], 0.0, a[2]]
}

/// `(short)(int)x` [M].
fn short_of(x: f32) -> i16 {
    x as i32 as i16
}

/// The helicopter states (+0x7c) [M 0x139be8 switch, table at 0x36b840].
pub mod state {
    /// Idle: the movement only (`FUN_00139380(1, 0)`); reset and release set it [M 0x1382f0,
    /// 0x13b498].
    pub const IDLE: u32 = 1;
    /// Parked at the goal: position = +0xa0, velocity and +0x30 zeroed every tick.
    pub const PARKED: u32 = 2;
    /// Approach: the goal is the target (plus its velocity × twice the arrive radius when
    /// +0xdd), the movement, then the gun at 65 while armed.
    pub const APPROACH: u32 = 3;
    /// As state 1.
    pub const IDLE_B: u32 = 4;
    /// Chase: the movement with the heading blend, pitch from the facing to the target.
    pub const CHASE: u32 = 5;
    /// 6/7/8: the road fly-by (150 / 35 offsets) — reviewed, not ported (no AICo on the disc
    /// sets them).
    pub const FLYBY_6: u32 = 6;
    pub const FLYBY_7: u32 = 7;
    pub const FLYBY_8: u32 = 8;
    /// 9 gun / 10 rockets at the command point +0x90, range 1000.
    pub const GUN_POST: u32 = 9;
    pub const ROCKET_POST: u32 = 10;
    /// +0x78 = 1, then the movement `FUN_00139380(0, 0)`.
    pub const HOLD: u32 = 11;
    /// Dead: the crash fall, the impact, the despawn.
    pub const DEAD: u32 = 16;
}

/// The movement modes (+0x74) [M 0x139380 switch].
pub mod mode {
    pub const NONE: u32 = 1;
    pub const HOVER: u32 = 2;
    pub const ROAD: u32 = 3;
    pub const PATH: u32 = 5;
}

/// The actor-side values the controller reads and writes on the helicopter actor
/// (`FUN_0013dfc8()`), plus the sim body.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeliActor {
    pub body: Body,
    /// `FUN_00195a30` (> 0 alive).
    pub health: f32,
    /// +0x94 roll, +0x98 yaw, +0x9c pitch, +0xa0 lift: the flight inputs.
    pub roll: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub lift: f32,
    /// +0x74: stepped −2 per tick while falling, +5 after the impact [M 0x139be8 case 16].
    pub spin: f32,
    /// +0xa8 byte: 1 every state-machine tick, 0 while a path drives the body [M].
    pub a8: bool,
    /// +0x48 → +0x28: the weapon object (the gun and the rockets fire only with one) [M].
    pub has_weapon: bool,
    /// +0x13c set (no radar object is registered at spawn, 0x13fa28) → +0x140 the blast radius
    /// of the proximity check in `FUN_0013b2a8` [M; what sets +0x13c was not traced].
    pub mine_radius: Option<f32>,
    /// +0x30 (the crash test's contact word) [M 0x139be8 case 16; its writer was not traced].
    pub contact: bool,
    /// ai +0x4c → +0xb0 / +0xb4, both written together: the gun/rocket "firing" flags [M].
    pub firing: bool,
    /// The sim object's +0x2c: half of it is the crash sweep's reach along the forward row [M
    /// 0x139be8 case 16; what the field holds was not traced].
    pub length: f32,
}

impl HeliActor {
    pub fn new(body: Body) -> HeliActor {
        HeliActor { body, health: 1.0, roll: 0.0, yaw: 0.0, pitch: 0.0, lift: 0.0, spin: 0.0, a8: false, has_weapon: true, mine_radius: None, contact: false, firing: false, length: 0.0 }
    }
}

/// The pose the scripted path's animation player gives this tick (`FUN_001b18a8` rows,
/// `FUN_001b1898` position, path object +0x88 velocity), already multiplied by the follower's
/// +0x70 placement (`FUN_002cbb88`) [M 0x139380 case 5]. The EAGL track playback itself
/// (`FUN_001b1c20` / `FUN_001b1258`) is not ported.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathPose {
    pub side: [f32; 3],
    pub up: [f32; 3],
    pub fwd: [f32; 3],
    pub pos: [f32; 3],
    pub vel: [f32; 3],
}

/// The world the controller asks.
pub trait HeliEnv {
    /// The ground height under `p` (`FUN_002322c8(·, p, 1)` then `FUN_002327a0(·, p, 0)`).
    fn ground(&mut self, p: [f32; 3]) -> f32;
    /// The +0x9c target's body (`FUN_0013dfc8(+0x9c)` → `FUN_001ff088`).
    fn target(&self) -> Body;
    /// `DAT_00339f4c`, ticks per second.
    fn fps(&self) -> i32 {
        60
    }
    /// The scripted path (mode 5): `None` when the controller has no path object (+0x50).
    fn path_pose(&mut self) -> Option<PathPose>;
    /// Navigator +0x54: the carrot (+0x20), the travel direction (+0x30), the road id (+0x3e).
    fn nav_carrot(&self) -> [f32; 3];
    fn nav_dir(&self) -> [f32; 3];
    fn nav_road(&self) -> u16;
    /// `FUN_00225c48(nav, pos, dir)`: re-join the network at `pos` heading `dir`.
    fn nav_rejoin(&mut self, pos: [f32; 3], dir: [f32; 3]);
    /// `FUN_00225358(nav)`.
    fn nav_reset(&mut self);
    /// `FUN_00224e00(dist, nav, dir, road)` (`road` = −1 → `None`).
    fn nav_advance(&mut self, dist: f32, dir: [f32; 3], road: Option<u16>);
    /// `FUN_00135018(target, dir)` → +0x3e: the road the target is on.
    fn target_road(&mut self, dir: [f32; 3]) -> u16;
    /// `FUN_00226c88(nav, road, out)` returned 0 with `out[0] != 0`: a route to `road` differs
    /// from the one held.
    fn route_differs(&mut self, road: u16) -> bool;
    /// Navigator +0x8c, the yaw probe of mode 3: `nav[0] = 5`, `FUN_00225c48(nav, pos, fwd, 1)`,
    /// then its carrot +0x20 [M 0x13b4f8].
    fn probe_carrot(&mut self, pos: [f32; 3], fwd: [f32; 3]) -> [f32; 3];
    /// `FUN_002118b8` + `FUN_002127f0`: the crash sweep from `a` to `b` hits something.
    fn crash_hit(&mut self, a: [f32; 3], b: [f32; 3]) -> bool;
    /// `FUN_001a2d10(weapon, channel)` ≠ 0: the launcher is still busy.
    fn launcher_busy(&self, channel: u8) -> bool;
}

/// What a tick produced besides the actor values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HeliEvent {
    /// `FUN_001a27a0(weapon, 9, 10, now, 2)`: one gun shot along +0xf0.
    Gun { aim: [f32; 3] },
    /// Launcher `channel` (8 or 7) fires at `target` (+0x110) [M 0x138b88].
    Rocket { channel: u8, target: [f32; 3] },
    /// The impact blast at `pos` (state 16) or the proximity blast (`FUN_0013b2a8`).
    Blast { pos: [f32; 3] },
    /// `FUN_0012da78`: the element is despawned by its actor.
    Despawn,
}

/// The AIHelicopter controller object (0x130 bytes, 16 in the pool `FUN_0013e688`).
#[derive(Debug, Clone, PartialEq)]
pub struct Heli {
    /// +0x58 attached (0x13fa28), +0x5c commanded (0x145ff8 after the apply) [M].
    pub attached: bool,
    pub commanded: bool,
    /// +0x74 movement mode, +0x78 hold-the-goal flag, +0x7c state.
    pub mode: u32,
    pub hold: u32,
    pub state: u32,
    /// +0x80 the speed factor (AICo +0x5c), +0x84 the speed cap (AICo +0x60).
    pub speed_k: f32,
    pub speed_cap: f32,
    /// +0x90 the post point (states 9/10), +0xa0 the goal.
    pub post: [f32; 3],
    pub goal: [f32; 3],
    /// +0xac the distance to the goal, +0xb0 the direction to the face point, +0xc0 its flat
    /// direction.
    pub dist: f32,
    pub dir: [f32; 3],
    pub flat_dir: [f32; 3],
    /// +0xbc the arrive radius (|AICo +0x4c|, at least 5); +0xdd its sign was positive.
    pub radius: f32,
    pub lead: bool,
    /// +0xd4 the target's road (−1 = 0xffff).
    pub road: u16,
    /// +0xd6 the hold-down counter: while non-zero the flight control flies with no speed and no
    /// yaw boost and counts it down (0xff at reset, 0 after a command) [M].
    pub hold_down: u8,
    /// +0xdc rockets left (`(char)(int)` AICo +0x68; 0xff for a state-10 command with none).
    pub rockets: u8,
    /// +0xde the death counter.
    pub dying: u8,
    /// +0xf0 the gun aim.
    pub gun_aim: [f32; 3],
    /// +0xfe the launcher toggle, +0x100 the rocket timer, +0x102 the gun timer (ticks).
    pub launcher: u8,
    pub rocket_timer: i16,
    pub gun_timer: i16,
    /// +0x110 the last rocket's target.
    pub rocket_target: [f32; 3],
    /// +0x11c the altitude above the ground (at least 3), +0x120 seconds between shots.
    pub altitude: f32,
    pub interval: f32,
    /// +0x50 holds a path object (mode 5) [M 0x13dfd8].
    pub has_path: bool,
    /// +0xfd: set when the fly-by applied its lane offset this tick (states 6-8) [M].
    pub fd: bool,
}

impl Default for Heli {
    fn default() -> Heli {
        Heli::reset()
    }
}

impl Heli {
    /// `FUN_001382f0` 0x1382f0, the reset [M]: state 1, mode 1, everything zero, +0xd4 = −1,
    /// +0xd6 = 0xff.
    pub fn reset() -> Heli {
        Heli {
            attached: false,
            commanded: false,
            mode: mode::NONE,
            hold: 0,
            state: state::IDLE,
            speed_k: 0.0,
            speed_cap: 0.0,
            post: [0.0; 3],
            goal: [0.0; 3],
            dist: 0.0,
            dir: [0.0; 3],
            flat_dir: [0.0; 3],
            radius: 0.0,
            lead: false,
            road: 0xffff,
            hold_down: 0xff,
            rockets: 0,
            dying: 0,
            gun_aim: [0.0; 3],
            launcher: 0,
            rocket_timer: 0,
            gun_timer: 0,
            rocket_target: [0.0; 3],
            altitude: 0.0,
            interval: 0.0,
            has_path: false,
            fd: false,
        }
    }

    /// `FUN_00138fa8` 0x138fa8, an AICo record applied to the helicopter [M]: state = +0x7a
    /// (10 when 0), +0x78 = AICo +0x78, mode = +0x79 when non-zero (5 attaches the +0x80 path),
    /// +0x80/+0x84 = +0x5c/+0x60, rockets `(char)(int)` +0x68 (0 in state 10 → 0xff), the point
    /// +0x40 → +0x90 in states 9/10 else the goal; state 2 forces mode 1 and puts the body at
    /// the goal with the command's placement rows; mode 1 → 2; the radius `|+0x4c|` ≥ 5 with
    /// +0xdd = (+0x4c ≥ 0); altitude `max(+0x58, 3)`; interval +0x64; both timers 0; +0xd6 = 0.
    /// Mode 3 re-joins the road at the body (`FUN_00225c48`, the caller's); the firing flags
    /// are cleared. Returns the body after a state-2 placement. `path` = whether +0x80 names
    /// a path object.
    pub fn apply_command(&mut self, c: &AiCommand, actor: &mut HeliActor, path: bool) {
        self.hold_down = 0;
        self.state = 10;
        self.hold = c.target_mode as u32;
        if c.profile != 0 {
            self.state = c.profile as u32;
        }
        if c.pattern != 0 {
            self.mode = c.pattern as u32;
            if c.pattern == 5 {
                self.has_path = path;
            }
        }
        self.speed_k = c.catchup;
        self.speed_cap = c.speed_cap;
        let r = c.ram_chance as i32 as u8;
        self.rockets = r;
        if r == 0 && self.state == state::ROCKET_POST {
            self.rockets = 0xff;
        }
        if matches!(self.state, state::GUN_POST | state::ROCKET_POST) {
            self.post = c.goal;
        } else {
            self.goal = c.goal;
        }
        if self.state == state::PARKED {
            self.mode = mode::NONE;
            actor.body.pos = self.goal;
            let m = c.placement;
            actor.body.side = [m[0][0], m[0][1], m[0][2]];
            actor.body.up = [m[1][0], m[1][1], m[1][2]];
            actor.body.fwd = [m[2][0], m[2][1], m[2][2]];
        }
        if self.mode == mode::NONE {
            self.mode = mode::HOVER;
        }
        actor.a8 = true;
        self.radius = c.follow;
        self.lead = true;
        if c.follow < 0.0 {
            self.lead = false;
            self.radius = -c.follow;
        }
        self.radius = self.radius.max(5.0);
        // `(int)x * (3.0 < x) | 3.0 * (3.0 >= x)` [M].
        self.altitude = if 3.0 < c.altitude { c.altitude } else { 3.0 };
        self.gun_timer = 0;
        self.interval = c.fire_rate;
        self.rocket_timer = 0;
        actor.firing = false;
        // `FUN_00145ff8` sets +0x5c right after the apply [M 0x145ff8 `piVar5[0x17] = 1`].
        self.commanded = true;
    }

    /// `FUN_0013b468` 0x13b468: commanded, attached, mode 5 with a path [M].
    pub fn set_path(&mut self) {
        self.commanded = true;
        self.mode = mode::PATH;
        self.attached = true;
        self.has_path = true;
    }

    /// `FUN_0013b498` 0x13b498: mode 1, state 1, no target handle, no path [M].
    pub fn release(&mut self) {
        self.mode = mode::NONE;
        self.state = state::IDLE;
        self.has_path = false;
    }

    /// `FUN_00138ef0` 0x138ef0, one tick [M]: only attached and commanded; outside states 2, 9,
    /// 10 with +0x78 clear the goal tracks the target's position; the death check, then the
    /// state machine.
    pub fn update(&mut self, a: &mut HeliActor, env: &mut dyn HeliEnv, rng: &mut DriveRng, ev: &mut Vec<HeliEvent>) {
        if !(self.attached && self.commanded) {
            return;
        }
        if !matches!(self.state, 2 | 9 | 10) && self.hold == 0 {
            self.goal = env.target().pos;
        }
        if !self.death_check(a, env, ev) {
            self.step(a, env, rng, ev);
        }
    }

    /// `FUN_0013b2a8` 0x13b2a8 [M]: with +0xde clear, a proximity mine (+0x13c) within its
    /// radius of the target's box (`|p − pos| − box_z ≤ +0x140`) blasts and counts +0xde;
    /// with +0xde set the element despawns (the helicopter's health handed to the actor's
    /// `FUN_00195a58`) and +0xde clears. True ends the tick. A crash that counts +0xde in state
    /// 16 is therefore despawned on the next tick, so state 16's own `+0xde ≠ 0` arm (the
    /// quarter-second spin) is not reached through this update [M: 0x139be8's only caller is
    /// 0x138f8c]. The proximity position is `FUN_00195808(FUN_00201ae0())`, read here as the
    /// target's [S].
    pub fn death_check(&mut self, a: &mut HeliActor, env: &mut dyn HeliEnv, ev: &mut Vec<HeliEvent>) -> bool {
        if self.dying == 0 {
            if let Some(r) = a.mine_radius {
                let t = env.target();
                if len(sub(t.pos, a.body.pos)) - t.box_z <= r {
                    ev.push(HeliEvent::Blast { pos: a.body.pos });
                    self.dying = self.dying.wrapping_add(1);
                    return true;
                }
            }
            false
        } else {
            ev.push(HeliEvent::Despawn);
            self.dying = 0;
            true
        }
    }

    /// `FUN_001385d8` 0x1385d8: +0xb0 = n(p − pos), +0xc0 = n(flat(p − pos)) [M].
    pub fn face(&mut self, pos: [f32; 3], p: [f32; 3]) {
        let d = sub(p, pos);
        self.dir = norm(d);
        self.flat_dir = norm(flat(d));
    }

    /// `FUN_00139be8` 0x139be8, the state machine [M; the arguments of the movement, face,
    /// gun and rocket calls from the disassembly 0x139ed0..0x13a1d0].
    pub fn step(&mut self, a: &mut HeliActor, env: &mut dyn HeliEnv, rng: &mut DriveRng, ev: &mut Vec<HeliEvent>) {
        if a.health <= 0.0 {
            self.state = state::DEAD;
            a.firing = false;
        }
        a.a8 = true;
        self.fd = false;
        match self.state {
            1 | 4 => self.movement(a, env, rng, true, false),
            11 => {
                self.hold = 1;
                self.movement(a, env, rng, false, false);
            }
            2 => {
                a.body.pos = self.goal;
                a.body.vel = [0.0; 3];
            }
            3 => {
                let t = env.target();
                self.goal = if self.lead { add(t.pos, scale(t.vel, self.radius + self.radius)) } else { t.pos };
                self.movement(a, env, rng, true, false);
                if self.rockets != 0 && (self.radius < self.dist || !self.lead) {
                    let fwd = a.body.fwd;
                    self.gun(65.0, a, a.body.pos, t.pos, fwd, env, rng, ev);
                    return;
                }
                a.firing = false;
            }
            5 => {
                self.movement(a, env, rng, false, true);
                let t = env.target();
                self.face(a.body.pos, t.pos);
                a.pitch = dot(self.flat_dir, norm(flat(a.body.fwd)));
            }
            9 | 10 => {
                self.movement(a, env, rng, true, false);
                let fwd = a.body.fwd;
                let side = a.body.side;
                let own = a.body.pos;
                let t = env.target();
                let at = if self.hold == 0 { t.pos } else { self.post };
                self.face(own, at);
                let d = dot(self.flat_dir, norm(flat(side)));
                a.pitch = d;
                let aim = if d.abs() < 0.2 { self.dir } else { fwd };
                let post = self.post;
                if self.state != 9 {
                    self.rockets_fire(1000.0, 0.0, a, own, post, aim, env, ev);
                } else {
                    self.gun(1000.0, a, own, post, aim, env, rng, ev);
                }
            }
            16 => self.crash(a, env, ev),
            // 6/7/8: not ported (see the module doc); 12..15: no arm.
            _ => {}
        }
    }

    /// `FUN_00139380` 0x139380, the movement [M; `yaw_boost` = its a1 (the flight control's
    /// param_5 in mode 3), `blend` = its a2 — disassembly 0x1393a0 `move s2, a1`, 0x139394
    /// `move s3, a2`].
    pub fn movement(&mut self, a: &mut HeliActor, env: &mut dyn HeliEnv, rng: &mut DriveRng, yaw_boost: bool, blend: bool) {
        let pos = a.body.pos;
        match self.mode {
            mode::HOVER => {
                self.dist = len(sub(pos, self.goal));
                let g = self.goal;
                self.face(pos, g);
                // `min(max(dist/radius·0.75, 1.01)·50, 50)` is 50 whenever it is computed [M].
                let s = ((self.dist / self.radius) * 0.75).max(1.01) * 50.0;
                let mut s = s.min(50.0);
                if self.dist < self.radius {
                    s = 0.0;
                }
                let s = (s * self.speed_k).min(self.speed_cap);
                let mut goal = self.goal;
                self.flight(a, env, rng, 0.0, s, &mut goal, false);
                a.pitch = dot(self.flat_dir, norm(flat(a.body.side)));
            }
            mode::ROAD => {
                self.dist = len(sub(pos, self.goal));
                let g = self.goal;
                self.face(pos, g);
                let t = env.target();
                let tv = len(t.vel);
                let speed = if self.hold == 0 && self.state == state::CHASE {
                    if self.dist <= self.radius { 0.0f32.min(self.speed_cap) } else { (tv * self.speed_k).min(self.speed_cap) }
                } else if self.hold == 0 && !matches!(self.state, 9 | 10) {
                    if self.dist < self.radius {
                        let s = if 0.0 < dot(t.vel, self.flat_dir) { tv } else { 0.0 };
                        s.min(self.speed_cap)
                    } else {
                        (tv * self.speed_k).max(25.0).min(self.speed_cap)
                    }
                } else if self.dist < self.radius {
                    0.0
                } else {
                    self.speed_cap
                };
                let adv = heli_advance(a.body.speed());
                let carrot = env.nav_carrot();
                let dcar = len(sub(pos, carrot));
                let nav_dir = env.nav_dir();
                let road = env.target_road(nav_dir);
                self.road = road;
                let mut rejoin = false;
                if self.radius <= self.dist {
                    if env.nav_road() == road {
                        rejoin = dot(self.flat_dir, nav_dir) < 0.0;
                    } else {
                        rejoin = env.route_differs(road);
                    }
                }
                if rejoin {
                    env.nav_rejoin(pos, nav_dir);
                    env.nav_reset();
                    let d = env.nav_dir();
                    env.nav_advance(adv, d, None);
                }
                if dcar < adv {
                    let fd = self.flat_dir;
                    env.nav_advance(adv + adv - dcar, fd, Some(self.road));
                }
                let mut goal = env.nav_carrot();
                let alt = self.altitude;
                self.flight(a, env, rng, alt, speed, &mut goal, yaw_boost);
                let side = norm(flat(a.body.side));
                if blend && self.dist < self.radius {
                    a.pitch = dot(self.flat_dir, scale(add(a.body.fwd, side), 0.5));
                    return;
                }
                a.pitch = dot(self.flat_dir, side);
            }
            mode::PATH => {
                if self.state == state::DEAD || !self.has_path {
                    return;
                }
                let Some(p) = env.path_pose() else { return };
                a.a8 = false;
                a.body.side = p.side;
                a.body.up = p.up;
                a.body.fwd = p.fwd;
                a.body.pos = p.pos;
                a.body.vel = p.vel;
            }
            _ => {}
        }
    }

    /// `FUN_0013b4f8` 0x13b4f8, the flight control [M; the side row is `FUN_001fd5b8` (the
    /// quaternion's first matrix row, `FUN_002cc230`), the forward row `FUN_001fd598`
    /// (`FUN_002cc350`, the third) — decoded from the VU code; the lateral velocity is the
    /// first component of the transposed matrix (`FUN_001fd548`) times the velocity, i.e.
    /// `dot(side, v)`]:
    ///
    /// - `+0xd6 ≠ 0`: no speed, no yaw boost, +0xd6 − 1;
    /// - `alt ≠ 0` puts the goal height at the ground; then `goal.y += alt`;
    /// - yaw input: in mode 3 outside state 5 the probe navigator's carrot (re-joined at the
    ///   body along the forward row) `dot(carrot − pos, side_flat)`, else `dot(flat(goal −
    ///   pos), side_flat)`; with `max > 0` (`__extendsfdf2` + `__gtdf2`) and the boost: yaw =
    ///   `8·(input − 0.5·dot(side, v))`; else state 5: `input` while slower than `max`, else 0;
    ///   other states: `−dot(side, v)`;
    /// - higher than 2 above the ground: roll steps 0.02 toward `dot(n(flat goal delta), fwd)
    ///   · max/50` while slower than `max`, else 0;
    /// - lift `2·(min(goal.y − y, 10) − v.y) + 9.8`, × `clamp(rand·4/65536, 0.5, 1)` higher
    ///   than 2 above the ground (one draw, always).
    #[allow(clippy::too_many_arguments)]
    pub fn flight(&mut self, a: &mut HeliActor, env: &mut dyn HeliEnv, rng: &mut DriveRng, alt: f32, max: f32, goal: &mut [f32; 3], boost: bool) {
        let pos = a.body.pos;
        let ground = env.ground(pos);
        let (mut max, mut boost) = (max, boost);
        if self.hold_down != 0 {
            max = 0.0;
            self.hold_down = self.hold_down.wrapping_sub(1);
            boost = false;
        }
        if alt != 0.0 {
            goal[1] = ground;
        }
        goal[1] += alt;
        let delta = flat(sub(*goal, pos));
        let dn = norm(delta);
        let v = a.body.vel;
        let flat_speed = len(flat(v));
        let fwd = a.body.fwd;
        let side_flat = norm(flat(a.body.side));
        let roll_dot = dot(dn, fwd);
        let input = if self.state != state::CHASE && self.mode == mode::ROAD {
            let c = env.probe_carrot(pos, fwd);
            dot(sub(c, pos), side_flat)
        } else {
            dot(delta, side_flat)
        };
        let lat = dot(a.body.side, v);
        a.yaw = if 0.0 < max && boost {
            let f = input - lat * 0.5;
            (f + f) * 4.0
        } else if self.state == state::CHASE {
            if flat_speed < max { input } else { 0.0 }
        } else {
            -lat
        };
        let height = pos[1] - ground;
        if 2.0 < height {
            a.roll = if flat_speed < max { approach(a.roll, roll_dot * (max / 50.0), f32::from_bits(0x3ca3_d70a)) } else { 0.0 };
        }
        let e = (goal[1] - pos[1]).min(10.0);
        let k = (rng.next() as i32 as f32 * RAND_SCALE * 4.0).max(0.5).min(1.0);
        let f = e - v[1];
        let mut lift = f + f + 9.8;
        if 2.0 < height {
            lift *= k;
        }
        a.lift = lift;
    }

    /// `FUN_001387a0` 0x1387a0, the gun [M; arguments from the disassembly 0x139f4c /
    /// 0x13a184: `from` = own position, `to` = the target (state 3) or the post (state 9),
    /// `aim` = the forward row (or +0xb0 when nearly facing)]: alive and `+0x120 ≥ 0`; the
    /// +0x102 timer counts down to 0 first; within `range`: +0xf0 = n(aim + 5·n(to − from));
    /// inside the 0.707 cone (`dot(n(to − from), aim)`) and with a weapon: the spread (three
    /// draws — aim.y > −0.01: x += (r − 0.5)·0.025, y −= r·0.15, z += (r − 0.5)·0.025; else each
    /// ± (r − 0.5)·0.05), the shot, the timer `(short)(int)(fps·interval)`; the firing flags =
    /// inside the cone.
    #[allow(clippy::too_many_arguments)]
    pub fn gun(&mut self, range: f32, a: &mut HeliActor, from: [f32; 3], to: [f32; 3], aim: [f32; 3], env: &mut dyn HeliEnv, rng: &mut DriveRng, ev: &mut Vec<HeliEvent>) {
        if !(0.0 < a.health && 0.0 <= self.interval) {
            return;
        }
        if 0 < self.gun_timer {
            self.gun_timer = (self.gun_timer - 1).max(0);
            return;
        }
        if !(len(sub(from, to)) < range) {
            return;
        }
        let d = norm(sub(to, from));
        self.gun_aim = norm(add(aim, scale(d, 5.0)));
        if 0.707 < dot(d, aim) {
            if a.has_weapon {
                let mut r = || rng.next() as i32 as f32 * RAND_SCALE;
                if -0.01 < aim[1] {
                    self.gun_aim[0] += (r() - 0.5) * 0.025;
                    self.gun_aim[1] -= r() * 0.15;
                    self.gun_aim[2] += (r() - 0.5) * 0.025;
                } else {
                    self.gun_aim[0] += (r() - 0.5) * 0.05;
                    self.gun_aim[1] += (r() - 0.5) * 0.05;
                    self.gun_aim[2] += (r() - 0.5) * 0.05;
                }
                ev.push(HeliEvent::Gun { aim: self.gun_aim });
                self.gun_timer = short_of(env.fps() as f32 * self.interval);
            }
            a.firing = true;
        } else {
            a.firing = false;
        }
    }

    /// `FUN_00138b88` 0x138b88, the rockets [M; `speed` = f13 (0 from the posts, the own speed
    /// in the fly-by)]: not in state 10 with +0xdc = 0; alive, `+0x120 ≥ 0`; the +0x100 timer
    /// first; within `range` and (`speed ≤ 5` or aim.y ≤ −0.05): inside the 0.8434 cone of the
    /// flattened aim and direction, launcher 8 (+0xfe = 0) or 7 fires when free, aiming at
    /// `to` (+0x110); then +0xdc − 1, the timer `(short)(int)max(fps·interval, fps·0.5)`, the
    /// launcher toggles. The firing flags are not touched.
    #[allow(clippy::too_many_arguments)]
    pub fn rockets_fire(&mut self, range: f32, speed: f32, a: &mut HeliActor, from: [f32; 3], to: [f32; 3], aim: [f32; 3], env: &mut dyn HeliEnv, ev: &mut Vec<HeliEvent>) {
        if self.state == state::ROCKET_POST && self.rockets == 0 {
            return;
        }
        if !(0.0 < a.health && 0.0 <= self.interval) {
            return;
        }
        if 0 < self.rocket_timer {
            self.rocket_timer = (self.rocket_timer - 1).max(0);
            return;
        }
        let dist = len(sub(from, to));
        if !(dist < range && (speed <= 5.0 || aim[1] <= -0.05)) {
            return;
        }
        let d = norm(flat(sub(to, from)));
        let aim_n = norm(flat(aim));
        if !(0.8434 < dot(d, aim_n)) {
            return;
        }
        let mut fired = false;
        if a.has_weapon {
            let ch = if self.launcher == 0 { 8 } else { 7 };
            if !env.launcher_busy(ch) {
                self.rocket_target = to;
                ev.push(HeliEvent::Rocket { channel: ch, target: to });
                fired = true;
            }
        }
        if fired {
            let fps = env.fps() as f32;
            self.rockets = self.rockets.wrapping_sub(1);
            self.rocket_timer = short_of((fps * self.interval).max(fps * 0.5));
            self.launcher = (self.launcher + 1) & 1;
        }
    }

    /// State 16 of `FUN_00139be8` [M]: with +0xde clear — mode 1; pitch `max(p − 0.1, −1)`,
    /// roll `min(r + 0.01, 1)`, lift `min(l·0.99, 0.25)`, spin − 2; when airborne (no contact
    /// or more than 1 above the ground) the sweep along `±n(fwd)·box_z/2` (`FUN_002127f0`) or
    /// the front end at or below the ground is the impact; on the ground with contact it is the
    /// impact at once. The impact blasts at the body (`vtbl+0x1c(1000, 0.5, …)`), counts +0xde,
    /// scales the velocity by 0.35 and zeroes the roll. With +0xde set (not reached through the
    /// update, see `death_check`): pitch − 0.01, spin + 5, after `fps/4` ticks the despawn.
    pub fn crash(&mut self, a: &mut HeliActor, env: &mut dyn HeliEnv, ev: &mut Vec<HeliEvent>) {
        if self.dying == 0 {
            self.mode = mode::NONE;
            a.pitch = (a.pitch - 0.1).max(-1.0);
            a.roll = (a.roll + 0.01).min(1.0);
            a.lift = (a.lift * 0.99).min(0.25);
            a.spin -= 2.0;
            let pos = a.body.pos;
            let ground = env.ground(pos);
            let impact = if !a.contact || ground < pos[1] - 1.0 {
                let f = scale(norm(a.body.fwd), a.length * 0.5);
                let front = add(pos, f);
                let back = sub(pos, f);
                env.crash_hit(front, back) || front[1] <= ground
            } else {
                true
            };
            if !impact {
                return;
            }
            ev.push(HeliEvent::Blast { pos });
            self.dying = self.dying.wrapping_add(1);
            a.body.vel = scale(a.body.vel, f32::from_bits(0x3eb3_3333));
            a.roll = 0.0;
        } else {
            a.pitch = (a.pitch - 0.01).max(-1.0);
            a.spin += 5.0;
            self.dying = self.dying.wrapping_add(1);
            let fps = env.fps();
            let q = (if fps < 0 { fps + 3 } else { fps }) >> 2;
            if (self.dying as i8 as i32) < q {
                return;
            }
            ev.push(HeliEvent::Despawn);
            self.dying = 0;
        }
    }
}
