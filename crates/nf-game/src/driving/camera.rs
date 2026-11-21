//! The driving engine's player chase camera: the "Heli" camera type of `data\render\camera.ini`
//! (docs/systems/driving-camera.md).
//!
//! Sources:
//! - [D] `data\render\camera.ini` (refpack-compressed, 4322 bytes packed, the same file in all
//!   eight `/DRIVING/*.VIV`). Its `[<Name>HeliCam<n>]` sections carry the follow tuning, the
//!   `[<Name>HeliCam<n>:Arm<k>]` sections the arm (offset) and its anchor.
//! - [M] the loader `FUN_001a49e0` (reads every key named below into the 0x4c-byte heli record,
//!   table `DAT_003339b8`; the 0x1c-byte mode record, table `DAT_003339bc`; 0x40-byte arms).
//! - [M] the heli update `FUN_001b2b48` (the mode record's +0x14 member pointer for type 2,
//!   `DAT_0037da88` = {0xffff0000, 0x1b2b48}), its anchor step `FUN_001d6498`, its world pull-in
//!   `FUN_001b9198` (+ `FUN_001b9020` / `FUN_001b8f80`), the look matrix `FUN_001d5700` and the
//!   fov step `FUN_001b8740` / `FUN_001d6720`.
//!
//! What runs per update (the normal path: no mode transition, not paused, target is a vehicle):
//! 1. arm = (Heli_Sideways, Heli_Height, Heli_Distance). Look-back (latched, mode `lookBack`)
//!    negates the distance; otherwise distance -= min(|v| * Fallback_Factor, Max_Fallback)
//!    [M 0x1b2c38-0x1b2c80, R5900 `min.s` at 0x1b2c70].
//! 2. "vertigo": `s += (fwd.y * (fwd.y < 0 ? MaxVertigoDownhill : MaxVertigoUphill) - s) *
//!    Vertigo_Lerp`, then the arm's (y, z) turn by asin(s) [M 0x1b2da8-0x1b2e28].
//! 3. the arm goes to world on the car's FLATTENED heading: `right*x + (0, y, 0) + fwd*z`,
//!    fwd = normalize(car.fwd.x, 0, car.fwd.z), right = (-fwd.z, 0, fwd.x) [M 0x1b2e30-0x1b2e70].
//! 4. the anchor: `a += (R*Anchor - a) * 0.05`, anchor point = car.pos + a [M FUN_001d6498].
//! 5. the follow rate = clamp(|v| * dt*60 * Speed_Rate_Diff, Min_Rate, Max_Rate)
//!    [M 0x1b3174-0x1b31b4]; the kept offset is rescaled to the target's length, then
//!    `offset += (target - offset) * rate` [M 0x1b3330-0x1b33c0]. eye = anchor + offset.
//! 6. checkCollisions: three segment casts car.pos -> the eye (x1.075, +-the camera basis) pull
//!    the eye in, never nearer than 2.5; a fourth cast lowers the eye while something stands
//!    between it and the car [M FUN_001b9198]. The casts see `CDat` collision only ([`CamWorld`]);
//!    then the eye is lifted out of other vehicles' and type-8 bodies' ellipsoids ([`push_out`],
//!    `FUN_001b8ae0` / `FUN_001b8d08`).
//! 7. look: forward = normalize(anchor + up*lookUp - eye), right = normalize(up x forward),
//!    up' = forward x right [M FUN_001d5700]. fov = the mode's defaultFov (33.0 unless set).

/// The per-camera follow tuning: the 0x4c-byte heli record `FUN_001a49e0` fills [M :705-740].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeliParams {
    /// `Min_Rate` (+0x00) - the follow rate floor.
    pub min_rate: f32,
    /// `Max_Rate` (+0x04) - the follow rate ceiling.
    pub max_rate: f32,
    /// `Speed_Rate_Diff` (+0x08) - rate per unit of |velocity|.
    pub speed_rate_diff: f32,
    /// `Fallback_Factor` (+0x10) - extra arm distance per unit of |velocity|.
    pub fallback_factor: f32,
    /// `Max_Fallback` (+0x1c) - the cap on that extra distance.
    pub max_fallback: f32,
    /// `Vertigo_Lerp` (+0x20).
    pub vertigo_lerp: f32,
    /// `MaxVertigoDownhill` (+0x30), clamped to [0, 1] at load (`FUN_001a49b0`).
    pub max_vertigo_downhill: f32,
    /// `MaxVertigoUphill` (+0x34), clamped to [0, 1] at load.
    pub max_vertigo_uphill: f32,
    /// `rigidArm` (+0x28 byte, default 0). Only the non-rigid arm is ported.
    pub rigid_arm: bool,
    /// `checkCollisions` (+0x29 byte, default 1).
    pub check_collisions: bool,
    /// `noiseAmount` (+0x3c). The noise term is not ported; every land-car heli cam has 0 [D].
    pub noise_amount: f32,
    /// `upRate` (+0x44) - 0 keeps the camera up vector where it is.
    pub up_rate: f32,
    /// `lookUp` (+0x48) - the look point's lift along the camera up.
    pub look_up: f32,
    /// mode `lookBack` (mode record byte 3).
    pub look_back: bool,
    /// mode `defaultFov` (mode record +0x0c, default 33.0 [M :266]).
    pub default_fov: f32,
}

/// One arm: the 0x40-byte arm record [M FUN_001a49e0 :756-770, FUN_001d69c8].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArmParams {
    /// `Heli_Sideways` (+0x30).
    pub sideways: f32,
    /// `Heli_Height` (+0x34).
    pub height: f32,
    /// `Heli_Distance` (+0x38), negative = behind the car.
    pub distance: f32,
    /// `Anchor_X/Y/Z` (+0x00/+0x04/+0x08), in the car's frame.
    pub anchor: [f32; 3],
}

/// `[AnyCarHeliCam0]` + `:Arm0` [D camera.ini]: the default camera of a car no car-specific
/// HeliCam names (the loader keeps the AnyCar heli cams only while no car-specific one was
/// taken [M FUN_001a49e0 :692-698]).
pub const ANYCAR_HELI_CAM0: (HeliParams, ArmParams) = (
    HeliParams {
        min_rate: 0.06,
        max_rate: 0.25,
        speed_rate_diff: 0.001_666_67,
        fallback_factor: 0.046_25,
        max_fallback: 1.22,
        vertigo_lerp: 0.5,
        max_vertigo_downhill: 1.0,
        max_vertigo_uphill: 1.0,
        rigid_arm: false,
        check_collisions: true,
        noise_amount: 0.0,
        up_rate: 0.0,
        look_up: 0.0,
        look_back: true,
        default_fov: 33.0,
    },
    ArmParams { sideways: 0.0, height: 0.2, distance: -5.0, anchor: [0.0, 0.95, 0.0] },
);

