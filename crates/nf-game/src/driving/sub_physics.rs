//! The SUB_PHYSICS vehicles of the driving missions: the submarine (`vanquishsub`, MIS11) and the
//! ultralights (`ultralight` MIS13A/B, `ultralightbig` MIS13C), plus the per-vehicle pass
//! dispatcher. docs/systems/drive-physics.md §8-§9 is the write-up and the function table.
//!
//! Evidence [M] = DRIVING.ELF (decompile under `work/decomp/DRIVING.ELF/`, raw bytes for the
//! VU0 microcode), [D] = disc data, [S] = the port's own reading, stated where used.
//!
//! What the game does, in short:
//! - `FUN_00188200` picks ONE pass per vehicle per tick [M 0x188bf8-0x188c60]: `SUB_PHYSICS`
//!   (attr +0xc0) != 0 → `FUN_00190f88`; else `IS_SNOWMOBILE` (+0xc8) → `FUN_00190510`; else the
//!   player (+0x2c8 == 0) → `FUN_0018cef8`, an AI → `FUN_0018ecb8`. A path-driven vehicle takes
//!   `FUN_00192498` instead, which returns at once for SUB_PHYSICS [M 0x192498:86].
//! - The sub and the ultralights run the SAME pass, `FUN_00190f88`: no code there reads
//!   `IS_SUB` or the (unregistered) `IS_FLYING` key [M; the attr schema `FUN_001968f8` registers
//!   IS_4X4, IS_RALLY, IS_SUB, IS_SNOWMOBILE, NO_WORLD_COLLISIONS, SUB_PHYSICS, IS_BOAT only].
//! - SUB_PHYSICS == 1 bodies get NO gravity: `FUN_001f7c60` skips its gravity force for them
//!   [M 0x1f7c60: the `attr+0xc0 == 1 → goto` past `FUN_001f87d8(gravity)`]. There is no
//!   buoyancy and no water surface in the pass: height changes come from the pitch stick (the
//!   axle forces along the body's up axis), thrust along the nose, the sheets, and a small
//!   position bob [M 0x191ff8-0x1920c0].
//! - The integration is the rigid-body VU0 microprogram at micro 0xad (`FUN_001f89e0` →
//!   `vcallms 0xad`; the program is the `.vutext` overlays at EE 0x324e70 (VU 0x000, 0x800
//!   bytes) + 0x325678 (VU 0x800, 0x2f8 bytes)) [M, disassembled with tools/vu1dis.py]. It is
//!   ported here op for op in `Body::integrate`.

use super::bounds::{ZoneQuad, ZONES};
use super::physics::{Car, CarInput, CarParams, FlatWorld, Rigid};

/// The integer frame rate the sheet response reads (sim+0xdbc = 0x339f4c, `FUN_001feda0` stores
/// `FUN_002d6b90()` = 60 on NTSC) [M].
pub const FPS: f32 = 60.0;

// ---------------------------------------------------------------------------
// The game's trig (angles in TURNS) and the input slew
// ---------------------------------------------------------------------------

fn wrap_half(mut x: f32) -> f32 {
    // [M 0x2cbd50 head] keep the fraction (an int part off for x > 1 or x < 0), then fold to
    // [-0.5, 0.5].
    if 1.0 < x {
        x -= (x as i32) as f32;
    } else if x < 0.0 {
        x -= ((x as i32) - 1) as f32;
    }
    if x < -0.5 {
        x += 1.0;
    } else if 0.5 < x {
        x -= 1.0;
    }
    x
}

fn sin_poly(t: f32) -> f32 {
    // [M 0x2cbd50 tail] t + c3 t^3 + c5 t^5 + c7 t^7 + c9 t^9, the four literals as stored.
    let c3 = f32::from_bits(0xbe2a_aaa4);
    let c5 = f32::from_bits(0x3c08_873e);
    let c7 = f32::from_bits(0xb94f_b21f);
    let c9 = f32::from_bits(0x362e_9c14);
    let t2 = t * t;
    let t3 = t2 * t;
    let t5 = t3 * t2;
    let t7 = t5 * t2;
    let t9 = t7 * t2;
    t + t3 * c3 + t5 * c5 + t7 * c7 + t9 * c9
}

/// `FUN_002cbd50`: sin of an angle in turns (1.0 = 360°) [M].
pub fn sin_turns(x: f32) -> f32 { sin_poly(wrap_half(x) * 6.283_185_5) }

