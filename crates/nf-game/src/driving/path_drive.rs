//! The player vehicle while a spline drives it, and the MIS01 helicopter's flight model.
//! Write-up: docs/systems/drive-player-vehicles.md.
//!
//! Evidence: [M] DRIVING.ELF decompile (EE addresses), [D] disc bytes, [S] suggested.
//!
//! What the game does [M]:
//! - The helicopter is an ordinary car. `helicopter.atr` [D MIS01.VIV] is the Vanquish's land
//!   tuning (MASS 1660, gears, springs, tyres) plus `SECONDARY_TYPE.s=vanquish`,
//!   `NO_SHADOW.b=true` and `IS_HELICOPTER.b=true`; no `SUB_PHYSICS`, no `IS_SNOWMOBILE`.
//!   `IS_HELICOPTER` is read once, by the car ctor `FUN_001862c0` (:202) for AI cars only
//!   (param 2 != 0 → car+0x37c = 0); for the player it is never read. The name `helicopter`
//!   (or `helivanquish`) only picks the engine sound object in `FUN_00186878`.
//! - **The pass** `FUN_00188200` gives a car `FUN_00192498` (the path pass) when its driver
//!   (car+0x2e4, vtable +0x5c `FUN_00192fe8`) holds a spline controller (driver+0x50,
//!   `FUN_0013e0b0`) and its health (`FUN_00195a30`: *car+0x54, 1.0 without one) is above 0;
//!   otherwise SUB_PHYSICS → `FUN_00190f88`, IS_SNOWMOBILE (attr +0xc8 through vtable +0x1d4
//!   = `FUN_00193170`, which returns car+0x23c, the attr block) → `FUN_00190510`, driver index
//!   car+0x2c8 == 0 → `FUN_0018cef8` (the player land pass), else `FUN_0018ecb8` (the AI land
//!   pass) [M 0x188b70-0x188c60]. `EControlToSpline` sets car+0x2c8 = 1 (`FUN_00131508` →
//!   vtable +0x94 `FUN_00193000`), so a spline-driven player whose path test fails would get
//!   the AI land pass.
//! - **On the spline** the vehicle's pose IS the path: the driver update `FUN_0012e350`, state 3,
//!   runs `FUN_00131620`: the body is placed (`FUN_001f7880`: linear and angular velocity and
//!   momentum zeroed, position = the follower's target row 3, orientation = the target's
//!   rotation times the controller's offset matrix +0x70, through a quaternion), then its
//!   velocity (+0x20) = the follower's velocity (+0x88) and its momentum (+0x40) = that ×
//!   mass (+0x4c). There is no lift, thrust or drag: the flight is the keyed track
//!   (`flight::PathFollower`).
//! - **The path pass** `FUN_00192498` returns at once for SUB_PHYSICS; else it derives what
//!   the drawing / sound side reads: speed (+0x288) = |v|; wheel spin (+0x274) =
//!   |v.xz| / (WHEEL_RADIUS · 6.32 · fps), 0 under 0.01, at most 1; two roll phases
//!   (+0x230, +0x234) += spin, wrapped into [0, 1]; heading c = row2 · v̂: c < 0.707 sets the
//!   skid flag +0x43e and the rear slip flags (+0x228, +0x22c = 1), c < 0.25 also the front
//!   ones (+0x220, +0x224); steer (+0x28c) = clamp(row0 · v̂, ±1); then, unless IS_SNOWMOBILE,
//!   per wheel the ground query `FUN_001f6ae0` at the wheel's world point and the spring
//!   length (+0x210) = clamp(d + SPRING_REST_LENGTH, 0, SPRING_COMPRESSION_LIMIT), a 0 length
//!   clearing that wheel's slip flag. For IS_SNOWMOBILE the springs and slip flags are zeroed.
//! - **Off the spline** the helicopter has no flight model of its own: the player land pass
//!   `FUN_0018cef8` (`physics::Car::step`) would run on it. In MIS01 the spline is ended in
//!   the same lists as `ECustomTransition("paris")`, which rebuilds the car as the Vanquish
//!   (`nf_formats::drive_vehicle`).

use super::flight::{mat_mul, Mat4, PathFollower};
use super::physics::{CarParams, Ground, WheelPlane};
use super::sub_physics::{drive_pass, DrivePass, VehicleAttrs};

/// The `6.32` of the wheel spin divisor, as the decompile prints the literal [M 0x192498].
pub const SPIN_CIRCUMFERENCE: f32 = 6.32;
/// The skid threshold on the heading cosine [M 0x192498: 0.707].
pub const SKID_REAR_COS: f32 = 0.707;
/// The front-wheel skid threshold [M 0x192498: 0.25].
pub const SKID_FRONT_COS: f32 = 0.25;

/// The path test of `FUN_00188200` [M 0x188b70-0x188bb8]: a driver, its spline controller, and
/// health above 0.
pub fn path_driven(has_driver: bool, has_spline: bool, health: f32) -> bool {
    has_driver && has_spline && health > 0.0
}