/// `[VanquishHeliCam0]` + `:Arm0` [D camera.ini]: `car = vanquish,vanquishalps,cobra_player`,
/// `defaultCamera = 1`. Only the arm differs from AnyCar's.
pub const VANQUISH_HELI_CAM0: (HeliParams, ArmParams) = (
    ANYCAR_HELI_CAM0.0,
    ArmParams { sideways: 0.0, height: 0.85, distance: -5.10, anchor: [0.0, 0.3, 0.0] },
);

/// The default heli camera for a car name: the first `defaultCamera = 1` HeliCam whose `car =`
/// list names it, else AnyCar's [D camera.ini]. Only the two shipped sets above are tabled; any
/// other name falls back to AnyCar (unproven for the other named cars).
pub fn default_heli_cam(car: &str) -> (HeliParams, ArmParams) {
    match car.to_ascii_lowercase().as_str() {
        "vanquish" | "vanquishalps" | "cobra_player" => VANQUISH_HELI_CAM0,
        _ => ANYCAR_HELI_CAM0,
    }
}

/// The literals of the update and the pull-in [M].
pub const ANCHOR_LERP: f32 = 0.05; // FUN_001d6498 0x3d4ccccd
pub const RAY_SCALE: f32 = 1.075; // FUN_001b9198 0x3f89999a
pub const RAY_LOW: f32 = -0.5; // 0xbf000000 along the camera up
pub const RAY_HIGH_STEP: f32 = 1.0; // then +1.0 up (net +0.5)
pub const RAY_SIDE: f32 = -0.4; // 0xbecccccd along the camera right, then +0.8 (net +0.4)
pub const HIT_IGNORE: f32 = 1.1; // hits nearer the car than this are ignored (0x3f8ccccd)
pub const MIN_DIST: f32 = 2.5; // heli cam, noiseAmount <= 0, param_4 = 1 (0x40200000)
pub const MIN_DIST_NOISY: f32 = 2.0; // heli cam with noise (not reached by land cars)
pub const PULL_LIFT: f32 = 0.5; // eye += up * 0.5*(1 - scale) (0x3f000000; 0.1 when noisy)
pub const DIP_GAP: f32 = 1.5; // a hit within this of MIN_DIST disables the dip (0x3fc00000)
pub const DIP_FROM: f32 = 0.125; // 0x3e000000
pub const DIP_TO: f32 = 0.975; // 0x3f79999a
pub const DIP_MAX: i32 = 0x50;
pub const DIP_STEP: f32 = -0.0125; // eye += up * count * -0.0125

/// `kCameraObjectRadiusEx.x/.y/.z` (`DAT_00333650/54/58`): camera.ini `[Global]` sets 1.5 each [D];
/// the loader's defaults are 1.675 / 1.925 / 1.325 [M FUN_001a49e0 :801-803].
pub const OBJECT_RADIUS_EX: [f32; 3] = [1.5, 1.5, 1.5];
pub const OBJECT_RADIUS_EX_DEFAULT: [f32; 3] = [1.675, 1.925, 1.325];
/// `kCameraObjectSphereRad` (`DAT_00333660`): 2.598 in camera.ini [D], loader default 2.875
/// (0x40380000) [M FUN_001a49e0 :804].
pub const OBJECT_SPHERE_RAD: f32 = 2.598;
pub const OBJECT_SPHERE_RAD_DEFAULT: f32 = 2.875;
/// The driving camera's near and far planes: `RCamera` ctor `FUN_001d40f0` sets +0x20 = 0.17
/// (0x3e2e147b) and +0x2c = 3000.0 (0x453b8000); `FUN_001d4878` hands both to the projection
/// `FUN_00295910` [M]. No write that moves the near plane while driving was found.
pub const NEAR_PLANE: f32 = 0.17;
pub const FAR_PLANE: f32 = 3000.0;

/// One collision triangle as the camera cast sees it: a `CDat` `ci` item's `ca` strip triangle
/// in world space with its class byte (vertex k+2's +0x2d) [M FUN_00213d68 reads +0x2d].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CamTri {
    pub v: [[f32; 3]; 3],
    pub class: u8,
}

/// The world the chase camera's segment casts test (`FUN_002127f0` -> `FUN_002166f8` on
/// `DAT_0033a540`, the CDat collision world) [M]:
///
/// - the gatherer the camera keeps at +0x35c (`FUN_002117e8`, flags 0xc) collects, around the
///   car-to-eye midpoint, from the `CGrd` grid (`DAT_0033a5c8`, `FUN_0021cf30`) the cells' kind-0
///   entries: u16 indices into the `ci` item array (world +8, 0x40-byte items) plus the cells'
///   linked kind-0 nodes (moved `ci` items, `FUN_0021cad0`) [M FUN_00217228]. Flag 8 keeps the
///   items' triangle groups (`FUN_00216ec0`), flag 4 their `ca` wall edges (`FUN_00219a50`);
/// - the cast tests the edges (`FUN_0021a248` -> `FUN_00219328`) and, flag 8 set, the triangles
///   (`FUN_00232638` -> `FUN_00214c08` -> `FUN_00213d68`, both windings accepted, a triangle whose
///   class byte meets the world's +0x2c mask skipped); `FUN_00216d30` keeps the nearer hit.
///
/// So the camera sees `CDat` collision only: no scene objects (`RSceneObj` props such as RACE's
/// `POS2_Cautionguard` rails are not `ci` items), no smackables, no cars (cars are handled by the
/// ellipsoid push-out, [`push_out`]). There is no slope filter: walls count.
///
/// Port: the triangle half only (the `ca` wall-edge list is not extracted by the port yet); a
/// 16-unit XZ grid stands in for the game's sphere gather (a port choice: same set of candidate
/// triangles along the segment). The world class mask is runtime state; 0 (nothing skipped)
/// unless set.
#[derive(Clone, Debug, Default)]
pub struct CamWorld {
    tris: Vec<CamTri>,
    cells: std::collections::HashMap<(i32, i32), Vec<u32>>,
    /// The world +0x2c class mask (unproven value; 0 skips nothing).
    pub class_mask: u8,
}

impl CamWorld {
    const CELL: f32 = 16.0;

