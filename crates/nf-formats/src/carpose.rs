//! Which `vd` draw list the driving engine draws a vehicle with, and the run-time matrices of
//! its parts (docs/formats/car-model.md §11). Pure functions of the vehicle state; evidence tags
//! [M] DRIVING.ELF code (EE address), [D] disc data.
//!
//! Matrices are the game's: row-major 4x4, row vectors (`p' = p·M`), rows 0-2 the images of the
//! local axes, row 3 the translation. `mul(a, b)` is `FUN_002cbb88(a, b, dst)`: `dst = a·b`, `a`
//! applied first [M 0x2cbb88: row i of dst = Σ_j a[i][j]·b[j]].
//!
//! The draw of one item [M FUN_001ad498 → FUN_001beee0 + table 0x333a60]: `local = kind(item)`,
//! then `world = local · car`. [`part_matrix`] returns `local` for the kinds the disc uses:
//!
//! | `+0x0d` | function | local matrix |
//! |---|---|---|
//! | 0xff | `FUN_001bf140` | own |
//! | 0xfe | `FUN_001bf600` | `skel[+0x0e − 1] · own` (`FUN_001bef68`: index past the count → 0) |
//! | 0xfc | `FUN_001bf3d8` | `R(v/256 · frame degrees, +y) · own` |
//! | 0xfb | `FUN_001bf458` | `R(v/256 · frame degrees, +z) · own` |
//! | 0xef | `FUN_001bf598` | `R(dials[v] degrees, +z) · own` (`FUN_001befc8`: v past 12 → dials[0]) |
//! | 0xf3 | `FUN_001bf338` → `FUN_001a30c0` | own (the animated branch needs `+0x0d` < 100) |
//! | 0x64 | `FUN_001bf5f8` | not drawn (returns 0) |
//! | 0xee | `FUN_001bfc20` | camera-facing; not ported |
//!
//! `v` is the item's `+0x0e`, or the u16 at `+2` of its run-time entry when `+0x0c & 0x10`
//! [M FUN_001bf048]; the entry table is not decoded, so such items are [`PartMatrix::Unported`].
//! `frame` is `DAT_00334000 = (float)DAT_00339f54` [M FUN_001d1ef8], the sim tick counter
//! (sim +0xdc4, +1 per tick unless sim +0xdb4 == 3 [M FUN_001fef20]).

use crate::carmesh::{Attach, CarMesh, CarPart};

pub type Mat4 = [[f32; 4]; 4];

pub const IDENTITY: Mat4 = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];

/// `FUN_002cbb88(a, b, dst)`: `a·b` (`a` applied first) [M 0x2cbb88].
pub fn mul(a: &Mat4, b: &Mat4) -> Mat4 {
    let mut d = [[0.0; 4]; 4];
    for i in 0..4 {
        for k in 0..4 {
            d[i][k] = a[i][0] * b[0][k] + a[i][1] * b[1][k] + a[i][2] * b[2][k] + a[i][3] * b[3][k];
        }
    }
    d
}

/// A point through a matrix (row vector, w = 1).
pub fn transform(m: &Mat4, p: [f32; 3]) -> [f32; 3] {
    [
        p[0] * m[0][0] + p[1] * m[1][0] + p[2] * m[2][0] + m[3][0],
        p[0] * m[0][1] + p[1] * m[1][1] + p[2] * m[2][1] + m[3][1],
        p[0] * m[0][2] + p[1] * m[1][2] + p[2] * m[2][2] + m[3][2],
    ]
}