/// `FUN_002cbeb0`: cos of an angle in turns — the sine path with +0.25 added after the fraction
/// step [M 0x2cbeb0].
pub fn cos_turns(x: f32) -> f32 {
    let mut x = x;
    if 1.0 < x {
        x -= (x as i32) as f32;
    } else if x < 0.0 {
        x -= ((x as i32) - 1) as f32;
    }
    x += 0.25;
    if x < -0.5 {
        x += 1.0;
    } else if 0.5 < x {
        x -= 1.0;
    }
    sin_poly(x * 6.283_185_5)
}

/// `FUN_002104a8(cur, target, step)`: move `cur` toward `target` by at most `step` [M]. The pad
/// layer `FUN_0018add8` slews the steer (+0x240) and pitch (+0x244) inputs with step 0.25 per tick
/// [M 0x18add8 tail, the 0x3e800000 literal].
pub fn approach(cur: f32, target: f32, step: f32) -> f32 {
    if target < cur {
        let v = cur - step;
        if v < target { target } else { v }
    } else {
        let v = cur + step;
        if target < v { target } else { v }
    }
}

// ---------------------------------------------------------------------------
// Small vector helpers (xyz)
// ---------------------------------------------------------------------------

type V3 = [f32; 3];
/// Rows = the body's local x (right), y (up), z (nose) axes in world space — the layout of the
/// orientation block at `*(inst+0x5c)` [M FUN_001f84d0 / FUN_001f82e8; FUN_00210540 builds row 2
/// from the heading, row 3 = (0,0,0,1)].
pub type Rows = [V3; 3];

fn dot(a: V3, b: V3) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn cross(a: V3, b: V3) -> V3 { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }
fn add(a: V3, b: V3) -> V3 { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }
fn scale(a: V3, s: f32) -> V3 { [a[0] * s, a[1] * s, a[2] * s] }
fn norm(a: V3) -> V3 {
    let l = dot(a, a).sqrt();
    if l > 0.0 { scale(a, 1.0 / l) } else { a }
}
/// local → world: `r0·v.x + r1·v.y + r2·v.z` [M FUN_001f84d0; the row-3 term adds 0].
fn to_world(m: &Rows, v: V3) -> V3 { add(add(scale(m[0], v[0]), scale(m[1], v[1])), scale(m[2], v[2])) }
/// world → local: `(r0·v, r1·v, r2·v)` [M FUN_001f82e8 / FUN_001f8380 / FUN_001f8250 / FUN_001f8508].
fn to_local(m: &Rows, v: V3) -> V3 { [dot(m[0], v), dot(m[1], v), dot(m[2], v)] }

// ---------------------------------------------------------------------------
// The pvehicle attributes this layer reads
// ---------------------------------------------------------------------------

/// The pvehicle attributes the pass choice and the SUB_PHYSICS pass read (the schema is
/// `FUN_001968f8` [M]; offsets in the attr block at actor+0x23c).
#[derive(Clone, Debug, PartialEq)]
pub struct VehicleAttrs {
    /// `MASS` → inst+0x4c [M FUN_001862c0:249-257]. Schema default 2000.
    pub mass: f32,
    /// `IS_SUB` +0xbc [M 0x1968f8:261]: read by the audio/effects code, not by the pass.
    pub is_sub: i32,
    /// `SUB_PHYSICS` +0xc0 [M 0x1968f8:273].
    pub sub_physics: i32,
    /// `NO_WORLD_COLLISIONS` +0xc4 [M 0x1968f8:269].
    pub no_world_collisions: i32,
    /// `IS_SNOWMOBILE` +0xc8 [M 0x1968f8:265].
    pub is_snowmobile: i32,
    /// `IS_BOAT` +0xcc [M 0x1968f8:277]: in `FUN_001f7c60` it scales gravity by 0.2 and adds a
    /// horizontal drag (−v) for a non-SUB_PHYSICS boat (the patrol boat) — the land layer's.
    pub is_boat: i32,
    /// `BOOST_EXTRA_ACC` (read by name at run time, `FUN_00169d80(attr, …, 0)`, default 0) [M].
    pub boost_extra_acc: f32,
    /// `BOOST_TIME` (`FUN_00169bf0(attr, …, 0)`, default 0) [M].
    pub boost_time: i32,
}

impl VehicleAttrs {
    /// The registered defaults: every flag 0 [M the `FUN_0016ec80(…, 0, 1, 0)` calls after each
    /// flag], MASS 2000 [M], the run-time reads' 0 defaults.
    pub fn game_defaults() -> VehicleAttrs {
        VehicleAttrs {
            mass: 2000.0,
            is_sub: 0,
            sub_physics: 0,
            no_world_collisions: 0,
            is_snowmobile: 0,
            is_boat: 0,
            boost_extra_acc: 0.0,
            boost_time: 0,
        }
    }