    pub fn new(tris: Vec<CamTri>) -> CamWorld {
        let mut cells: std::collections::HashMap<(i32, i32), Vec<u32>> = std::collections::HashMap::new();
        for (i, t) in tris.iter().enumerate() {
            let (mut x0, mut x1, mut z0, mut z1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
            for p in &t.v {
                x0 = x0.min(p[0]);
                x1 = x1.max(p[0]);
                z0 = z0.min(p[2]);
                z1 = z1.max(p[2]);
            }
            for cx in (x0 / Self::CELL).floor() as i32..=(x1 / Self::CELL).floor() as i32 {
                for cz in (z0 / Self::CELL).floor() as i32..=(z1 / Self::CELL).floor() as i32 {
                    cells.entry((cx, cz)).or_default().push(i as u32);
                }
            }
        }
        CamWorld { tris, cells, class_mask: 0 }
    }

    pub fn len(&self) -> usize {
        self.tris.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tris.is_empty()
    }

    /// The nearest hit point of segment a->b (two-sided), or None: the camera's `FUN_002127f0`.
    pub fn cast(&self, a: [f32; 3], b: [f32; 3]) -> Option<[f32; 3]> {
        let d = sub(b, a);
        let (cx0, cx1) = ((a[0].min(b[0]) / Self::CELL).floor() as i32, (a[0].max(b[0]) / Self::CELL).floor() as i32);
        let (cz0, cz1) = ((a[2].min(b[2]) / Self::CELL).floor() as i32, (a[2].max(b[2]) / Self::CELL).floor() as i32);
        let mut best: Option<f32> = None;
        for cx in cx0..=cx1 {
            for cz in cz0..=cz1 {
                let Some(list) = self.cells.get(&(cx, cz)) else { continue };
                for &i in list {
                    let t = &self.tris[i as usize];
                    if t.class & self.class_mask != 0 {
                        continue;
                    }
                    if let Some(s) = segment_tri(a, d, t.v) {
                        if best.is_none_or(|b| s < b) {
                            best = Some(s);
                        }
                    }
                }
            }
        }
        best.map(|s| add(a, scale(d, s)))
    }
}

/// Segment a + d*s (s in [0, 1]) against a triangle, either winding: the s of the hit.
fn segment_tri(a: [f32; 3], d: [f32; 3], v: [[f32; 3]; 3]) -> Option<f32> {
    let e1 = sub(v[1], v[0]);
    let e2 = sub(v[2], v[0]);
    let h = cross(d, e2);
    let det = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];
    if det.abs() < 1e-12 {
        return None;
    }
    let inv = 1.0 / det;
    let sv = sub(a, v[0]);
    let u = inv * (sv[0] * h[0] + sv[1] * h[1] + sv[2] * h[2]);
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = cross(sv, e1);
    let w = inv * (d[0] * q[0] + d[1] * q[1] + d[2] * q[2]);
    if w < 0.0 || u + w > 1.0 {
        return None;
    }
    let s = inv * (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]);
    (0.0..=1.0).contains(&s).then_some(s)
}

/// A body the eye is pushed out of (`FUN_001b8ae0` -> `FUN_001b8d08`) [M]:
///
/// - every vehicle in the sim's vehicle list (`DAT_00339f5c..60`) except the camera's own target
///   (camera +0x118): half-extents = the vehicle's size vector (its +0x5c object's +0x4c0) times
///   `kCameraObjectRadiusEx`, gate `|eye - pos|^2 <= |size|^2 * kCameraObjectSphereRad`;
/// - every live physics body of the 0x60-slot table (`0x339190`, `DAT_003394c0`) whose type byte
///   +0x1c is 8: half-extents = `kCameraObjectRadiusEx * radius (+0x2c)`, gate
///   `|eye - pos|^2 <= radius * kCameraObjectSphereRad` (the radius itself, not squared, as the
///   code compares).
///
/// `rows` are the body's orientation rows (right, up, forward), the frame the test runs in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Obstacle {
    pub pos: [f32; 3],
    pub rows: [[f32; 3]; 3],
    pub half: [f32; 3],
    /// The squared-distance gate described above.
    pub gate2: f32,
}

impl Obstacle {
    /// A vehicle of size `size` (its +0x4c0 vector, unnamed in the code) [M FUN_001b8ae0 loop 1].
    pub fn vehicle(pos: [f32; 3], rows: [[f32; 3]; 3], size: [f32; 3]) -> Obstacle {
        let half = [size[0] * OBJECT_RADIUS_EX[0], size[1] * OBJECT_RADIUS_EX[1], size[2] * OBJECT_RADIUS_EX[2]];
        let s2 = size[0] * size[0] + size[1] * size[1] + size[2] * size[2];
        Obstacle { pos, rows, half, gate2: s2 * OBJECT_SPHERE_RAD }
    }

    /// A type-8 physics body of radius `radius` (+0x2c) [M FUN_001b8ae0 loop 2].
    pub fn body(pos: [f32; 3], rows: [[f32; 3]; 3], radius: f32) -> Obstacle {
        let half = [OBJECT_RADIUS_EX[0] * radius, OBJECT_RADIUS_EX[1] * radius, OBJECT_RADIUS_EX[2] * radius];
        Obstacle { pos, rows, half, gate2: radius * OBJECT_SPHERE_RAD }
    }
}

/// `FUN_001b8d08`: if `eye` is inside the obstacle's ellipsoid (half-extents `half` on its rows),
/// lift it to the ellipsoid's surface along the local up row: local y = sqrt((1 - x^2/rx^2 -
/// z^2/rz^2) * ry^2), back to world, world offset y made positive, + pos. None when outside
/// (any |local| >= its half-extent, or the ellipsoid sum >= 1) [M].
pub fn push_out(eye: [f32; 3], o: &Obstacle) -> Option<[f32; 3]> {
    let d = sub(eye, o.pos);
    let dot = |r: [f32; 3]| r[0] * d[0] + r[1] * d[1] + r[2] * d[2];
    let l = [dot(o.rows[0]), dot(o.rows[1]), dot(o.rows[2])];
    let [rx, ry, rz] = o.half;
    if !(l[0].abs() < rx && l[1].abs() < ry && l[2].abs() < rz) {
        return None;
    }
    let fx = l[0] * l[0] / (rx * rx);
    let fz = l[2] * l[2] / (rz * rz);
    if fx + l[1] * l[1] / (ry * ry) + fz >= 1.0 {
        return None;
    }
    let ly = ((1.0 - fx - fz) * ry * ry).sqrt();
    let mut w = add(add(scale(o.rows[0], l[0]), scale(o.rows[1], ly)), scale(o.rows[2], l[2]));
    w[1] = w[1].abs();
    Some(add(w, o.pos))
}

/// `FUN_001b8ae0`: each obstacle in order, gated by its sphere test, may move the eye [M].
pub fn push_out_all(mut eye: [f32; 3], obstacles: &[Obstacle]) -> [f32; 3] {
    for o in obstacles {
        let d = sub(o.pos, eye);
        if d[0] * d[0] + d[1] * d[1] + d[2] * d[2] <= o.gate2 {
            if let Some(e) = push_out(eye, o) {
                eye = e;
            }
        }
    }
    eye
}