/// Rotation by `rad` about the unit `axis`, the game's row layout (`FUN_002d11c8` rows 0-2:
/// `[x²t+c, xyt+sz, xzt−sy]`, `[xyt−sz, y²t+c, yzt+sx]`, `[xzt+sy, yzt−sx, z²t+c]`, t = 1 − c).
fn rot(rad: f32, axis: [f32; 3]) -> Mat4 {
    let (s, c) = rad.sin_cos();
    let t = 1.0 - c;
    let [x, y, z] = axis;
    [
        [x * x * t + c, x * y * t + s * z, x * z * t - s * y, 0.0],
        [x * y * t - s * z, y * y * t + c, y * z * t + s * x, 0.0],
        [x * z * t + s * y, y * z * t - s * x, z * z * t + c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// `FUN_002d11c8(deg, x, y, z, out)`: `deg` in degrees (`× 1/360 × 2π`), axis normalised, identity
/// when `deg/360 == 0` [M 0x2d11c8].
pub fn rot_axis_degrees(deg: f32, axis: [f32; 3]) -> Mat4 {
    if deg * 0.002_777_777_8 == 0.0 {
        return IDENTITY;
    }
    let n = 1.0 / (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    rot(deg * 0.002_777_777_8 * 6.283_185_5, [axis[0] * n, axis[1] * n, axis[2] * n])
}

/// `FUN_002cc590(turns, out)`: about +y, `turns` of a full circle (its sin/cos `FUN_002cbd50` /
/// `FUN_002cbeb0` take turns: `× 2π` after the wrap), translation zero [M 0x2cc590].
pub fn rot_y_turns(turns: f32) -> Mat4 { rot(turns * std::f32::consts::TAU, [0.0, 1.0, 0.0]) }

/// `FUN_002cc510(turns, out)`: about +x, turns, translation zero [M 0x2cc510].
pub fn rot_x_turns(turns: f32) -> Mat4 { rot(turns * std::f32::consts::TAU, [1.0, 0.0, 0.0]) }

// ---------------------------------------------------------------------------------------------
// The draw list.

/// Which object updates the vehicle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawOwner {
    /// A `pvehicle` (vtables 0x379ef0 / 0x37a9f8, both run `FUN_00188200`), with its driver kind
    /// `+0x2c8`: 0 = the player's car [M FUN_00176658 :231 passes 0], 2 / 4 (stored as 2) = script
    /// spawns [M FUN_0012cb08 :202-206, FUN_001862c0 :170-174], 3 = `FUN_0013fae0` :162 spawns.
    Vehicle { driver_kind: u32 },
    /// An `AHelicopter` (vtable 0x37a4e8, `FUN_00193408`): its update never calls the select
    /// slot, so `+0x348` stays 0 and every item draws [M: the only vtable +0x14 calls through
    /// a vehicle's +0x3c vtable are at 0x188424 / 0x188448, raw scan of the ELF].
    Helicopter,
}

/// What `FUN_00188200` reads to pick the list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawState {
    pub owner: DrawOwner,
    /// The current camera mode's `interiorView` (mode record byte +7, `*(DAT_00333ed8+4)+0x120`;
    /// camera.ini `interiorView`, [M FUN_001a49e0 :271]). Set on `[VanquishDashboardCam0]` and
    /// `[BumperCam]` [D camera.ini].
    pub interior_view: bool,
    /// `DAT_0033a024 + 0x350`: the AutoDrive script event's flag (`10EAutoDrive`, `FUN_00148220` →
    /// `FUN_00206398`) [M].
    pub auto_drive: bool,
}

/// The `vd` list `FUN_00188200` selects each update [M 0x1883b4-0x18844c]: driver kind < 2 →
/// 2 when `interiorView`, else 5 when AutoDrive is on, else 1; any other kind → 1. The call
/// goes through vtable slot +0x14 = `FUN_001d2618`, which stores `FUN_001c8ba8`'s `vd#N` payload
/// at +0x348 (0 = no such record, every item draws). `None` = nothing is ever selected (the
/// constructor's 0 [M FUN_001d2338]). Lists 3 and 4 are never selected [M: no other call site].
pub fn selected_list(s: &DrawState) -> Option<u16> {
    match s.owner {
        DrawOwner::Helicopter => None,
        DrawOwner::Vehicle { driver_kind } if driver_kind < 2 => Some(if s.interior_view {
            2
        } else if s.auto_drive {
            5
        } else {
            1
        }),
        DrawOwner::Vehicle { .. } => Some(1),
    }
}

/// The player's car is not drawn at all while the camera mode's type is 1 (`[BumperCam]`: the
/// type the loader gives a section named `Bumper`, mode record +0x0a [M FUN_001a49e0 :641]), or
/// while a transition runs (`+0x12c` > 0) to/from a type-1 mode (`+0x128`) [M FUN_001d35d8
/// 0x1d3650-0x1d369c]. Other cars are always drawn.
pub fn player_car_hidden(mode_is_bumper: bool, transition_mode_is_bumper: bool, transition_running: bool) -> bool {
    mode_is_bumper || (transition_running && transition_mode_is_bumper)
}

// ---------------------------------------------------------------------------------------------
// The dials (`car + 0x40`, 12 floats, the 0xef table).

/// One row of the needle table at 0x3340f0: `{in_min, in_max, base, range}` [D ELF .data].
pub const DIAL_ROWS: [[f32; 4]; 4] = [[0.0, 1.0, 0.0, 0.0], [0.0, 330.0, 120.0, -225.0], [0.0, 7000.0, 121.0, -210.0], [0.0, 1.0, 0.0, 0.0]];

/// `FUN_001d3440(v, car, row)`: `base + (clamp(v, min, max) − min)/(max − min) · range` [M].
pub fn dial_map(v: f32, row: usize) -> f32 {
    let [lo, hi, base, range] = DIAL_ROWS[row];
    let v = v.max(lo).min(hi);
    base + ((v - lo) / (hi - lo)) * range
}

/// The dial table `FUN_001d34a8` writes each model update [M]: slot 0 = the front-left steer
/// angle (turns, [`WheelRig::update`]'s return) × −750; for the player's car only, slot 1 =
/// `dial_map(speed · 3.6 · 1.5, 1)` with speed 0 while the velocity points backwards
/// (`vel · fwd <= 0`), slot 2 = `dial_map(rpm, 2)` (rpm = `FUN_0018aa88`). Slots 3-11 keep the
/// constructor's 0 (no writer found).
pub fn dials(steer0: f32, player: Option<(f32, bool, f32)>, prev: &[f32; 12]) -> [f32; 12] {
    let mut d = *prev;
    d[0] = steer0 * -750.0;
    if let Some((speed, forward, rpm)) = player {
        d[1] = dial_map(if forward { speed * 3.6 * 1.5 } else { 0.0 }, 1);
        d[2] = dial_map(rpm, 2);
    }
    d
}

// ---------------------------------------------------------------------------------------------
// The wheel rig (`FUN_001d2648` setup, `FUN_001d2cb0` per model update).

/// The `pvehicle` attributes the rig reads [M FUN_001968f8 registrations].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WheelAttrs {
    /// `SPRING_REST_LENGTH` (+0x60, default 2.0).
    pub rest: f32,
    /// `SPRING_COMPRESSION_LIMIT_DRAW` (+0x68, default 2.0).
    pub limit_draw: f32,
    /// `WHEEL_RADIUS` (+0x6c, default 1.0).
    pub wheel_radius: f32,
    /// `IS_SUB` (+0xbc): the hub slots hold the fins and the fin terms run.
    pub is_sub: bool,
}

/// The four wheel slots, in the order of the name tables at 0x334070 / 0x334080 / 0x334090:
/// left front, right front, left rear, right rear [M FUN_001d2648].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WheelRig {
    /// 1-based `Skel` index of each wheel (0 = name not found). A car without a `Skel` record
    /// gets 4 run-time matrices and wheels 1..4 [M 0x1d2684-0x1d26a4, 0x1d2778-0x1d2784].
    pub wheel: [u16; 4],
    /// 1-based `Skel` index of each hub (`leftfronthub`…; `leftfin`… on an `IS_SUB` vehicle).
    pub hub: [u16; 4],
    /// `car + 0x238`: the rest offset of each wheel, `origin_y − wheel_ref_height` of the item
    /// the scan found, else 0.
    pub off: [f32; 4],
    /// The number of run-time matrices (`car + 0x78`).
    pub skel_count: usize,
}