    /// Reads the `.atr` INI text the way `CarParams::parse_atr` does (suffixes `.f/.i/.u`
    /// stripped, `//` comments, whitespace-padded values) [D the shipped pvehicle files]. Keys
    /// the schema does not register (`IS_FLYING`, `HAS_*`) are ignored, as the game ignores them.
    pub fn parse_atr(text: &str) -> VehicleAttrs {
        let mut a = VehicleAttrs::game_defaults();
        for line in text.lines() {
            let line = line.split("//").next().unwrap_or("").trim();
            if line.is_empty() || line.starts_with('[') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else { continue };
            let key = key.trim();
            let key = key.split('.').next().unwrap_or(key).trim();
            let num: &str = value
                .trim()
                .split(|c: char| !(c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | 'e' | 'E')))
                .next()
                .unwrap_or("");
            let Ok(v) = num.parse::<f32>() else { continue };
            match key {
                "MASS" => a.mass = v,
                "IS_SUB" => a.is_sub = v as i32,
                "SUB_PHYSICS" => a.sub_physics = v as i32,
                "NO_WORLD_COLLISIONS" => a.no_world_collisions = v as i32,
                "IS_SNOWMOBILE" => a.is_snowmobile = v as i32,
                "IS_BOAT" => a.is_boat = v as i32,
                "BOOST_EXTRA_ACC" => a.boost_extra_acc = v,
                "BOOST_TIME" => a.boost_time = v as i32,
                _ => {}
            }
        }
        a
    }
}

/// The per-vehicle controller pass `FUN_00188200` runs [M 0x188bf8-0x188c60].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrivePass {
    /// `FUN_00190f88`: the SUB_PHYSICS pass (this module).
    Sub,
    /// `FUN_00190510`: IS_SNOWMOBILE (not ported).
    Snowmobile,
    /// `FUN_0018cef8`: the player's land car (`physics::Car::step` is its port).
    PlayerLand,
    /// `FUN_0018ecb8`: an AI land car (`ai::AiDriver`).
    AiLand,
    /// `FUN_00192498`: the path-driven branch (vtbl+0x5c object set and `FUN_0013e0b0` != 0 and
    /// alive). For a SUB_PHYSICS vehicle it returns at once [M 0x192498:86] — no pass runs.
    Path,
}

/// The pass choice [M 0x188b70-0x188c60]: `driver` = actor+0x2c8 (0 = the player),
/// `path_driven` = the 0x188b70 test (`vtbl+0x5c` object, `FUN_0013e0b0`, alive).
pub fn drive_pass(attrs: &VehicleAttrs, driver: u32, path_driven: bool) -> DrivePass {
    if path_driven {
        return DrivePass::Path;
    }
    if attrs.sub_physics != 0 {
        DrivePass::Sub
    } else if attrs.is_snowmobile != 0 {
        DrivePass::Snowmobile
    } else if driver == 0 {
        DrivePass::PlayerLand
    } else {
        DrivePass::AiLand
    }
}

/// `FUN_00210938(mass, hx, hy, hz)`: the inverse inertia diagonal from the model's half extents
/// (`FUN_001c9778`: half the model header's max − min) [M]. A long body (hz > 4) is a box with
/// `I = m/3·(b² + c²)`; a short one uses the second form.
pub fn inv_inertia(mass: f32, half: V3) -> V3 {
    let (x, y, z) = (half[0], half[1], half[2]);
    if 4.0 < z {
        let m = mass * 0.3333;
        [1.0 / (m * (y * y + z * z)), 1.0 / (m * (x * x + z * z)), 1.0 / (m * (x * x + y * y))]
    } else {
        let r2 = x * x + y * y;
        let iz = 0.5 / (mass * 0.5 * r2);
        let ix = 0.5 / (mass * 0.08333 * (z + z) * (z + z) + mass * 0.25 * r2);
        [ix, ix, iz]
    }
}

// ---------------------------------------------------------------------------
// The rigid body and the VU0 integration
// ---------------------------------------------------------------------------

/// One rigid body: the sim instance fields the pass and the integrator touch.
#[derive(Clone, Debug)]
pub struct Body {
    /// inst+0x00: the orientation quaternion (x, y, z, w) [M FUN_001f8cf0 stores vf20 there].
    pub quat: [f32; 4],
    /// The rows rebuilt from `quat` (class+0x00..+0x2f) [M micro 0x110-0x12e].
    pub rows: Rows,
    /// inst+0x10 position.
    pub pos: V3,
    /// inst+0x20 velocity.
    pub vel: V3,
    /// inst+0x30 angular velocity (world).
    pub ang_vel: V3,
    /// inst+0x50 angular momentum (world).
    pub ang_mom: V3,
    /// inst+0x4c mass (also the momentum's w lane, the `DIV vf00.w, vf24.w` at micro 0xaf).
    pub mass: f32,
    /// The w lanes of inst+0x10/+0x20/+0x30: the local inverse inertia [M FUN_001f6e98:136-140;
    /// micro 0x136-0x13c].
    pub inv_inertia: V3,
    /// class+0x40..+0x6f: the world inverse inertia `R·diag·Rᵀ` [M micro 0x13d-0x14f].
    pub inv_inertia_world: Rows,
    /// inst+0x60: the force accumulator (acceleration × mass).
    pub force: V3,
    /// inst+0x70: the torque accumulator.
    pub torque: V3,
}