/// The car as the camera reads it: the physics object's position (+0x10) and velocity (+0x20)
/// and its orientation rows (FUN_001d5b90: row 0 right, row 1 up, row 2 forward).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarPose {
    pub pos: [f32; 3],
    pub right: [f32; 3],
    pub up: [f32; 3],
    pub fwd: [f32; 3],
    pub vel: [f32; 3],
}

impl CarPose {
    /// A level car from the port's yaw (forward = (sin yaw, 0, cos yaw), as `physics::Car`).
    /// right = up x forward, the camera's own basis convention [M FUN_001d5700]; the car row 0
    /// sign is unproven and only reaches `Anchor_X` (0 on every land heli cam but two [D]).
    pub fn from_yaw(pos: [f32; 3], yaw: f32, vel: [f32; 3]) -> CarPose {
        let fwd = [yaw.sin(), 0.0, yaw.cos()];
        let up = [0.0, 1.0, 0.0];
        CarPose { pos, right: cross(up, fwd), up, fwd, vel }
    }
}

/// One update's result.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CamView {
    pub eye: [f32; 3],
    /// The look point (anchor + up*lookUp).
    pub look: [f32; 3],
    pub fwd: [f32; 3],
    pub up: [f32; 3],
    pub right: [f32; 3],
    /// camera +0xa4. The projection gets twice this (`FUN_001d4db0` -> `FUN_00295910`), whose x
    /// scale is 1/tan(that/2): 33.0 = a 66 degree wide view (the +0x120 viewport factor unproven).
    pub fov: f32,
}