const WHEEL_NAMES: [&str; 4] = ["leftfrontwheel", "rightfrontwheel", "leftrearwheel", "rightrearwheel"];
const HUB_NAMES: [&str; 4] = ["leftfronthub", "rightfronthub", "leftrearhub", "rightrearhub"];
const FIN_NAMES: [&str; 4] = ["leftfin", "rightfin", "leftrudder", "rightrudder"];

impl WheelRig {
    /// `FUN_001d2648`. `sub_fins` = vehicle +0x40 == 1 and `IS_SUB`. The item scan for the wheel
    /// offset does not restart per wheel: one pointer walks on from the item after the previous
    /// match (or `count` items further), `count` items per wheel [M 0x1d26c4 sets it once,
    /// 0x1d27d0 advances it in every iteration's delay slot]. Items past the end of `in#0` are
    /// memory this decoder does not have; they count as no match.
    pub fn new(mesh: &CarMesh, sub_fins: bool) -> WheelRig {
        let find = |name: &str| mesh.skel.iter().position(|s| s == name).map_or(0, |i| i as u16 + 1);
        let no_skel = mesh.skel.is_empty();
        let href = mesh.wheel_ref_height().unwrap_or(0.0);
        let mut rig = WheelRig { wheel: [0; 4], hub: [0; 4], off: [0.0; 4], skel_count: if no_skel { 4 } else { mesh.skel.len() } };
        let n = mesh.parts.len();
        let mut at = 0usize;
        for k in 0..4 {
            rig.hub[k] = find(if sub_fins { FIN_NAMES[k] } else { HUB_NAMES[k] });
            rig.wheel[k] = if no_skel { k as u16 + 1 } else { find(WHEEL_NAMES[k]) };
            for _ in 0..n {
                let idx = at;
                at += 1;
                let Some(p) = mesh.parts.get(idx) else { continue };
                if let Attach::Skel(s) = p.attach {
                    if s.wrapping_add(1) == rig.wheel[k] {
                        rig.off[k] = p.origin[1] - href;
                        break;
                    }
                }
            }
        }
        rig
    }
}

/// The vehicle state the rig reads each update.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WheelInput {
    /// Vehicle +0x230 / +0x234: the front and rear axle roll, turns in [0, 1] (vtbl+0x194 =
    /// `FUN_00193178`). See [`spin_step`].
    pub spin: [f32; 2],
    /// Vehicle +0x28c (vtbl+0xcc = `FUN_00193190`): the steer angle, turns.
    pub steer: f32,
    /// Vehicle +0x210 + 4k (vtbl+0x1ec = `FUN_00193198`): each wheel's spring compression.
    pub compression: [f32; 4],
    /// The 0.002 steer rate: driver kind (vtbl+0x9c = +0x2c8) 0 or 1 and the camera mode's
    /// `interiorView`; else 0.01 [M 0x1d2d50-0x1d2dc8].
    pub slow_steer: bool,
    /// The fin inputs of an `IS_SUB` vehicle: vehicle +0x244 and +0x254 (vtbl+0x134 copies
    /// +0x240..+0x25f), and vehicle +0x29c != 0 (vtbl+0x1dc), which zeroes the second.
    pub fins: Option<(f32, f32, bool)>,
}

/// The rig's own memory on the draw object: +0x328 steer, +0x32c / +0x338 the fin terms
/// (0 from `FUN_001d2338`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WheelMemory {
    pub steer: f32,
    pub fin_pitch: f32,
    pub fin_yaw: f32,
}