fn quat_rows(q: [f32; 4]) -> Rows {
    // [M micro 0x110-0x12e]: vf04..vf07 = 2q·q.{x,y,z,w}; the three rows as built there.
    let (x, y, z, w) = (q[0], q[1], q[2], q[3]);
    [
        [1.0 - 2.0 * y * y - 2.0 * z * z, 2.0 * x * y + 2.0 * z * w, 2.0 * x * z - 2.0 * y * w],
        [2.0 * x * y - 2.0 * z * w, 1.0 - 2.0 * x * x - 2.0 * z * z, 2.0 * y * z + 2.0 * w * x],
        [2.0 * x * z + 2.0 * y * w, 2.0 * y * z - 2.0 * w * x, 1.0 - 2.0 * x * x - 2.0 * y * y],
    ]
}

fn world_inertia(rows: &Rows, inv: V3) -> Rows {
    // [M micro 0x13d-0x14f]: vf12..14 = r_i · inv_i; vf15..17 = Σ_i (r_i inv_i) r_i[k] —
    // symmetric, so row k = Σ_i inv_i r_i[k] r_i.
    let mut out = [[0.0f32; 3]; 3];
    for k in 0..3 {
        for i in 0..3 {
            out[k] = add(out[k], scale(rows[i], inv[i] * rows[i][k]));
        }
    }
    out
}

impl Body {
    /// A body at rest heading `yaw` (radians about +y, 0 = +z), the start pose
    /// `FUN_001fee68` → `FUN_00210540` builds from a level heading [M; docs/formats/trackstart.md].
    pub fn new(mass: f32, inv_inertia: V3, pos: V3, yaw: f32) -> Body {
        let (s, c) = (yaw * 0.5).sin_cos();
        let quat = [0.0, s, 0.0, c];
        let rows = quat_rows(quat);
        Body {
            quat,
            rows,
            pos,
            vel: [0.0; 3],
            ang_vel: [0.0; 3],
            ang_mom: [0.0; 3],
            mass,
            inv_inertia,
            inv_inertia_world: world_inertia(&rows, inv_inertia),
            force: [0.0; 3],
            torque: [0.0; 3],
        }
    }

    /// `FUN_001f87d8`: add an acceleration (stored × mass) [M].
    pub fn add_accel(&mut self, a: V3) { self.force = add(self.force, scale(a, self.mass)); }
    /// `FUN_001f8888`: add a torque × mass [M].
    pub fn add_torque_mass(&mut self, t: V3) { self.torque = add(self.torque, scale(t, self.mass)); }
    /// `FUN_001f8778`: add a torque as is [M].
    pub fn add_torque(&mut self, t: V3) { self.torque = add(self.torque, t); }
    /// `FUN_001f8168`: a local angular impulse, rotated to world and added to inst+0x50 [M].
    pub fn add_ang_impulse_local(&mut self, l: V3) { self.ang_mom = add(self.ang_mom, to_world(&self.rows, l)); }