/// The pass `FUN_00188200` runs for a player car [M]: `driver` = car+0x2c8 (0 = the player
/// drives, 1 while `EControlToSpline` holds it).
pub fn player_pass(attrs: &VehicleAttrs, driver: u32, has_spline: bool, health: f32) -> DrivePass {
    drive_pass(attrs, driver, path_driven(true, has_spline, health))
}

/// The body as `FUN_00131620` leaves it: rows 0-2 the orientation, row 3 the position.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PathBody {
    pub pose: Mat4,
    /// +0x20.
    pub velocity: [f32; 3],
    /// +0x40: velocity × mass.
    pub momentum: [f32; 3],
    /// +0x30 / +0x50: zeroed by `FUN_001f7880`.
    pub angular_velocity: [f32; 3],
}

impl PathBody {
    pub fn pos(&self) -> [f32; 3] { [self.pose[3][0], self.pose[3][1], self.pose[3][2]] }
    pub fn row(&self, i: usize) -> [f32; 3] { [self.pose[i][0], self.pose[i][1], self.pose[i][2]] }
}

/// `FUN_00131620` [M]: the follower's target rotation (`FUN_001b18a8` copies it with the
/// position row cleared) times the controller's offset matrix (`FUN_002cbb88`), the target's
/// position row, the follower's velocity, momentum = velocity × mass. The orientation's trip
/// through a quaternion (`FUN_002cc690` then `FUN_002cc0c8`) is taken as exact for the
/// orthonormal path rows [S].
pub fn place_on_path(f: &PathFollower, offset: &Mat4, mass: f32) -> PathBody {
    let mut rot = f.pose;
    rot[3] = [0.0, 0.0, 0.0, 1.0];
    for r in rot.iter_mut().take(3) {
        r[3] = 0.0;
    }
    let mut pose = mat_mul(&rot, offset);
    pose[3] = [f.pose[3][0], f.pose[3][1], f.pose[3][2], 1.0];
    PathBody {
        pose,
        velocity: f.velocity,
        momentum: f.velocity.map(|v| v * mass),
        angular_velocity: [0.0; 3],
    }
}

/// `FUN_00213618` [M]: the surface bump the ground query adds to a wheel's depth. Surfaces
/// 2/3/5/10: 0.01 at frequency 2; 4/7: 0.015 at frequency 2; 6: a flat -10; else 0. `p` is the
/// wheel point; the trig is `sinf` (`FUN_002bfc38`) of z and `cosf` (`FUN_002bfa48`) of x.
pub fn surface_bump(surface: u8, p: [f32; 3]) -> f32 {
    let (freq, amp) = match surface {
        2 | 3 | 5 | 10 => (2.0f32, 0.01f32),
        4 | 7 => (2.0, 0.015),
        6 => (0.0, -10.0),
        _ => (0.0, 0.0),
    };
    if amp == 0.0 {
        return 0.0;
    }
    let tau = 6.283_185_5f32;
    ((p[2] * freq * tau).sin() + (p[0] * freq * tau).cos()) * amp
}

/// `FUN_001f6ae0`'s depth (the out vector's w) for a wheel point over a plane [M]:
/// `(v0 − p) · n + bump`, plus 10 for an IS_BOAT car (body class < 4). No plane: normal up,
/// depth 0 (the `pfVar3[0xc] == 0` arm).
pub fn wheel_depth(plane: Option<&WheelPlane>, p: [f32; 3], is_boat: bool) -> ([f32; 3], f32) {
    let Some(pl) = plane else { return ([0.0, 1.0, 0.0], 0.0) };
    let n = pl.normal;
    let mut d = (pl.v0[0] - p[0]) * n[0] + (pl.v0[1] - p[1]) * n[1] + (pl.v0[2] - p[2]) * n[2] + surface_bump(pl.surface, p);
    if is_boat {
        d += 10.0;
    }
    (n, d)
}

/// What `FUN_00192498` keeps on the car.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PathDrive {
    /// +0x288: |velocity|.
    pub speed: f32,
    /// +0x274: the wheel spin per tick, in turns, [0, 1].
    pub wheel_spin: f32,
    /// +0x230 / +0x234: the two roll phases, wrapped into [0, 1].
    pub roll: [f32; 2],
    /// +0x43e: heading more than 45 degrees off the velocity.
    pub skid: bool,
    /// +0x220..+0x22c: the per-wheel slip flags (set to 1 here, cleared for an unloaded wheel;
    /// other writers keep their values otherwise).
    pub slip: [f32; 4],
    /// +0x28c: row0 · v̂ in [-1, 1].
    pub steer: f32,
    /// +0x210..: the spring lengths.
    pub spring: [f32; 4],
    /// +0x190..: each wheel's ground normal and depth (the query's out vector).
    pub ground: [([f32; 3], f32); 4],
    /// +0x90 + 0x40·i: the per-wheel cached plane (a miss keeps the last one, as `Ground` does
    /// for the land pass).
    pub cache: [Option<WheelPlane>; 4],
}