/// The heli camera's kept state (the RPlayerCamera fields `FUN_001b2b48` carries).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChaseCamera {
    /// +0x2f0 - the eye offset from the anchor.
    pub offset: [f32; 3],
    /// +0xd0 - the smoothed anchor offset from the car.
    pub anchor_off: [f32; 3],
    /// +0x258 - the vertigo sine.
    pub vertigo: f32,
    /// +0x2c0 - the camera up target (moves only with upRate > 0).
    pub up_ref: [f32; 3],
    /// rows 0/1 of the camera matrix (the pull-in casts read last update's).
    pub right: [f32; 3],
    pub up: [f32; 3],
    /// `DAT_00333a38` / `DAT_00333a40` - the dip counter and its remembered hit.
    pub dip_count: i32,
    pub dip_point: [f32; 3],
    /// +0x21c / +0x25c - the arm length and follow rate of the last update.
    pub arm_len: f32,
    pub rate: f32,
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn len(a: [f32; 3]) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}
fn norm(a: [f32; 3]) -> [f32; 3] {
    let l = len(a);
    if l > 0.0 { scale(a, 1.0 / l) } else { a }
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn rot(car: &CarPose, v: [f32; 3]) -> [f32; 3] {
    add(add(scale(car.right, v[0]), scale(car.up, v[1])), scale(car.fwd, v[2]))
}

/// The arm's target offset from the anchor (steps 1-3) and the updated vertigo sine.
pub fn arm_target(p: &HeliParams, arm: &ArmParams, car: &CarPose, look_back: bool, vertigo: f32) -> ([f32; 3], f32) {
    let speed = len(car.vel);
    let (x, y, mut z) = (arm.sideways, arm.height, arm.distance);
    if look_back && p.look_back {
        z = -z;
    } else {
        z -= (speed * p.fallback_factor).min(p.max_fallback);
    }
    let fy = car.fwd[1];
    let k = if fy < 0.0 { p.max_vertigo_downhill } else { p.max_vertigo_uphill };
    let s = vertigo + (fy * k - vertigo) * p.vertigo_lerp;
    let c = (1.0 - s * s).max(0.0).sqrt();
    let (y, z) = (s * z + c * y, c * z - s * y);
    let fl = (car.fwd[0] * car.fwd[0] + car.fwd[2] * car.fwd[2]).sqrt();
    let (fx, fz) = if fl > 0.0 { (car.fwd[0] / fl, car.fwd[2] / fl) } else { (0.0, 1.0) };
    let right = [-fz, 0.0, fx];
    ([right[0] * x + fx * z, y, right[2] * x + fz * z], s)
}

/// The follow rate: clamp(|v| * Speed_Rate_Diff, Min_Rate, Max_Rate) at dt = 1/60 (the
/// `DAT_00339f50 * 60` factor is 1 there [M 0x1b3194-0x1b319c]).
pub fn follow_rate(p: &HeliParams, speed: f32) -> f32 {
    (speed.abs() * p.speed_rate_diff).min(p.max_rate).max(p.min_rate)
}

/// `FUN_001b8f80`: the 2-D edge sign on the dropped axis.
fn edge(axis: u8, p: [f32; 3], a: [f32; 3], b: [f32; 3]) -> f32 {
    match axis {
        2 => (a[1] - b[1]) * (p[0] - b[0]) - (p[1] - b[1]) * (a[0] - b[0]),
        1 => (a[0] - b[0]) * (p[2] - b[2]) - (p[0] - b[0]) * (a[2] - b[2]),
        _ => (a[2] - b[2]) * (p[1] - b[1]) - (p[2] - b[2]) * (a[1] - b[1]),
    }
}

/// `FUN_001b9020`: is `p` inside triangle (a, b, c), projected on the axis of the normal's
/// largest SIGNED component (no abs: the game's compares at 0x1b9098-0x1b90c4) [M].
pub fn in_triangle(p: [f32; 3], a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> bool {
    let n = cross(sub(a, b), sub(c, b));
    let mut axis = if n[0] < n[1] { 1 } else { 0 };
    let m = if n[0] < n[1] { n[1] } else { n[0] };
    if m < n[2] {
        axis = 2;
    }
    if edge(axis, p, a, b) <= 0.0 {
        if edge(axis, p, b, c) > 0.0 {
            return false;
        }
        edge(axis, p, c, a) <= 0.0
    } else {
        if edge(axis, p, b, c) < 0.0 {
            return false;
        }
        edge(axis, p, c, a) >= 0.0
    }
}

impl ChaseCamera {
    /// A camera settled on its target (the port's start state; the game's spawn-time camera
    /// state is unproven).
    pub fn new(p: &HeliParams, arm: &ArmParams, car: &CarPose) -> ChaseCamera {
        let fy = car.fwd[1];
        let settled = fy * if fy < 0.0 { p.max_vertigo_downhill } else { p.max_vertigo_uphill };
        let (target, vertigo) = arm_target(p, arm, car, false, settled);
        let up = [0.0, 1.0, 0.0];
        let f = norm(scale(target, -1.0));
        let right = norm(cross(up, f));
        ChaseCamera {
            offset: target,
            anchor_off: rot(car, arm.anchor),
            vertigo,
            up_ref: up,
            right,
            up: cross(f, right),
            dip_count: 0,
            dip_point: [0.0; 3],
            arm_len: len(target),
            rate: follow_rate(p, len(car.vel)),
        }
    }

    /// One camera update (`FUN_001b2b48`, normal path). `ray(from, to)` is the world segment
    /// cast (`FUN_002127f0`): the first hit point on from->to, or None. No obstacles.
    pub fn update(
        &mut self,
        p: &HeliParams,
        arm: &ArmParams,
        car: &CarPose,
        look_back: bool,
        ray: &mut dyn FnMut([f32; 3], [f32; 3]) -> Option<[f32; 3]>,
    ) -> CamView {
        self.update_full(p, arm, car, look_back, ray, &[])
    }

    /// [`ChaseCamera::update`] with the bodies `FUN_001b8ae0` pushes the eye out of (other
    /// vehicles, type-8 bodies; never the target itself), applied at the end of the pull-in.
    pub fn update_full(
        &mut self,
        p: &HeliParams,
        arm: &ArmParams,
        car: &CarPose,
        look_back: bool,
        ray: &mut dyn FnMut([f32; 3], [f32; 3]) -> Option<[f32; 3]>,
        obstacles: &[Obstacle],
    ) -> CamView {
        let (target, s) = arm_target(p, arm, car, look_back, self.vertigo);
        self.vertigo = s;
        // FUN_001d6498(cam, 1, arm): the anchor offset eases toward the car-frame anchor.
        let a_t = rot(car, arm.anchor);
        self.anchor_off = add(self.anchor_off, scale(sub(a_t, self.anchor_off), ANCHOR_LERP));
        let anchor = add(car.pos, self.anchor_off);
        self.arm_len = len(target);
        self.rate = follow_rate(p, len(car.vel));
        self.offset = scale(norm(self.offset), self.arm_len);
        self.offset = add(self.offset, scale(sub(target, self.offset), self.rate));
        let mut eye = add(anchor, self.offset);
        if p.check_collisions {
            eye = self.pull_in(p, car, eye, ray);
            // FUN_001b9198 ends with FUN_001b8ae0(cam, &eye) [M 0x1b9198 tail].
            eye = push_out_all(eye, obstacles);
        }
        // up target (+0x2c0): only upRate > 0 moves it [M 0x1b34f0..: lerp toward the car up].
        if p.up_rate > 0.0 {
            let fy = car.fwd[1].abs();
            let f = (-car.up[1]).max(0.0);
            let f = (fy + (1.0 - fy) * f * (1.0 - car.right[1].abs())).min(1.0);
            let t = norm(add([0.0, 1.0, 0.0], scale(sub(car.up, [0.0, 1.0, 0.0]), f)));
            self.up_ref = norm(add(self.up_ref, scale(sub(t, self.up_ref), p.up_rate)));
        }
        // FUN_001d5700: the look matrix.
        let look = add(anchor, scale(self.up_ref, p.look_up));
        let fwd = norm(sub(look, eye));
        let right = norm(cross(self.up_ref, fwd));
        let up = cross(fwd, right);
        self.right = right;
        self.up = up;
        CamView { eye, look, fwd, up, right, fov: p.default_fov }
    }

    /// `FUN_001b9198(cam, &eye, car.pos, 1)`: the pull-in and the dip.
    fn pull_in(
        &mut self,
        p: &HeliParams,
        car: &CarPose,
        eye: [f32; 3],
        ray: &mut dyn FnMut([f32; 3], [f32; 3]) -> Option<[f32; 3]>,
    ) -> [f32; 3] {
        let noisy = p.noise_amount > 0.0;
        let min_d = if noisy { MIN_DIST_NOISY } else { MIN_DIST };
        let lift = if noisy { 0.1 } else { PULL_LIFT };
        let o = car.pos;
        let d = sub(eye, o);
        let base = add(o, scale(d, RAY_SCALE));
        let e1 = add(base, scale(self.up, RAY_LOW));
        let e2 = add(add(e1, scale(self.up, RAY_HIGH_STEP)), scale(self.right, RAY_SIDE));
        let e3 = add(e2, scale(self.right, -2.0 * RAY_SIDE));
        let mut k = 1.0f32;
        let mut dip_ok = true;
        for e in [e1, e2, e3] {
            if let Some(h) = ray(o, e) {
                let mut hd = len(sub(h, o));
                let rl = len(sub(e, o));
                if hd > HIT_IGNORE {
                    if hd < min_d {
                        hd = min_d;
                    }
                    let r = hd / rl;
                    if r < k {
                        k = r;
                    }
                    if hd - min_d < DIP_GAP {
                        dip_ok = false;
                    }
                }
            }
        }
        let mut eye = eye;
        if k < 1.0 {
            eye = add(add(o, scale(d, k)), scale(self.up, lift * (1.0 - k)));
        }
        if dip_ok {
            let dl = len(sub(eye, o));
            let from = add(eye, scale(car.fwd, dl * DIP_FROM));
            let to = add(eye, scale(car.fwd, dl * DIP_TO));
            if let Some(h) = ray(from, to) {
                self.dip_point = h;
                if self.dip_count < DIP_MAX {
                    self.dip_count += 1;
                }
            }
            if self.dip_count > 0 {
                if in_triangle(self.dip_point, eye, to, o) {
                    if self.dip_count < DIP_MAX {
                        self.dip_count += 1;
                    }
                } else {
                    self.dip_count -= 1;
                }
                eye = add(eye, scale(self.up, self.dip_count as f32 * DIP_STEP));
            }
        }
        eye
    }
}

/// A minimal reader for camera.ini's text (`[Section]`, `key = value`, `//` and `;` tails):
/// the first number of `key` in `section`.
pub fn ini_f32(text: &str, section: &str, key: &str) -> Option<f32> {
    let mut cur = "";
    for line in text.lines() {
        let l = line.trim();
        if let Some(s) = l.strip_prefix('[') {
            cur = s.split(']').next().unwrap_or("").trim();
            continue;
        }
        if !cur.eq_ignore_ascii_case(section) {
            continue;
        }
        let Some((k, v)) = l.split_once('=') else { continue };
        if !k.trim().eq_ignore_ascii_case(key) {
            continue;
        }
        let v = v.trim();
        let end = v
            .char_indices()
            .find(|&(i, c)| !(c.is_ascii_digit() || c == '.' || ((c == '-' || c == '+') && i == 0)))
            .map_or(v.len(), |(i, _)| i);
        return v[..end].parse().ok();
    }
    None
}

/// The HeliCam pair of `section` from camera.ini's text, with the loader's defaults for the keys
/// a section omits [M FUN_001a49e0: floats 0, defaultFov 33, checkCollisions 1, others 0].
pub fn parse_heli(text: &str, section: &str) -> (HeliParams, ArmParams) {
    let g = |k: &str, d: f32| ini_f32(text, section, k).unwrap_or(d);
    let arm_sec = format!("{section}:Arm0");
    let a = |k: &str| ini_f32(text, &arm_sec, k).unwrap_or(0.0);
    (
        HeliParams {
            min_rate: g("Min_Rate", 0.0),
            max_rate: g("Max_Rate", 0.0),
            speed_rate_diff: g("Speed_Rate_Diff", 0.0),
            fallback_factor: g("Fallback_Factor", 0.0),
            max_fallback: g("Max_Fallback", 0.0),
            vertigo_lerp: g("Vertigo_Lerp", 0.0),
            max_vertigo_downhill: g("MaxVertigoDownhill", 0.0).clamp(0.0, 1.0),
            max_vertigo_uphill: g("MaxVertigoUphill", 0.0).clamp(0.0, 1.0),
            rigid_arm: g("rigidArm", 0.0) != 0.0,
            check_collisions: g("checkCollisions", 1.0) != 0.0,
            noise_amount: g("noiseAmount", 0.0),
            up_rate: g("upRate", 0.0),
            look_up: g("lookUp", 0.0),
            look_back: g("lookBack", 0.0) != 0.0,
            default_fov: g("defaultFov", 33.0),
        },
        ArmParams {
            sideways: a("Heli_Sideways"),
            height: a("Heli_Height"),
            distance: a("Heli_Distance"),
            anchor: [a("Anchor_X"), a("Anchor_Y"), a("Anchor_Z")],
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn race_start() -> CarPose {
        // RACE: the car at z 168.8 facing -z (yaw pi), at rest.
        CarPose::from_yaw([0.0, 0.0, 168.8], std::f32::consts::PI, [0.0; 3])
    }

    fn close(a: [f32; 3], b: [f32; 3], eps: f32) -> bool {
        (0..3).all(|i| (a[i] - b[i]).abs() < eps)
    }

    #[test]
    fn anycar_arm_sits_five_behind_and_1_15_up_at_rest() {
        let (p, arm) = ANYCAR_HELI_CAM0;
        let car = race_start();
        let mut cam = ChaseCamera::new(&p, &arm, &car);
        let v = cam.update(&p, &arm, &car, false, &mut |_, _| None);
        // anchor (0, 0.95, 0) + arm (0, 0.2, -5) on heading -z -> eye (0, 1.15, 173.8).
        assert!(close(v.eye, [0.0, 1.15, 173.8], 1e-3), "{:?}", v.eye);
        assert!(close(v.look, [0.0, 0.95, 168.8], 1e-3), "{:?}", v.look);
        assert_eq!(v.fov, 33.0);
        assert!(v.up[1] > 0.99, "{:?}", v.up);
    }

    #[test]
    fn vanquish_arm_is_5_1_behind_with_a_0_3_anchor() {
        let (p, arm) = default_heli_cam("Vanquish");
        assert_eq!(arm, VANQUISH_HELI_CAM0.1);
        let car = CarPose::from_yaw([0.0; 3], 0.0, [0.0; 3]);
        let mut cam = ChaseCamera::new(&p, &arm, &car);
        let v = cam.update(&p, &arm, &car, false, &mut |_, _| None);
        assert!(close(v.eye, [0.0, 1.15, -5.10], 1e-3), "{:?}", v.eye);
        assert_eq!(default_heli_cam("paradis_car"), ANYCAR_HELI_CAM0);
    }

    #[test]
    fn speed_pushes_the_arm_back_up_to_max_fallback() {
        let (p, arm) = ANYCAR_HELI_CAM0;
        let car = CarPose::from_yaw([0.0; 3], 0.0, [0.0, 0.0, 10.0]);
        let (t, _) = arm_target(&p, &arm, &car, false, 0.0);
        assert!((t[2] - (-5.0 - 0.4625)).abs() < 1e-5, "10*0.04625 extra: {t:?}");
        let fast = CarPose::from_yaw([0.0; 3], 0.0, [0.0, 0.0, 100.0]);
        let (t, _) = arm_target(&p, &arm, &fast, false, 0.0);
        assert!((t[2] - (-5.0 - 1.22)).abs() < 1e-5, "capped at Max_Fallback: {t:?}");
        // look-back swings the arm in front, no fallback.
        let (t, _) = arm_target(&p, &arm, &fast, true, 0.0);
        assert!((t[2] - 5.0).abs() < 1e-5, "{t:?}");
    }

    #[test]
    fn follow_rate_is_clamped_speed_times_diff() {
        let (p, _) = ANYCAR_HELI_CAM0;
        assert_eq!(follow_rate(&p, 0.0), 0.06);
        assert!((follow_rate(&p, 60.0) - 0.1).abs() < 1e-5);
        assert_eq!(follow_rate(&p, 400.0), 0.25);
    }

    #[test]
    fn a_yaw_step_is_followed_at_the_rate_not_snapped() {
        let (p, arm) = ANYCAR_HELI_CAM0;
        let car = CarPose::from_yaw([0.0; 3], 0.0, [0.0; 3]);
        let mut cam = ChaseCamera::new(&p, &arm, &car);
        let turned = CarPose::from_yaw([0.0; 3], std::f32::consts::FRAC_PI_2, [0.0; 3]);
        cam.update(&p, &arm, &turned, false, &mut |_, _| None);
        // offset (0, 0.2, -5) -> target (-5, 0.2, 0): one step at 0.06.
        let exp = [-5.0 * 0.06, 0.2, -5.0 * 0.94];
        assert!(close(cam.offset, exp, 1e-4), "{:?}", cam.offset);
    }

    #[test]
    fn the_anchor_eases_at_0_05() {
        let (p, arm) = ANYCAR_HELI_CAM0;
        let car = CarPose::from_yaw([0.0; 3], 0.0, [0.0; 3]);
        let mut cam = ChaseCamera::new(&p, &arm, &car);
        cam.anchor_off = [0.0; 3];
        cam.update(&p, &arm, &car, false, &mut |_, _| None);
        assert!((cam.anchor_off[1] - 0.95 * 0.05).abs() < 1e-6);
    }

    #[test]
    fn vertigo_tilts_the_arm_with_the_slope() {
        let (p, arm) = ANYCAR_HELI_CAM0;
        // nose up 0.2 (sin): s -> 0 + (0.2*1 - 0)*0.5 = 0.1.
        let f = [0.0, 0.2f32, (1.0f32 - 0.04).sqrt()];
        let car = CarPose { pos: [0.0; 3], right: cross([0.0, 1.0, 0.0], f), up: [0.0, 1.0, 0.0], fwd: f, vel: [0.0; 3] };
        let (t, s) = arm_target(&p, &arm, &car, false, 0.0);
        assert!((s - 0.1).abs() < 1e-6);
        let c = (1.0f32 - 0.01).sqrt();
        assert!((t[1] - (0.1 * -5.0 + c * 0.2)).abs() < 1e-5, "{t:?}");
        assert!((t[2] - (c * -5.0 - 0.1 * 0.2)).abs() < 1e-5, "{t:?}");
    }

    /// A wall plane z = `wz` (hits any segment crossing it).
    fn wall(wz: f32) -> impl FnMut([f32; 3], [f32; 3]) -> Option<[f32; 3]> {
        move |a: [f32; 3], b: [f32; 3]| {
            if (a[2] - wz) * (b[2] - wz) >= 0.0 {
                return None;
            }
            let t = (wz - a[2]) / (b[2] - a[2]);
            Some([a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, wz])
        }
    }

    /// The three cast ends (relative to the car) for eye offset `d` and camera basis (up, right).
    fn cast_ends(d: [f32; 3], up: [f32; 3], right: [f32; 3]) -> [[f32; 3]; 3] {
        let b = scale(d, 1.075);
        let e1 = add(b, scale(up, -0.5));
        let e2 = add(add(b, scale(up, 0.5)), scale(right, -0.4));
        let e3 = add(add(b, scale(up, 0.5)), scale(right, 0.4));
        [e1, e2, e3]
    }

    #[test]
    fn a_wall_behind_the_car_pulls_the_eye_in_front_of_it() {
        let (p, arm) = ANYCAR_HELI_CAM0;
        let car = race_start();
        let mut cam = ChaseCamera::new(&p, &arm, &car);
        let (u, r) = (cam.up, cam.right);
        let mut w = wall(173.0);
        let v = cam.update(&p, &arm, &car, false, &mut w);
        assert!(v.eye[2] < 173.0, "eye {:?} still behind the wall", v.eye);
        // the least t = 4.2 / (cast end z) of the three casts is the scale.
        let d = [0.0, 1.15, 5.0];
        let k = cast_ends(d, u, r).iter().map(|e| 4.2 / e[2]).fold(1.0f32, f32::min);
        let exp = add(add([0.0, 0.0, 168.8], scale(d, k)), scale(u, 0.5 * (1.0 - k)));
        assert!(close(v.eye, exp, 1e-3), "{:?} vs {exp:?}", v.eye);
        assert!(k < 0.8 && k > 0.75, "k {k}");
        assert_eq!(cam.dip_count, 0);
    }

    #[test]
    fn a_hit_nearer_than_1_1_is_ignored_and_the_floor_is_2_5() {
        let (p, arm) = ANYCAR_HELI_CAM0;
        let car = CarPose::from_yaw([0.0; 3], 0.0, [0.0; 3]);
        let mut cam = ChaseCamera::new(&p, &arm, &car);
        let u = cam.up;
        let v = cam.update(&p, &arm, &car, false, &mut wall(-1.0));
        // no pull-in; the dip cast (eye -> car) crosses the wall: +1 for the hit, +1 for the
        // remembered point in the triangle -> the eye drops 2*0.0125 along the camera up.
        let exp = add([0.0, 1.15, -5.0], scale(u, -0.025));
        assert!(close(v.eye, exp, 1e-4), "a 1.0 hit is ignored: {:?} vs {exp:?}", v.eye);
        assert_eq!(cam.dip_count, 2);
        let mut cam = ChaseCamera::new(&p, &arm, &car);
        let (u, r) = (cam.up, cam.right);
        let v = cam.update(&p, &arm, &car, false, &mut wall(-1.5));
        // every hit is raised to 2.5; the longest cast gives the least scale.
        let d = [0.0, 1.15, -5.0];
        let k = cast_ends(d, u, r).iter().map(|e| 2.5 / len(*e)).fold(1.0f32, f32::min);
        let exp = add(scale(d, k), scale(u, 0.5 * (1.0 - k)));
        assert!(close(v.eye, exp, 1e-4), "{:?} vs {exp:?}", v.eye);
        assert_eq!(cam.dip_count, 0, "a hit at the floor disables the dip");
    }

    #[test]
    fn the_triangle_test_uses_the_signed_axis_pick() {
        // normal +z -> the xy projection: a real inside/outside test.
        let (a, b, c) = ([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]);
        assert!(in_triangle([0.2, 0.2, 0.0], a, b, c));
        assert!(!in_triangle([0.9, 0.9, 0.0], a, b, c));
        // the other winding (normal -z): no component beats 0, so axis 0 (the yz projection),
        // where the flat triangle is a line and every point passes.
        let (b, c) = (c, b);
        assert!(in_triangle([0.9, 0.9, 0.0], a, b, c));
    }

    /// A vertical wall quad at z = `wz` (x -10..10, y -5..5) as two triangles of `class`.
    fn wall_tris(wz: f32, class: u8) -> Vec<CamTri> {
        let (a, b, c, d) = ([-10.0, -5.0, wz], [10.0, -5.0, wz], [10.0, 5.0, wz], [-10.0, 5.0, wz]);
        vec![CamTri { v: [a, b, c], class }, CamTri { v: [a, d, c], class }]
    }

    #[test]
    fn the_camera_world_hits_walls_either_winding_and_keeps_the_nearest() {
        let mut tris = wall_tris(-3.0, 0);
        tris.extend(wall_tris(-4.0, 0));
        let w = CamWorld::new(tris);
        let h = w.cast([0.0, 1.0, 0.0], [0.0, 1.0, -5.0]).expect("a vertical wall is in the cast set");
        assert!(close(h, [0.0, 1.0, -3.0], 1e-5), "{h:?}");
        // the other way round meets the far wall first (both windings count).
        let h = w.cast([0.0, 1.0, -5.0], [0.0, 1.0, 0.0]).expect("hit");
        assert!(close(h, [0.0, 1.0, -4.0], 1e-5), "{h:?}");
        assert_eq!(w.cast([0.0, 1.0, 0.0], [0.0, 1.0, -2.9]), None, "the segment ends short of it");
    }

    #[test]
    fn the_world_class_mask_skips_matching_triangles() {
        let mut w = CamWorld::new(wall_tris(-3.0, 0x04));
        assert!(w.cast([0.0, 1.0, 0.0], [0.0, 1.0, -5.0]).is_some(), "mask 0: the class-4 wall counts");
        w.class_mask = 0x04;
        assert_eq!(w.cast([0.0, 1.0, 0.0], [0.0, 1.0, -5.0]), None);
    }

    #[test]
    fn a_wall_in_the_camera_world_pulls_the_eye_in() {
        let (p, arm) = ANYCAR_HELI_CAM0;
        let car = race_start();
        let mut cam = ChaseCamera::new(&p, &arm, &car);
        let (u, r) = (cam.up, cam.right);
        let w = CamWorld::new(wall_tris(173.0, 0));
        let v = cam.update(&p, &arm, &car, false, &mut |a, b| w.cast(a, b));
        let d = [0.0, 1.15, 5.0];
        let k = cast_ends(d, u, r).iter().map(|e| 4.2 / e[2]).fold(1.0f32, f32::min);
        let exp = add(add([0.0, 0.0, 168.8], scale(d, k)), scale(u, 0.5 * (1.0 - k)));
        assert!(close(v.eye, exp, 1e-3), "{:?} vs {exp:?}", v.eye);
    }

    const ID: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    #[test]
    fn push_out_lifts_the_eye_to_the_ellipsoid_top() {
        // a type-8 body of radius 1: half-extents 1.5 on every row.
        let o = Obstacle::body([0.0; 3], ID, 1.0);
        assert_eq!(o.half, [1.5, 1.5, 1.5]);
        let ly = (2.25f32 - 0.25 - 0.09).sqrt();
        let e = push_out([0.5, 0.2, 0.3], &o).expect("inside");
        assert!(close(e, [0.5, ly, 0.3], 1e-5), "{e:?}");
        // below the centre it still goes to the top (the world offset y is made positive).
        let e = push_out([0.5, -0.2, 0.3], &o).expect("inside");
        assert!(close(e, [0.5, ly, 0.3], 1e-5), "{e:?}");
        // an upside-down body: its up row points down, the offset y is still made positive.
        let flipped = Obstacle::body([0.0; 3], [[1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, -1.0]], 1.0);
        let e = push_out([0.5, 0.2, 0.3], &flipped).expect("inside");
        assert!(close(e, [0.5, ly, 0.3], 1e-5), "{e:?}");
        assert_eq!(push_out([1.2, 0.0, 1.2], &o), None, "inside the box, outside the ellipsoid");
        assert_eq!(push_out([1.6, 0.0, 0.0], &o), None);
    }

    #[test]
    fn push_out_runs_in_the_bodys_frame() {
        // a vehicle of size (1, 1, 2) facing +x: half-extents (1.5, 1.5, 3) on (right, up, fwd).
        let pose = CarPose::from_yaw([10.0, 0.0, 0.0], std::f32::consts::FRAC_PI_2, [0.0; 3]);
        let rows = [pose.right, pose.up, pose.fwd];
        let o = Obstacle::vehicle(pose.pos, rows, [1.0, 1.0, 2.0]);
        assert!((o.gate2 - 6.0 * OBJECT_SPHERE_RAD).abs() < 1e-5);
        let e = push_out_all([12.9, 0.0, 0.0], &[o]);
        let ly = ((1.0f32 - 2.9 * 2.9 / 9.0) * 2.25).sqrt();
        assert!(close(e, [12.9, ly, 0.0], 1e-4), "2.9 along the car's forward is inside: {e:?}");
        // the same offset across the car is outside (half-extent 1.5 there).
        assert_eq!(push_out_all([10.0, 0.0, 2.9], &[o]), [10.0, 0.0, 2.9]);
    }

    #[test]
    fn the_body_gate_compares_the_radius_unsquared() {
        // radius 4: half-extents 6, gate |d|^2 <= 4 * 2.598 = 10.39. At 3.3 (|d|^2 10.89) the eye
        // is inside the ellipsoid but past the gate, so it stays.
        let o = Obstacle::body([0.0; 3], ID, 4.0);
        assert!(push_out([3.3, 0.0, 0.0], &o).is_some(), "inside the ellipsoid");
        assert_eq!(push_out_all([3.3, 0.0, 0.0], &[o]), [3.3, 0.0, 0.0]);
        let e = push_out_all([3.2, 0.0, 0.0], &[o]);
        assert!(e[1] > 4.0, "3.2 (|d|^2 10.24) passes the gate: {e:?}");
    }

    #[test]
    fn another_car_at_the_eye_lifts_it_but_no_obstacle_leaves_it() {
        let (p, arm) = ANYCAR_HELI_CAM0;
        let car = race_start();
        let mut a = ChaseCamera::new(&p, &arm, &car);
        let mut b = a;
        let plain = a.update(&p, &arm, &car, false, &mut |_, _| None);
        let other = CarPose::from_yaw([0.0, 0.0, 173.8], std::f32::consts::PI, [0.0; 3]);
        let o = Obstacle::vehicle(other.pos, [other.right, other.up, other.fwd], [1.0, 0.7, 2.2]);
        let v = b.update_full(&p, &arm, &car, false, &mut |_, _| None, &[o]);
        assert!(close(plain.eye, [0.0, 1.15, 173.8], 1e-3));
        // local y = 1.15 is past the 0.7*1.5 = 1.05 half height: outside, no push.
        assert!(close(v.eye, plain.eye, 1e-6), "{:?}", v.eye);
        let tall = Obstacle::vehicle(other.pos, [other.right, other.up, other.fwd], [1.0, 1.0, 2.2]);
        let mut c = ChaseCamera::new(&p, &arm, &car);
        let v = c.update_full(&p, &arm, &car, false, &mut |_, _| None, &[tall]);
        let ly = (1.0f32 * 2.25).sqrt();
        assert!(close(v.eye, [0.0, ly, 173.8], 1e-3), "lifted to the top, 1.5: {:?}", v.eye);
    }

    #[test]
    fn the_near_plane_is_0_17() {
        assert_eq!(NEAR_PLANE.to_bits(), 0x3e2e147b);
        assert_eq!(FAR_PLANE.to_bits(), 0x453b8000);
        assert_eq!(OBJECT_SPHERE_RAD_DEFAULT.to_bits(), 0x40380000);
        assert_eq!(OBJECT_RADIUS_EX_DEFAULT.map(f32::to_bits), [0x3fd66666, 0x3ff66666, 0x3fa9999a]);
    }

    #[test]
    fn the_ini_reader_takes_the_first_number() {
        let t = "[X:Arm0]\nHeli_Distance = -5.10 // -4.75\nrelPosLerp = 0.05; \n";
        assert_eq!(ini_f32(t, "X:Arm0", "Heli_Distance"), Some(-5.10));
        assert_eq!(ini_f32(t, "X:Arm0", "relPosLerp"), Some(0.05));
    }
}