    /// The VU0 microprogram at micro 0xad [M], op for op (the M_LIMIT / AM_LIMIT clamps from
    /// `FUN_001f89e0`'s vf05 = (−M_LIMIT·m, M_LIMIT·m, −AM_LIMIT, AM_LIMIT); vf04 = (dt, 0.5, …)):
    /// - 0xad/0xb1: momentum p (inst+0x40) clamped per axis to ±M_LIMIT·m BEFORE the force;
    /// - 0xaf/0xb5: p += F·dt; 0xae/0xb6: L += τ·dt; 0xb9: v = p/m;
    /// - 0xba-0xbc: ω = W·L (W = last tick's world inverse inertia); 0xbd/0xd5: pos += v·dt;
    /// - 0xc0/0xc4: ω clamped per axis to ±AM_LIMIT; 0xc8-0xd4: L = R·I·Rᵀ·ω from the clamped ω;
    /// - 0xd6-0x10c: q += 0.5·(ω·dt)⊗q, normalised (RSQRT) — the ×vf04.z pre-scale at 0xd7
    ///   cancels in the normalise;
    /// - 0x110-0x14f: rows from q, then W = R·diag(inv)·Rᵀ.
    /// Then `FUN_001f7c60` zeroes both accumulators (and, for SUB_PHYSICS, applies no gravity).
    pub fn integrate(&mut self, rigid: &Rigid, dt: f32) {
        let m = self.mass;
        let lim = rigid.m_limit * m;
        let mut p = scale(self.vel, m);
        for k in 0..3 {
            p[k] = p[k].max(-lim).min(lim);
        }
        p = add(p, scale(self.force, dt));
        let l = add(self.ang_mom, scale(self.torque, dt));
        self.vel = scale(p, 1.0 / m);
        let w = &self.inv_inertia_world;
        let mut om = add(add(scale(w[0], l[0]), scale(w[1], l[1])), scale(w[2], l[2]));
        self.pos = add(self.pos, scale(self.vel, dt));
        for k in 0..3 {
            om[k] = om[k].max(-rigid.am_limit).min(rigid.am_limit);
        }
        let ol = to_local(&self.rows, om);
        let inertia = [1.0 / self.inv_inertia[0], 1.0 / self.inv_inertia[1], 1.0 / self.inv_inertia[2]];
        self.ang_mom = to_world(&self.rows, [ol[0] * inertia[0], ol[1] * inertia[1], ol[2] * inertia[2]]);
        self.ang_vel = om;
        // q' = q + 0.5·(ω dt, 0)⊗q  [M 0xd6-0xf6: (q.w·a + a×q.v, −a·q.v)].
        let a = scale(om, dt);
        let qv = [self.quat[0], self.quat[1], self.quat[2]];
        let qw = self.quat[3];
        let dv = add(cross(a, qv), scale(a, qw));
        let dw = -dot(a, qv);
        let mut q = [qv[0] + 0.5 * dv[0], qv[1] + 0.5 * dv[1], qv[2] + 0.5 * dv[2], qw + 0.5 * dw];
        let n = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
        for c in &mut q {
            *c /= n;
        }
        self.quat = q;
        self.rows = quat_rows(q);
        self.inv_inertia_world = world_inertia(&self.rows, self.inv_inertia);
        self.force = [0.0; 3];
        self.torque = [0.0; 3];
    }
}

// ---------------------------------------------------------------------------
// The SUB_PHYSICS craft and FUN_00190f88
// ---------------------------------------------------------------------------

/// The pad fields `FUN_00190f88` reads (actor offsets) [M 0x190f88 head].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SubInput {
    /// +0x250
    pub throttle: f32,
    /// +0x254
    pub brake: f32,
    /// +0x240, the steer stick (yaw), already slewed by `FUN_0018add8` (see [`approach`]).
    pub steer: f32,
    /// +0x244, the pitch stick.
    pub pitch: f32,
    /// +0x259: the pad byte events 0x1b/0x1c set/clear [M 0x18add8 switch]; on the land car it
    /// is the handbrake path's byte. Here it selects the roll-free frame.
    pub handbrake: bool,
}

/// The front and rear axle points in the body frame (`DAT_00379e50` / `DAT_00379e60`) [M bytes].
pub const AXLE_FRONT: V3 = [0.0, 0.19, 1.25];
pub const AXLE_REAR: V3 = [0.0, 0.19, -1.25];

/// One SUB_PHYSICS vehicle: the body plus the actor fields `FUN_00190f88` keeps.
#[derive(Clone, Debug)]
pub struct SubCraft {
    pub attrs: VehicleAttrs,
    pub body: Body,
    /// actor+0x2c8 (0 = the player).
    pub driver: u32,
    /// `FUN_00195a30() > 0` (the vehicle's health pointer; 1.0 when none) [M].
    pub alive: bool,
    /// The profile flag at `DAT_003339bc[profile·0x1c + 7]` the player branch reads [M
    /// 0x190f88:162-166]; which option it is, is unproven. false = the shipped default [S].
    pub full_frame: bool,
    /// +0x29a: the brake-hold counter.
    pub brake_count: u8,
    /// +0x29c: reversing (set by the counter).
    pub reversing: bool,
    /// +0x288: |v| (signed: negated while going backward).
    pub speed: f32,
    /// +0x28c: the steer slip angle (turns).
    pub slip: f32,
    /// +0x294: the boost timer.
    pub boost_timer: i32,
    /// +0x420: the bob offset currently added to pos.y.
    pub bob: f32,
    /// +0x424 / +0x428: the bob phases (turns).
    pub bob_phase: [f32; 2],
    /// +0x44c: the roll direction latch.
    pub roll_latch: bool,
    /// +0x44d (signed byte), set by `FUN_001932b8` [M]; 0 at spawn [M FUN_001862c0:113].
    pub roll_mode: i8,
    /// +0x44f (signed byte, degrees), set by `FUN_001932c0` [M]; 0 at spawn.
    pub roll_target: i8,
    /// +0x2d6, set by `FUN_001932c0`; 1 at spawn [M FUN_001862c0:112].
    pub roll_release: bool,
    /// +0x29e, set by `FUN_001931b8`; 0 unless a script sets it [S: the ctor's zeroing is not
    /// traced].
    pub stunt: bool,
}