impl WheelRig {
    /// `FUN_001d2cb0` [M 0x1d2cb0-0x1d30e8]: writes the hub and wheel matrices into `skel`
    /// (index n = `Skel(n)`) and returns the front-left angle (the steering-wheel dial input).
    /// Only runs for a vehicle whose +0x40 word is 1 (else it returns 0 and writes nothing).
    ///
    /// - steer `s` = clamp(input, ±0.075), moved at most `rate` (0.01, or 0.002) from last time;
    /// - angles = `[s, −s, 0, 0]` (a mirrored right wheel turns the same way once its own matrix
    ///   negates x); an `IS_SUB` adds the fin terms;
    /// - per wheel k: hub = `R_y(angle[k])` with translation y =
    ///   `WHEEL_RADIUS + min(compression[k], LIMIT_DRAW) − off[k] − REST`; wheel =
    ///   `R_x(spin[k < 2 ? front : rear]) · hub`. A slot whose index is 0 writes a scratch
    ///   matrix instead.
    pub fn update(&self, attrs: &WheelAttrs, input: &WheelInput, mem: &mut WheelMemory, skel: &mut [Mat4]) -> f32 {
        let rate = if input.slow_steer { 0.002 } else { 0.01 };
        let prev = mem.steer;
        let mut s = input.steer.min(0.075).max(-0.075);
        let d = s - prev;
        if rate < d {
            s = prev + rate;
        }
        if d < -rate {
            s = prev - rate;
        }
        let mut angle = [s, -s, 0.0, 0.0];
        if let (true, Some((a_in, b_in, b_zero))) = (attrs.is_sub, input.fins) {
            let mut a = a_in;
            let da = a_in - mem.fin_pitch;
            if 0.075 < da {
                a = mem.fin_pitch + 0.075;
            }
            if da < -0.075 {
                a = mem.fin_pitch - 0.075;
            }
            let mut b = if b_zero { 0.0 } else { b_in };
            let db = b - mem.fin_yaw;
            if 0.1 < db {
                b = mem.fin_yaw + 0.1;
            }
            if db < -0.1 {
                b = mem.fin_yaw - 0.1;
            }
            mem.fin_yaw = b;
            mem.fin_pitch = a;
            angle[0] += a * 0.07;
            angle[1] += a * 0.07;
            angle[3] = -s + b * 0.15;
            angle[2] = -s - b * 0.15;
        }
        mem.steer = s;
        let spin = [input.spin[0], input.spin[0], input.spin[1], input.spin[1]];
        for k in 0..4 {
            let mut hub = rot_y_turns(angle[k]);
            let x = input.compression[k];
            let v = if attrs.limit_draw < x { attrs.limit_draw } else { x };
            hub[3][1] = ((attrs.wheel_radius + v) - self.off[k]) - attrs.rest;
            let wheel = mul(&rot_x_turns(spin[k]), &hub);
            if self.hub[k] != 0 {
                if let Some(m) = skel.get_mut(self.hub[k] as usize - 1) {
                    *m = hub;
                }
            }
            if self.wheel[k] != 0 {
                if let Some(m) = skel.get_mut(self.wheel[k] as usize - 1) {
                    *m = wheel;
                }
            }
        }
        angle[0]
    }
}

/// The game's wrap of an axle roll: above 1 subtract its integer part, below 0 subtract
/// (integer part − 1); exactly 1.0 stays [M FUN_0018ecb8 :532-551, FUN_0018cef8 :784-801].
pub fn wrap_turns(v: f32) -> f32 {
    if 1.0 < v {
        v - (v as i32) as f32
    } else if v < 0.0 {
        v - ((v as i32) - 1) as f32
    } else {
        v
    }
}

/// The plain roll step: both axles += the vehicle's per-tick step (+0x274, the
/// `speed / (WHEEL_RADIUS·6.32·fps)` term of docs/systems/drive-suspension.md §3), then wrapped
/// [M FUN_0018ecb8 :532-551]. The player pass `FUN_0018cef8` :768-782 varies it on flags whose
/// meaning is not traced (front only ×0.1, rear += 0.013 instead).
pub fn spin_step(spin: [f32; 2], step: f32) -> [f32; 2] { [wrap_turns(spin[0] + step), wrap_turns(spin[1] + step)] }

// ---------------------------------------------------------------------------------------------
// The per-item matrix.

/// The run-time inputs of the item matrices.
#[derive(Clone, Debug, PartialEq)]
pub struct CarPose {
    /// `DAT_00334000`: the sim tick counter as f32.
    pub frame: f32,
    /// The run-time `Skel` matrices (`car + 0x7c`, count `car + 0x78`); identity after
    /// `FUN_001cc8e8`, the wheels and hubs rewritten by [`WheelRig::update`].
    pub skel: Vec<Mat4>,
    /// `car + 0x40`: [`dials`].
    pub dials: [f32; 12],
}

impl CarPose {
    /// The pose after setup: identity `Skel` matrices (one per `Skel` entry, 4 without one), zero
    /// dials [M FUN_001cc8e8, FUN_001cc568, FUN_001d2648].
    pub fn rest(mesh: &CarMesh) -> CarPose {
        let n = if mesh.skel.is_empty() { 4 } else { mesh.skel.len() };
        CarPose { frame: 0.0, skel: vec![IDENTITY; n], dials: [0.0; 12] }
    }
}