fn wrap01(x: f32) -> f32 {
    // [M 0x192498 :82-101] 1 < x → x − int(x); x < 0 → x − (int(x) − 1); 1.0 itself stays.
    if 1.0 < x {
        x - (x as i32) as f32
    } else if x < 0.0 {
        x - ((x as i32) - 1) as f32
    } else {
        x
    }
}

impl PathDrive {
    /// One `FUN_00192498` over the body `place_on_path` left [M]. `fps` = the integer rate
    /// (`DAT_00339f4c`, 60 on NTSC).
    pub fn step(&mut self, body: &PathBody, params: &CarParams, attrs: &VehicleAttrs, fps: i32, ground: &dyn Ground) {
        if attrs.sub_physics != 0 {
            return;
        }
        let v = body.velocity;
        self.speed = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        let mut spin = (v[0] * v[0] + v[2] * v[2]).sqrt();
        if 0.0 < params.wheel_radius {
            spin /= params.wheel_radius * SPIN_CIRCUMFERENCE * fps as f32;
        }
        if spin < 0.01 {
            spin = 0.0;
        }
        let spin = if spin < 1.0 { spin } else { 1.0 };
        self.wheel_spin = spin;
        self.roll = [wrap01(self.roll[0] + spin), wrap01(self.roll[1] + spin)];
        // v̂ through vrsqrt: a zero velocity gives a zero vector on the VU (0 × the max float)
        // [S: the VU's rsqrt(0) result is not measured].
        let len = self.speed;
        let dir = if len > 0.0 { v.map(|c| c / len) } else { [0.0; 3] };
        let dotr = |r: [f32; 3]| r[0] * dir[0] + r[1] * dir[1] + r[2] * dir[2];
        let c = dotr(body.row(2));
        if c < SKID_REAR_COS {
            self.skid = true;
            self.slip[3] = 1.0;
            self.slip[2] = 1.0;
            if c < SKID_FRONT_COS {
                self.slip[0] = 1.0;
                self.slip[1] = 1.0;
            }
        } else {
            self.skid = false;
        }
        self.steer = dotr(body.row(0)).clamp(-1.0, 1.0);
        if attrs.is_snowmobile != 0 {
            self.spring = [0.0; 4];
            self.slip = [0.0; 4];
            return;
        }
        let pos = body.pos();
        for (i, w) in params.wheel_points().iter().enumerate() {
            let r = |k: usize| body.row(k);
            let p = [0, 1, 2].map(|k| r(0)[k] * w[0] + r(1)[k] * w[1] + r(2)[k] * w[2] + pos[k]);
            if let Some(pl) = ground.wheel_plane(p) {
                self.cache[i] = Some(pl);
            }
            let (n, d) = wheel_depth(self.cache[i].as_ref(), p, attrs.is_boat != 0);
            self.ground[i] = (n, d);
            let x = (d + params.spring_rest).min(params.spring_limit);
            let x = if 0.0 < x { x } else { 0.0 };
            self.spring[i] = x;
            if x <= 0.0 {
                self.slip[i] = 0.0;
            }
        }
    }
}

/// The player helicopter on its spline, one 60 Hz tick: the follower advances, the body takes
/// its pose, then the path pass runs. The follower and the pose copy run in the driver update
/// and the pass in the car controller; their order inside the frame is [S].
#[derive(Clone, Debug)]
pub struct HeliFlight {
    pub follower: PathFollower,
    /// The controller's offset matrix (+0x70): identity when the `AISp` record's +0x6c is set
    /// (every MIS01 record) [M `FUN_002db690`, D].
    pub offset: Mat4,
    pub body: PathBody,
    pub pass: PathDrive,
    pub params: CarParams,
    pub attrs: VehicleAttrs,
}

impl HeliFlight {
    pub fn new(follower: PathFollower, offset: Mat4, params: CarParams, attrs: VehicleAttrs) -> HeliFlight {
        let body = place_on_path(&follower, &offset, attrs.mass);
        HeliFlight { follower, offset, body, pass: PathDrive::default(), params, attrs }
    }

    /// `now` = `flight::clock60(tick, fps)`. Returns true when the track wrapped.
    pub fn step(
        &mut self,
        now: f32,
        fps: i32,
        tracks: &std::collections::BTreeMap<u16, nf_formats::drive_script::PathTrack>,
        ground: &dyn Ground,
    ) -> bool {
        let wrapped = self.follower.update(now, fps, tracks);
        self.body = place_on_path(&self.follower, &self.offset, self.attrs.mass);
        self.pass.step(&self.body, &self.params, &self.attrs, fps, ground);
        wrapped
    }
}