impl SubCraft {
    /// `bob_seed` is the 16-bit random draw `FUN_00187350` turns into the first bob phase
    /// (`seed · 1/65536`) [M 0x187350:101]; the second phase and the offset start at 0.
    pub fn new(attrs: VehicleAttrs, half_extents: V3, pos: V3, yaw: f32, bob_seed: u16) -> SubCraft {
        let inv = inv_inertia(attrs.mass, half_extents);
        let body = Body::new(attrs.mass, inv, pos, yaw);
        SubCraft {
            attrs,
            body,
            driver: 0,
            alive: true,
            full_frame: false,
            brake_count: 0,
            reversing: false,
            speed: 0.0,
            slip: 0.0,
            boost_timer: 0,
            bob: 0.0,
            bob_phase: [bob_seed as f32 * 1.525_878_9e-5, 0.0],
            roll_latch: false,
            roll_mode: 0,
            roll_target: 0,
            roll_release: true,
            stunt: false,
        }
    }

    fn player(&self) -> bool { self.driver == 0 }

    /// `FUN_00190f88` [M], the forces and torques for this tick (into the body's accumulators),
    /// the bob, and the boundary sheets. `input` is mutable because the pass zeroes the stored
    /// brake (+0x254) while the reverse counter runs [M 0x191080-0x1910a4].
    pub fn controller(&mut self, input: &mut SubInput) {
        let m = self.body.rows;
        let brake0 = input.brake;
        let mut steer = input.steer;
        let mut pitch = input.pitch;

        // The reverse latch [M 0x190fe8-0x1910b4].
        self.reversing = false;
        if brake0 == 0.0 {
            self.brake_count = 0x1e;
        } else {
            let fwd = to_local(&m, self.body.vel)[2];
            if 2.5 <= fwd {
                self.brake_count = 0;
            } else if (self.brake_count as i8) > 14 {
                if fwd <= 0.0 {
                    self.reversing = true;
                }
            } else {
                self.brake_count = self.brake_count.wrapping_add(1);
                if fwd < 0.5 {
                    input.brake = 0.0;
                }
            }
        }

        // The player's stick curves and the profile flag [M :160-167].
        let mut full = false;
        if self.player() {
            steer = steer * steer * steer;
            pitch = pitch * pitch * pitch * pitch * pitch;
            full = self.full_frame;
        }
        let mut af = AXLE_FRONT;
        let mut ar = AXLE_REAR;
        // The force frame [M :175-225]: the body rows, or with +0x259 set (and the flag clear) a
        // roll-free frame from the nose: x = norm(up × nose), y = nose × x.
        let mf: Rows = if full || !input.handbrake {
            m
        } else {
            let x = norm(cross([0.0, 1.0, 0.0], m[2]));
            [x, cross(m[2], x), m[2]]
        };
        let off_f = to_world(&mf, AXLE_FRONT);
        let off_r = to_world(&mf, AXLE_REAR);
        let vl = to_local(&mf, self.body.vel);
        let wl = to_local(&mf, self.body.ang_vel);
        if input.handbrake {
            af[1] = 0.0;
            ar[1] = 0.0;
        }

        // Thrust [M :344-367]: (throttle − stored brake)·7, plus the boost; (0, 0, 2·f) in the
        // body frame through FUN_001f84d0 (the body rows, not the force frame).
        let mut f = (input.throttle - input.brake) * 7.0;
        if self.boost_timer != 0 {
            self.boost_timer -= 1;
            if self.attrs.boost_time * 3 < self.boost_timer {
                f += self.attrs.boost_extra_acc;
            }
        }
        self.body.add_accel(to_world(&m, [0.0, 0.0, f + f]));
        // Drag [M :368-379]: −0.5·v for the player, −0.2·v otherwise.
        let k = if self.player() { -0.5 } else { -0.2 };
        self.body.add_accel(scale(self.body.vel, k));

        // Speed and the grip curves [M :380-399].
        let spd = dot(self.body.vel, self.body.vel).sqrt();
        self.speed = spd;
        let t = (spd / 15.0).min(1.0);
        let mut g_pitch = (1.0 - t) * 2.5 + 1.0;
        let mut g_steer = (1.0 - t) * 3.0 + 1.0;
        if self.player() {
            let fwd = to_local(&m, self.body.vel)[2];
            if fwd < 0.0 || input.throttle < brake0 {
                g_steer = 2.0;
                g_pitch = 2.0;
                let s = self.speed.min(20.0) * 0.05;
                pitch *= s;
                steer *= s;
            }
            if fwd < 0.0 {
                pitch = -pitch;
                self.speed = -self.speed;
            }
        } else if self.reversing {
            pitch = -pitch;
            self.speed = -self.speed;
        }
        let gain = if self.player() { 0.125 } else { 0.222 };
        let lim = f32::from_bits(0x3e63_53f8); // 0.222

        // --- the yaw block [M :436-591] ---
        let mut uf = add(cross(wl, af), vl);
        let mut ur = add(cross(wl, ar), vl);
        let steer_s = steer * g_steer * gain;
        self.slip = steer_s.min(lim).max(-lim);
        let mut assist = false;
        if !self.player() && self.speed < 5.0 && self.alive {
            assist = true;
            let d = self.slip * (5.0 - self.speed.abs()) * 4.0;
            ur[0] += d;
            uf[0] -= d;
        }
        // FUN_002cc590(−slip): a y-rotation; only the x lane of rot·uf is used.
        let (s, c) = (sin_turns(-self.slip), cos_turns(-self.slip));
        let ufx = c * uf[0] + s * uf[2];
        let keep_y = full && input.handbrake;
        for (ux, cap, off) in [(ufx, 12.5f32, off_f), (ur[0], 9.375f32, off_r)] {
            let fx = (-ux * 5.0).min(cap).max(-cap);
            let mut fw = to_world(&mf, [fx, 0.0, 0.0]);
            if !keep_y {
                fw[1] = 0.0;
            }
            let mut tq = cross(off, fw);
            if !assist {
                self.body.add_accel(fw);
            }
            if !self.player() {
                let mut tl = to_local(&m, tq);
                tl[2] *= 4.0;
                tq = to_world(&m, tl);
            }
            self.body.add_torque_mass(tq);
        }

        // --- the pitch block [M :592-704], gated on |pitch| > 0.01 || |steer term| < 0.01 ---
        if 0.01 < pitch.abs() || steer_s.abs() < 0.01 {
            let mut uf = add(cross(wl, af), vl);
            let mut ur = add(cross(wl, ar), vl);
            let pa = (pitch * g_pitch * gain).min(lim).max(-lim);
            let mut assist = false;
            if !self.player() && self.speed < 5.0 && self.alive {
                assist = true;
                let d = pa * (5.0 - self.speed.abs()) * 10.0;
                ur[1] -= d;
                uf[1] += d;
            }
            // FUN_002cc510(−pa): an x-rotation; the y lane of rot·uf = c·uf.y − s·uf.z.
            let (s, c) = (sin_turns(-pa), cos_turns(-pa));
            let ufy = c * uf[1] - s * uf[2];
            for (uy, off) in [(ufy, off_f), (ur[1], off_r)] {
                let fy = (-uy * 4.0).min(12.0).max(-12.0);
                let fw = to_world(&mf, [0.0, fy, 0.0]);
                let tq = cross(off, fw);
                if !assist {
                    self.body.add_accel(fw);
                }
                self.body.add_torque_mass(tq);
            }
        }

        // --- the angular drag and the roll leveller [M :705-858] ---
        let mut td = scale(to_local(&m, self.body.ang_mom), -2.0);
        let nose = m[2];
        let right = norm(cross([0.0, 1.0, 0.0], nose));
        let up = cross(nose, right);
        let mass = self.body.mass;
        'level: {
            if 0.7 <= nose[1].abs() {
                break 'level;
            }
            let tgt;
            if !input.handbrake {
                tgt = self.roll_target;
            } else if !self.player() {
                let d = dot(up, m[1]);
                if !self.roll_latch { td[2] += d * 10.0 * mass } else { td[2] -= d * 10.0 * mass }
                break 'level;
            } else {
                tgt = self.roll_target;
            }
            if tgt == 0 {
                td[2] -= dot(up, m[0]) * 10.0 * mass;
            } else {
                let sc = sin_turns(tgt as f32 * 0.002_777_777_8);
                if m[0][1].abs() < sc.abs() {
                    let d = dot(up, m[1]);
                    if tgt >= 0 { td[2] -= d * 10.0 * mass } else { td[2] += d * 10.0 * mass }
                } else if self.roll_release {
                    self.roll_target = 0;
                }
            }
            if self.roll_mode == 0 {
                self.roll_latch = m[0][1] < 0.0;
                let wl = to_local(&m, self.body.ang_vel);
                if wl[2].abs() <= 0.1 || m[1][1] <= 0.34 {
                    break 'level;
                }
                self.roll_latch = wl[2] < 0.0;
            } else if self.roll_mode < 1 {
                self.roll_latch = false;
            } else {
                self.roll_latch = true;
            }
        }
        let tw = to_world(&m, td);
        self.body.add_torque(tw);