/// What the game does with one item.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PartMatrix {
    /// Drawn with this local → car matrix (then × the car's matrix).
    Drawn(Mat4),
    /// Skipped: `+0x0c & 1`, or a kind whose function returns 0.
    NotDrawn,
    /// A kind or flag this module does not evaluate (the camera-facing 0xee, the run-time
    /// entry of `+0x0c & 0x10`, the kinds no disc file uses).
    Unported(u8),
}

/// The item's own matrix (`FUN_001bf140`: words 3/7/11 zeroed, word 15 = 1).
pub fn own_matrix(p: &CarPart) -> Mat4 {
    let a = &p.axes;
    [
        [a[0][0], a[0][1], a[0][2], 0.0],
        [a[1][0], a[1][1], a[1][2], 0.0],
        [a[2][0], a[2][1], a[2][2], 0.0],
        [p.origin[0], p.origin[1], p.origin[2], 1.0],
    ]
}

/// The item's local → car matrix in `pose` [M FUN_001ad498, FUN_001beee0, table 0x333a60; the
/// per-kind rules are in the module doc].
pub fn part_matrix(p: &CarPart, pose: &CarPose) -> PartMatrix {
    if p.hidden() {
        return PartMatrix::NotDrawn;
    }
    let own = own_matrix(p);
    let (kind, link) = match p.attach {
        Attach::Fixed => return PartMatrix::Drawn(own),
        Attach::Skel(n) => (0xfe, n.wrapping_add(1)),
        Attach::Other(k, l) => (k, l),
    };
    if p.flags & 0x10 != 0 && matches!(kind, 0xef | 0xfb | 0xfc | 0xee) {
        return PartMatrix::Unported(kind);
    }
    match kind {
        0xfe => {
            // FUN_001bef68(link − 1): an index past the count reads matrix 0; with no matrices
            // at all the default table (0x383430) is 16 zero words [D ELF .data].
            let i = link.wrapping_sub(1) as usize;
            let skel = match pose.skel.get(i).or_else(|| pose.skel.first()) {
                Some(m) => *m,
                None => [[0.0; 4]; 4],
            };
            PartMatrix::Drawn(mul(&skel, &own))
        }
        0xfc | 0xfb => {
            let deg = (link as i16) as f32 * 0.003_906_25 * pose.frame;
            let axis = if kind == 0xfc { [0.0, 1.0, 0.0] } else { [0.0, 0.0, 1.0] };
            PartMatrix::Drawn(mul(&rot_axis_degrees(deg, axis), &own))
        }
        0xef => {
            let v = if (link as usize) < pose.dials.len() { pose.dials[link as usize] } else { pose.dials[0] };
            PartMatrix::Drawn(mul(&rot_axis_degrees(v, [0.0, 0.0, 1.0]), &own))
        }
        0xf3 => PartMatrix::Drawn(own),
        0x64 => PartMatrix::NotDrawn,
        k => PartMatrix::Unported(k),
    }
}