        // --- the bob [M :860-875] ---
        self.body.pos[1] -= self.bob;
        let s1 = sin_turns(self.bob_phase[0]);
        let s2 = sin_turns(self.bob_phase[1]);
        self.bob_phase[0] += 0.0022;
        self.bob = (s1 + 1.0 + s2) * 0.075;
        if 1.0 < self.bob_phase[0] {
            self.bob_phase[0] -= 1.0;
        }
        self.bob_phase[1] += 0.0035;
        if 1.0 < self.bob_phase[1] {
            self.bob_phase[1] -= 1.0;
        }
        self.body.pos[1] += self.bob;

        // --- the boundary sheets [M 0x1920c4-0x1923bc; docs/systems/driving-world-bounds.md] ---
        if self.driver != 2 || self.alive {
            let p4 = [self.body.pos[0], self.body.pos[1], self.body.pos[2], 1.0];
            for z in ZONES.iter() {
                if !z.hit(p4) {
                    continue;
                }
                let n = z.normal;
                let v_n = ZoneQuad::clamped_approach_speed(-dot(self.body.vel, n));
                let sc = z.penetration_scale(z.signed_dist(p4));
                // The accumulator read is the live one: each applied sheet adds before the next
                // sheet's f·n [M FUN_001f87d8 inside the loop].
                let fa = [self.body.force[0], self.body.force[1], self.body.force[2], 0.0];
                if let Some(a) = z.boundary_force(v_n, fa, mass, FPS, sc) {
                    self.body.add_accel(a);
                    let tl = z.scrape_torque_local(self.speed, dot(m[2], n), dot(m[0], n));
                    self.body.add_torque_mass(to_world(&m, tl));
                }
            }
        }

        // --- the +0x29e impulse [M :920-940]: (pitch, steer, 0)·throttle·mass, local, into L ---
        if self.stunt {
            let k = input.throttle * mass;
            self.body.add_ang_impulse_local([input.pitch * k, input.steer * k, 0.0]);
        }
    }

    /// One tick: the controller pass, then the integration (the game runs the integration at the
    /// top of the NEXT frame [M 0x1ff3d8 D→E→controller]; the order of values is the same).
    pub fn step(&mut self, input: &SubInput, rigid: &Rigid, dt: f32) {
        let mut inp = *input;
        self.controller(&mut inp);
        self.body.integrate(rigid, dt);
    }
}

// ---------------------------------------------------------------------------
// The port's dispatcher
// ---------------------------------------------------------------------------

/// A driving-mission vehicle under the pass its attributes select.
#[derive(Clone, Debug)]
pub enum Vehicle {
    Land(Car),
    Sub(SubCraft),
}

impl Vehicle {
    /// The player's vehicle (driver 0, not path-driven) [M 0x188bf8]: SUB_PHYSICS → `SubCraft`,
    /// everything else → the land car (the snowmobile pass is not ported; it gets the land step).
    pub fn player(attrs: VehicleAttrs, params: CarParams, half_extents: V3, pos: V3, yaw: f32, bob_seed: u16) -> Vehicle {
        match drive_pass(&attrs, 0, false) {
            DrivePass::Sub => Vehicle::Sub(SubCraft::new(attrs, half_extents, pos, yaw, bob_seed)),
            _ => Vehicle::Land(Car::new(params, pos, yaw)),
        }
    }

    /// One tick under the vehicle's pass. `pitch` is the +0x244 stick (the land car ignores it).
    pub fn step(&mut self, input: &CarInput, pitch: f32, world: &FlatWorld, rigid: &Rigid, dt: f32) {
        match self {
            Vehicle::Land(c) => c.step(input, world, rigid, dt),
            Vehicle::Sub(s) => s.step(
                &SubInput { throttle: input.throttle, brake: input.brake, steer: input.steer, pitch, handbrake: input.handbrake },
                rigid,
                dt,
            ),
        }
    }

    pub fn pos(&self) -> V3 {
        match self {
            Vehicle::Land(c) => c.pos,
            Vehicle::Sub(s) => s.body.pos,
        }
    }
}