/// The LOD `FUN_001ad2e0` draws for `measure` (`FUN_001ad0b8`: `max(0, |camera − car| − 5) · K`,
/// K the f32 at +0x30 of the camera holder, not identified): `+0x0c & 8` → LOD 0; else the first
/// LOD whose distance is ≥ the measure; nothing when none is, or when that LOD has no model [M].
pub fn lod_index(p: &CarPart, measure: f32) -> Option<usize> {
    let k = if p.flags & 8 != 0 { 0 } else { p.lods.iter().position(|(d, _)| measure <= *d)? };
    p.lods.get(k).filter(|(_, m)| m.is_some()).map(|_| k)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::carmesh::CarPart;

    const ID3: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    fn part(index: usize, attach: Attach, origin: [f32; 3], flags: u8) -> CarPart {
        CarPart { index, flags, attach, axes: ID3, origin, set: 0, pass: 2, lods: vec![], strips: vec![] }
    }

    fn near(a: [f32; 3], b: [f32; 3]) -> bool { a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-5) }

    #[test]
    fn the_player_car_draws_list_1_2_in_an_interior_view_5_under_autodrive_and_others_always_1() {
        let st = |owner, interior_view, auto_drive| DrawState { owner, interior_view, auto_drive };
        let player = DrawOwner::Vehicle { driver_kind: 0 };
        assert_eq!(selected_list(&st(player, false, false)), Some(1));
        assert_eq!(selected_list(&st(player, true, false)), Some(2));
        assert_eq!(selected_list(&st(player, true, true)), Some(2), "interiorView is tested first [M 0x1883e4]");
        assert_eq!(selected_list(&st(player, false, true)), Some(5));
        assert_eq!(selected_list(&st(DrawOwner::Vehicle { driver_kind: 1 }, false, true)), Some(5));
        for k in [2, 3] {
            assert_eq!(selected_list(&st(DrawOwner::Vehicle { driver_kind: k }, true, true)), Some(1), "sltiu 2 [M 0x1883b8]");
        }
        assert_eq!(selected_list(&st(DrawOwner::Helicopter, true, true)), None);
        // The two-body case (supersnow's shape): list 1 = items 1-2 + 3, list 5 = 0 + 3. The old
        // view drew drawn(None) = all four.
        let m = CarMesh {
            parts: (0..4).map(|i| part(i, Attach::Fixed, [0.0; 3], 0)).collect(),
            draw_lists: vec![(1, vec![1, 2, 3]), (5, vec![0, 3])],
            ..Default::default()
        };
        let ids = |v: Vec<&CarPart>| v.iter().map(|p| p.index).collect::<Vec<_>>();
        assert_eq!(ids(m.list_for(&st(player, false, false))), vec![1, 2, 3]);
        assert_eq!(ids(m.list_for(&st(player, false, true))), vec![0, 3]);
        assert_eq!(ids(m.list_for(&st(player, true, false))), vec![0, 1, 2, 3], "no vd#2: every item");
        assert_eq!(ids(m.drawn(None)).len(), 4);
        assert!(player_car_hidden(true, false, false));
        assert!(player_car_hidden(false, true, true));
        assert!(!player_car_hidden(false, true, false));
    }

    #[test]
    fn mul_applies_its_first_matrix_first_and_the_rotations_follow_the_game_rows() {
        // R_y(0.25 turn): x → (0, 0, −1) (row 0 = (c, 0, −s)); R_x(0.25): y → (0, 0, 1) (row 1 = (0, c, s)).
        assert!(near(transform(&rot_y_turns(0.25), [1.0, 0.0, 0.0]), [0.0, 0.0, -1.0]));
        assert!(near(transform(&rot_x_turns(0.25), [0.0, 1.0, 0.0]), [0.0, 0.0, 1.0]));
        // FUN_002d11c8's degrees agree with the turns form; axis z: x → (0, 1, 0) at 90°.
        let a = rot_axis_degrees(90.0, [0.0, 2.0, 0.0]);
        let b = rot_y_turns(0.25);
        assert!((0..4).all(|i| (0..4).all(|k| (a[i][k] - b[i][k]).abs() < 1e-6)));
        assert!(near(transform(&rot_axis_degrees(90.0, [0.0, 0.0, 1.0]), [1.0, 0.0, 0.0]), [0.0, 1.0, 0.0]));
        assert_eq!(rot_axis_degrees(0.0, [0.0, 0.0, 1.0]), IDENTITY);
        // A spun wheel stays at its own origin: skel · own, not own · skel.
        let mut t = IDENTITY;
        t[3] = [1.0, 2.0, 3.0, 1.0];
        let spin = rot_x_turns(0.25);
        assert!(near(transform(&mul(&spin, &t), [0.0; 3]), [1.0, 2.0, 3.0]));
        assert!(!near(transform(&mul(&t, &spin), [0.0; 3]), [1.0, 2.0, 3.0]), "the other order orbits the car origin");
    }

    #[test]
    fn each_kind_gets_its_run_time_matrix() {
        let mut pose = CarPose { frame: 10.0, skel: vec![IDENTITY; 3], dials: [0.0; 12] };
        pose.skel[2] = rot_x_turns(0.25);
        let o = [0.5, 0.25, 1.0];
        let tip = |p: &CarPart| match part_matrix(p, &pose) {
            PartMatrix::Drawn(m) => transform(&m, [0.0, 1.0, 0.0]),
            other => panic!("{other:?}"),
        };
        // Fixed / 0xf3: the item's own matrix.
        assert!(near(tip(&part(0, Attach::Fixed, o, 0)), [0.5, 1.25, 1.0]));
        assert!(near(tip(&part(0, Attach::Other(0xf3, 0), o, 0)), [0.5, 1.25, 1.0]));
        // 0xfe Skel(2): R_x(0.25) first: y → z, then the own origin.
        assert!(near(tip(&part(0, Attach::Skel(2), o, 0)), [0.5, 0.25, 2.0]));
        // Past the count: matrix 0 (identity here).
        assert!(near(tip(&part(0, Attach::Skel(9), o, 0)), [0.5, 1.25, 1.0]));
        // 0xfc, v = 4864 (a rotor): 4864/256 · 10 = 190° about y.
        let r = match part_matrix(&part(0, Attach::Other(0xfc, 4864), [0.0; 3], 0), &pose) {
            PartMatrix::Drawn(m) => transform(&m, [1.0, 0.0, 0.0]),
            _ => unreachable!(),
        };
        let a = 190f32.to_radians();
        assert!(near(r, [a.cos(), 0.0, -a.sin()]), "{r:?}");
        // 0xfb, v = 1792 at frame 10: 70° about z.
        let r = match part_matrix(&part(0, Attach::Other(0xfb, 1792), [0.0; 3], 0), &pose) {
            PartMatrix::Drawn(m) => transform(&m, [1.0, 0.0, 0.0]),
            _ => unreachable!(),
        };
        let a = 70f32.to_radians();
        assert!(near(r, [a.cos(), a.sin(), 0.0]), "{r:?}");
        // 0xef: dial[v] degrees about z; v past 12 reads dial 0.
        pose.dials[1] = 90.0;
        pose.dials[0] = -90.0;
        let dial = |v| match part_matrix(&part(0, Attach::Other(0xef, v), [0.0; 3], 0), &pose) {
            PartMatrix::Drawn(m) => transform(&m, [1.0, 0.0, 0.0]),
            _ => unreachable!(),
        };
        assert!(near(dial(1), [0.0, 1.0, 0.0]));
        assert!(near(dial(40), [0.0, -1.0, 0.0]));
        assert_eq!(part_matrix(&part(0, Attach::Fixed, o, 1), &pose), PartMatrix::NotDrawn);
        assert_eq!(part_matrix(&part(0, Attach::Other(0x64, 0), o, 0), &pose), PartMatrix::NotDrawn);
        assert_eq!(part_matrix(&part(0, Attach::Other(0xee, 1024), o, 0), &pose), PartMatrix::Unported(0xee));
        assert_eq!(part_matrix(&part(0, Attach::Other(0xfc, 1), o, 0x10), &pose), PartMatrix::Unported(0xfc));
    }

    fn rig_mesh() -> CarMesh {
        // Skel: hood, lfhub, lfw, lrw, rfhub, rfw, rrw (1-based 1..7); Base words 5/9/14 = −0.5, 0.5, 1.2
        // → ref height 0.2.
        let skel = ["hood", "leftfronthub", "leftfrontwheel", "leftrearwheel", "rightfronthub", "rightfrontwheel", "rightrearwheel"];
        let mut base = [0.0; 16];
        base[5] = -0.5;
        base[9] = 0.5;
        base[14] = 1.2;
        CarMesh {
            skel: skel.iter().map(|s| s.to_string()).collect(),
            parts: vec![
                part(0, Attach::Fixed, [0.0; 3], 0),
                part(1, Attach::Skel(2), [-0.8, 0.35, 1.4], 0), // lfw
                part(2, Attach::Skel(5), [0.8, 0.35, 1.4], 0x80), // rfw
                part(3, Attach::Skel(3), [-0.8, 0.30, -1.3], 0), // lrw
                part(4, Attach::Skel(6), [0.8, 0.30, -1.3], 0x80), // rrw
            ],
            base: Some(base),
            ..Default::default()
        }
    }

    #[test]
    fn the_rig_scans_on_from_the_last_match_and_finds_each_wheel_offset() {
        let m = rig_mesh();
        assert!((m.wheel_ref_height().unwrap() - 0.2).abs() < 1e-6);
        let rig = WheelRig::new(&m, false);
        assert_eq!(rig.wheel, [3, 6, 4, 7]);
        assert_eq!(rig.hub, [2, 5, 0, 0]);
        for (k, want) in [0.15, 0.15, 0.10, 0.10].into_iter().enumerate() {
            assert!((rig.off[k] - want).abs() < 1e-6, "{k}: {}", rig.off[k]);
        }
        // Out of order (rear-left item before front-right): the scan for the front-right wheel
        // starts after the front-left match, finds it at item 3, then left-rear starts at item 4
        // and runs 5 items, 4 of them past the end: never found; right-rear starts past the end.
        // Both offsets stay 0, though both wheels are in the list.
        let mut m2 = m.clone();
        m2.parts.swap(2, 3);
        let r2 = WheelRig::new(&m2, false);
        assert!((r2.off[1] - 0.15).abs() < 1e-6);
        assert_eq!((r2.off[2], r2.off[3]), (0.0, 0.0));
        // No Skel record: wheels 1..4.
        let mut m3 = m.clone();
        m3.skel.clear();
        assert_eq!(WheelRig::new(&m3, false).wheel, [1, 2, 3, 4]);
        assert_eq!(WheelRig::new(&m3, false).skel_count, 4);
    }

    #[test]
    fn the_rig_steers_rate_limits_lifts_and_spins_the_wheels() {
        let m = rig_mesh();
        let rig = WheelRig::new(&m, false);
        let attrs = WheelAttrs { rest: 0.24, limit_draw: 0.125, wheel_radius: 0.36, is_sub: false };
        let mut pose = CarPose::rest(&m);
        let mut mem = WheelMemory::default();
        let input = WheelInput { spin: [0.25, 0.5], steer: 0.2, compression: [0.05, 0.3, 0.0, 0.1], slow_steer: false, fins: None };
        let s0 = rig.update(&attrs, &input, &mut mem, &mut pose.skel);
        // Clamped to 0.075 but moved at most 0.01 from 0.
        assert!((s0 - 0.01).abs() < 1e-7 && (mem.steer - 0.01).abs() < 1e-7);
        let s1 = rig.update(&attrs, &WheelInput { slow_steer: true, ..input }, &mut mem, &mut pose.skel);
        assert!((s1 - 0.012).abs() < 1e-6);
        // Hub translation y = R + min(x, LIMIT_DRAW) − off − REST.
        let ty = |k: usize, x: f32| 0.36 + x.min(0.125) - rig.off[k] - 0.24;
        let lfhub = pose.skel[1];
        assert!((lfhub[3][1] - ty(0, 0.05)).abs() < 1e-6);
        assert!((pose.skel[4][3][1] - ty(1, 0.3)).abs() < 1e-6, "right front capped at LIMIT_DRAW");
        // Left front wheel = R_x(front spin) · hub: its local y axis turns to (0, 0, 1) then steers.
        let w = pose.skel[2];
        let hub_only = rot_y_turns(s1);
        let y = transform(&w, [0.0, 1.0, 0.0]);
        let want = transform(&hub_only, [0.0, 0.0, 1.0]);
        assert!(near([y[0], y[1] - w[3][1], y[2]], want), "{y:?} vs {want:?}");
        // The right front hub turns by −s (its item negates x; the part steers the same way).
        let rx = transform(&pose.skel[4], [1.0, 0.0, 0.0]);
        let a = s1 * std::f32::consts::TAU;
        assert!(near([rx[0], rx[1] - pose.skel[4][3][1], rx[2]], [a.cos(), 0.0, a.sin()]));
        // The rear wheels: rear spin, no steer; R_x(0.5): y → −y.
        let rr = pose.skel[6];
        assert!(near(transform(&rr, [0.0, 1.0, 0.0]), [0.0, -1.0 + rr[3][1], 0.0]));
        // Placed: the front-left wheel item's origin stays at its own origin + the lift.
        let p = &m.parts[1];
        let PartMatrix::Drawn(mm) = part_matrix(p, &pose) else { panic!() };
        assert!(near(transform(&mm, [0.0; 3]), [-0.8, 0.35 + ty(0, 0.05), 1.4]));
    }

    #[test]
    fn sub_fins_add_their_terms() {
        let attrs = WheelAttrs { rest: 2.0, limit_draw: 2.0, wheel_radius: 1.0, is_sub: true };
        let rig = WheelRig { wheel: [0; 4], hub: [1, 2, 3, 4], off: [0.0; 4], skel_count: 4 };
        let mut skel = vec![IDENTITY; 4];
        let mut mem = WheelMemory::default();
        let inp = WheelInput { spin: [0.0; 2], steer: 0.005, compression: [0.0; 4], slow_steer: false, fins: Some((0.5, 0.05, false)) };
        let a0 = rig.update(&attrs, &inp, &mut mem, &mut skel);
        // pitch limited to 0.075, yaw 0.05: [s + 0.075·0.07, −s + 0.075·0.07, −s − 0.05·0.15, −s + 0.05·0.15].
        assert!((a0 - (0.005 + 0.075 * 0.07)).abs() < 1e-7);
        assert_eq!((mem.fin_pitch, mem.fin_yaw), (0.075, 0.05));
        let ang = |m: &Mat4| m[0][2].atan2(m[0][0]) / -std::f32::consts::TAU;
        for (k, want) in [0.005 + 0.00525, -0.005 + 0.00525, -0.005 - 0.0075, -0.005 + 0.0075].into_iter().enumerate() {
            assert!((ang(&skel[k]) - want).abs() < 1e-6, "{k}: {}", ang(&skel[k]));
            assert!((skel[k][3][1] - (1.0 - 2.0)).abs() < 1e-6);
        }
        // +0x29c set: the yaw input reads 0.
        let mut mem2 = WheelMemory { fin_yaw: 0.05, ..Default::default() };
        rig.update(&attrs, &WheelInput { fins: Some((0.0, 0.05, true)), ..inp }, &mut mem2, &mut skel);
        assert_eq!(mem2.fin_yaw, 0.0);
    }

    #[test]
    fn dials_spin_wrap_and_lod() {
        assert_eq!(dial_map(0.0, 1), 120.0);
        assert_eq!(dial_map(500.0, 1), -105.0);
        assert!((dial_map(3500.0, 2) - 16.0).abs() < 1e-4);
        let d = dials(0.02, Some((10.0, true, 7000.0)), &[0.0; 12]);
        assert!((d[0] + 15.0).abs() < 1e-5);
        assert!((d[1] - (120.0 - 225.0 * 54.0 / 330.0)).abs() < 1e-3);
        assert_eq!(d[2], -89.0);
        assert_eq!(dials(0.0, Some((10.0, false, 0.0)), &[0.0; 12])[1], 120.0, "backwards reads 0");
        let ai = dials(0.01, None, &d);
        assert_eq!((ai[1], ai[2]), (d[1], d[2]), "slots 1-2 only for the player");
        assert!((wrap_turns(1.25) - 0.25).abs() < 1e-6);
        assert!((wrap_turns(-0.25) - 0.75).abs() < 1e-6);
        assert_eq!(wrap_turns(1.0), 1.0);
        assert!((spin_step([0.9, 0.1], 0.2)[0] - 0.1).abs() < 1e-6);
        let mut p = part(0, Attach::Fixed, [0.0; 3], 0);
        p.lods = vec![(0.09, Some("a".into())), (1.0, Some("b".into())), (2.0, None)];
        assert_eq!(lod_index(&p, 0.05), Some(0));
        assert_eq!(lod_index(&p, 0.5), Some(1));
        assert_eq!(lod_index(&p, 1.5), None, "a LOD without a model draws nothing");
        assert_eq!(lod_index(&p, 3.0), None);
        p.flags = 8;
        assert_eq!(lod_index(&p, 3.0), Some(0));
    }
}
