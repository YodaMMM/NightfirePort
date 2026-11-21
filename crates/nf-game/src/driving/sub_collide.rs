//! The world collision of a driving vehicle's hull (`FUN_001fc2f0`, pass B of the physics step
//! `FUN_001ff3d8`) for the SUB_PHYSICS bodies (the sub and the ultralights), with its response
//! (`FUN_001fb318`) and damage hook (`FUN_001fc198`). docs/systems/drive-physics.md §10 is the
//! write-up.
//!
//! Evidence [M] = DRIVING.ELF (decompile under `work/decomp/DRIVING.ELF/`, the call sites read in
//! the raw instructions), [D] = disc data, [S] = the port's own choice, stated where used.
//!
//! What the game does, in short [M 0x1fc2f0]:
//! - The pass runs for a body whose `inst+0x7c` bit 0 is set: the body constructor sets it for
//!   kinds 1 and 2 (the player is kind 1) [M FUN_001f6e98 :249-254]. `inst+0x7c` is otherwise the
//!   body's radius, `|half extents|` [M FUN_001f6e98 :176-188]. `NO_WORLD_COLLISIONS` is not read
//!   here: its one reader in the physics is pass C (`FUN_001f9500` :85), so the ultralights
//!   (which set it) still run this pass.
//! - Eleven points for a vehicle (kind < 4) [M :152-218]: the eight hull corners
//!   (class+0x280+sel·0x80, written by the flush `FUN_001f8cf0` as the body rows times
//!   `(±(hx+0.18), ±hy, ±hz)`), the nose point `pos + row2·(hz+0.05)`, and the two side points
//!   `pos ± row0·hx`, with `(hx, hy, hz)` the half extents of the model header box.
//! - Each point is a segment from the body's position to `point + vel·dt` (the velocity re-read
//!   for every point, so an earlier point's response moves the later segments) [M :219-232].
//! - The query (`FUN_002127f0` → `FUN_002166f8`) is run with the class mask 0x20 (triangles whose
//!   class byte has 0x20 are skipped) [M :251 sets `DAT_0033a540+0x2c = 0x20`; FUN_00213d68's
//!   mask test]. A SUB_PHYSICS body's gather flags are 0xc (a land car's 4), and bit 8 turns on
//!   the full triangle test [M :95-100, FUN_00211ed0, FUN_002127f0's `+0x5c >> 3 & 1`].
//! - A hit on surface byte 0x0e sends mission event 0x15 and gets no response [M :287-289].
//! - The response gets `depth = |end − hit|` (end = the extrapolated point), weight 1.0, or 0.25
//!   and a quarter of the depth for the two side points [M :348-356].

use super::sub_physics::{Body, Rows};
use super::physics::Tuning;

type V3 = [f32; 3];

fn dot(a: V3, b: V3) -> f32 { a[0] * b[0] + a[1] * b[1] + a[2] * b[2] }
fn cross(a: V3, b: V3) -> V3 { [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]] }
fn add(a: V3, b: V3) -> V3 { [a[0] + b[0], a[1] + b[1], a[2] + b[2]] }
fn sub(a: V3, b: V3) -> V3 { [a[0] - b[0], a[1] - b[1], a[2] - b[2]] }
fn scale(a: V3, s: f32) -> V3 { [a[0] * s, a[1] * s, a[2] * s] }
fn len(a: V3) -> f32 { dot(a, a).sqrt() }
fn to_world(m: &Rows, v: V3) -> V3 { add(add(scale(m[0], v[0]), scale(m[1], v[1])), scale(m[2], v[2])) }
fn to_local(m: &Rows, v: V3) -> V3 { [dot(m[0], v), dot(m[1], v), dot(m[2], v)] }
fn mat_vec(w: &Rows, v: V3) -> V3 { add(add(scale(w[0], v[0]), scale(w[1], v[1])), scale(w[2], v[2])) }

// ---------------------------------------------------------------------------
// The Physics:Rigid BUILDING_* globals the response reads
// ---------------------------------------------------------------------------

/// The building-collision tuning [M FUN_001f6600 registers; .data 0x339134-0x339158, equal to
/// the shipped `Physics\Rigid\default.tun` lines [D]].
#[derive(Clone, Debug, PartialEq)]
pub struct Building {
    /// `BUILDING_MICRO_THRESHOLD` 0x339134 = 25: the cap on |v·n| in the restitution term.
    pub micro_threshold: f32,
    /// `BUILDING_FRICTION_FACTOR` 0x33913c = 0.0045: the contact-point velocity's share taken off.
    pub friction_factor: f32,
    /// `BUILDING_COEFF_RESTITUTION` 0x339138 = 0.01.
    pub coeff_restitution: f32,
    /// `BUILDING_FRICTION_LIMIT` 0x339140 = 0.125: the per-axis cap of the friction term.
    pub friction_limit: f32,
    /// `BUILDING_FORCE_LIMIT` 0x339148 = 10: the cap on the normal impulse (as a velocity).
    pub force_limit: f32,
    /// `PF_SCALE` 0x339158 = 10: the depth's weight in the impulse.
    pub pf_scale: f32,
}

impl Building {
    pub fn game_defaults() -> Building {
        Building {
            micro_threshold: 25.0,
            friction_factor: 0.0045,
            coeff_restitution: 0.01,
            friction_limit: 0.125,
            force_limit: 10.0,
            pf_scale: 10.0,
        }
    }

    /// The tun override pass (full registered names) [M FUN_001f6600 :31-36].
    pub fn from_tun(t: &Tuning) -> Building {
        let mut b = Building::game_defaults();
        if let Some(v) = t.get("BUILDING_MICRO_THRESHOLD") { b.micro_threshold = v; }
        if let Some(v) = t.get("BUILDING_FRICTION_FACTOR") { b.friction_factor = v; }
        if let Some(v) = t.get("BUILDING_COEFF_RESTITUTION") { b.coeff_restitution = v; }
        if let Some(v) = t.get("BUILDING_FRICTION_LIMIT") { b.friction_limit = v; }
        if let Some(v) = t.get("BUILDING_FORCE_LIMIT") { b.force_limit = v; }
        if let Some(v) = t.get("PF_SCALE") { b.pf_scale = v; }
        b
    }

    /// `DAT_00339150` = COEFF_RESTITUTION / MICRO_THRESHOLD (0.0004 with the shipped values): the
    /// slope of the restitution term [M 0x1f6a68-0x1f6ac4 and FUN_001fd338 :9 compute it; the
    /// .data initial is 0.0]. Which of the two writers runs before the first collision is not
    /// traced [S: the port takes the computed value].
    pub fn restitution_slope(&self) -> f32 { self.coeff_restitution / self.micro_threshold }
}

// ---------------------------------------------------------------------------
// The hull points
// ---------------------------------------------------------------------------

/// The extra x half-width the flush adds to the hull corners of a vehicle (kind < 4)
/// [M FUN_001f8cf0 :88-90, `fVar12 + 0.18`].
pub const HULL_X_PAD: f32 = 0.18;
/// The nose point's reach past the box [M FUN_001fc2f0 :156, `+0x4c8 + 0.05`].
pub const NOSE_REACH: f32 = 0.05;

/// The eleven points of a vehicle, as offsets from the body position in world space
/// [M FUN_001f8cf0 :82-200 for 0-7 (corner order (−,−,−) (+,−,−) (+,−,+) (−,−,+) (−,+,+)
/// (+,+,+) (+,+,−) (−,+,−) in (x, y, z)), FUN_001fc2f0 :152-185 for 8-10].
pub fn hull_offsets(rows: &Rows, half: V3) -> [V3; 11] {
    let x = half[0] + HULL_X_PAD;
    let (y, z) = (half[1], half[2]);
    let c = [
        [-x, -y, -z],
        [x, -y, -z],
        [x, -y, z],
        [-x, -y, z],
        [-x, y, z],
        [x, y, z],
        [x, y, -z],
        [-x, y, -z],
    ];
    let mut out = [[0.0f32; 3]; 11];
    for (i, p) in c.iter().enumerate() {
        out[i] = to_world(rows, *p);
    }
    out[8] = scale(rows[2], half[2] + NOSE_REACH);
    out[9] = scale(rows[0], half[0]);
    out[10] = scale(rows[0], -half[0]);
    out
}

/// The body radius stored at inst+0x7c: `|half extents|` [M FUN_001f6e98 :176-188].
pub fn body_radius(half: V3) -> f32 { len(half) }

// ---------------------------------------------------------------------------
// The world triangles and the segment query
// ---------------------------------------------------------------------------

/// The class mask the pass sets around its queries: triangles whose class byte has a bit of it
/// are skipped [M FUN_001fc2f0 :251 `DAT_0033a540+0x2c = 0x20`; FUN_00213d68's
/// `(mask & tri+0x2d) != 0` rejection].
pub const QUERY_CLASS_MASK: u8 = 0x20;
/// How far behind the segment start a crossing still counts as the nearest one [M FUN_00213d68:
/// `-1.0 < d`]. Such a crossing wins over any ahead and then fails the `0 <= t` test of
/// `FUN_00231af0`, so a surface up to 1 m behind the start hides one ahead.
pub const BEHIND_SLACK: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldTri {
    pub v: [V3; 3],
    pub surface: u8,
    pub class: u8,
}

/// One answer of the segment query.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldHit {
    /// Where the segment crosses the triangle's plane [M FUN_00231af0].
    pub point: V3,
    /// The face normal, made to point up, its y capped at 0.9999, then turned toward the
    /// segment start [M FUN_002166f8 0x216894-0x2169b8, 0x216a38-0x216ab4].
    pub normal: V3,
    /// The fraction of the segment.
    pub t: f32,
    pub surface: u8,
    pub class: u8,
    /// The triangle's index.
    pub tri: usize,
}

/// The track's collision triangles with an xz grid for the query (the game gathers the triangles
/// near the body into a per-vehicle cache, `FUN_00211e18` / `FUN_00217588`; the grid stands in
/// for that cache [S]).
pub struct WorldMesh {
    pub tris: Vec<WorldTri>,
    cell: f32,
    min: [f32; 2],
    dim: [i32; 2],
    cells: Vec<Vec<u32>>,
    stamp: std::cell::RefCell<(u32, Vec<u32>)>,
}

impl WorldMesh {
    /// `tris`: world-space corners and the triangle word's low half (surface byte, class byte),
    /// as `nf_formats::crp_mesh::CollisionTri` holds them.
    pub fn new(tris: impl IntoIterator<Item = ([V3; 3], u16)>) -> WorldMesh {
        let tris: Vec<WorldTri> = tris.into_iter().map(|(v, f)| WorldTri { v, surface: f as u8, class: (f >> 8) as u8 }).collect();
        let cell = 8.0f32;
        let mut lo = [f32::MAX; 2];
        let mut hi = [f32::MIN; 2];
        for t in &tris {
            for p in &t.v {
                lo[0] = lo[0].min(p[0]);
                lo[1] = lo[1].min(p[2]);
                hi[0] = hi[0].max(p[0]);
                hi[1] = hi[1].max(p[2]);
            }
        }
        if tris.is_empty() {
            lo = [0.0; 2];
            hi = [0.0; 2];
        }
        let dim = [((hi[0] - lo[0]) / cell) as i32 + 1, ((hi[1] - lo[1]) / cell) as i32 + 1];
        let mut cells = vec![Vec::new(); (dim[0] * dim[1]) as usize];
        for (i, t) in tris.iter().enumerate() {
            let (mut a, mut b) = ([f32::MAX; 2], [f32::MIN; 2]);
            for p in &t.v {
                a[0] = a[0].min(p[0]);
                a[1] = a[1].min(p[2]);
                b[0] = b[0].max(p[0]);
                b[1] = b[1].max(p[2]);
            }
            let (x0, z0) = (((a[0] - lo[0]) / cell) as i32, ((a[1] - lo[1]) / cell) as i32);
            let (x1, z1) = (((b[0] - lo[0]) / cell) as i32, ((b[1] - lo[1]) / cell) as i32);
            for z in z0.max(0)..=z1.min(dim[1] - 1) {
                for x in x0.max(0)..=x1.min(dim[0] - 1) {
                    cells[(z * dim[0] + x) as usize].push(i as u32);
                }
            }
        }
        let n = tris.len();
        WorldMesh { tris, cell, min: lo, dim, cells, stamp: std::cell::RefCell::new((0, vec![0; n])) }
    }

    fn for_each_near(&self, lo: [f32; 2], hi: [f32; 2], mut f: impl FnMut(usize)) {
        let mut st = self.stamp.borrow_mut();
        st.0 = st.0.wrapping_add(1);
        if st.0 == 0 {
            st.1.iter_mut().for_each(|s| *s = 0);
            st.0 = 1;
        }
        let tag = st.0;
        let x0 = (((lo[0] - self.min[0]) / self.cell).floor() as i32).max(0);
        let z0 = (((lo[1] - self.min[1]) / self.cell).floor() as i32).max(0);
        let x1 = (((hi[0] - self.min[0]) / self.cell).floor() as i32).min(self.dim[0] - 1);
        let z1 = (((hi[1] - self.min[1]) / self.cell).floor() as i32).min(self.dim[1] - 1);
        for z in z0..=z1 {
            for x in x0..=x1 {
                for &i in &self.cells[(z * self.dim[0] + x) as usize] {
                    let i = i as usize;
                    if st.1[i] != tag {
                        st.1[i] = tag;
                        f(i);
                    }
                }
            }
        }
    }

    /// The segment query `FUN_002166f8`'s triangle path [M]: among the triangles the line through
    /// `start`→`end` passes through (both windings; `FUN_00213d68`'s point-in-triangle test in the
    /// segment's own frame) and whose class byte misses `mask`, the crossing nearest the start
    /// with `d > −1` (d = distance along the segment from the start) is taken; it is a hit only
    /// when it lies on the segment, `0 <= t <= 1` [M FUN_00231af0].
    pub fn segment(&self, start: V3, end: V3, mask: u8) -> Option<WorldHit> {
        let dv = sub(end, start);
        let l = len(dv);
        if !(l > 1e-4) {
            // FUN_00231d68: a segment shorter than 0.0001 on every axis builds no frame.
            return None;
        }
        let (t, i) = self.nearest_crossing(start, end, mask, -BEHIND_SLACK / l)?;
        if !(0.0..=1.0).contains(&t) {
            return None;
        }
        let tr = &self.tris[i];
        let point = add(start, scale(dv, t));
        let [v0, v1, v2] = tr.v;
        let c = cross(sub(v1, v0), sub(v0, v2));
        let mut n = if c == [0.0, 0.0, 0.0] { [0.0, 1.0, 0.0] } else { scale(c, 1.0 / len(c)) };
        if n[1] < 0.0 {
            n = scale(n, -1.0);
        }
        if 0.9999 <= n[1] {
            n[1] = f32::from_bits(0x3f7f_f972);
        }
        if dot(n, sub(start, point)) < 0.0 {
            n = scale(n, -1.0);
        }
        Some(WorldHit { point, normal: n, t, surface: tr.surface, class: tr.class, tri: i })
    }

    /// A plain test (not the game's): the first triangle (class missing `mask`) the segment
    /// `a`→`b` crosses, as (t, index). Tests use it to check that a body never passes through.
    pub fn crossing(&self, a: V3, b: V3, mask: u8) -> Option<(f32, usize)> {
        let (t, i) = self.nearest_crossing(a, b, mask, 0.0)?;
        (t <= 1.0).then_some((t, i))
    }

    /// The nearest line crossing with `t > min_t` (t in segment lengths), both windings.
    fn nearest_crossing(&self, start: V3, end: V3, mask: u8, min_t: f32) -> Option<(f32, usize)> {
        let dv = sub(end, start);
        let s0 = add(start, scale(dv, min_t.min(0.0)));
        let lo = [s0[0].min(end[0]) - 0.01, s0[2].min(end[2]) - 0.01];
        let hi = [s0[0].max(end[0]) + 0.01, s0[2].max(end[2]) + 0.01];
        let mut best: Option<(f32, usize)> = None;
        self.for_each_near(lo, hi, |i| {
            let tr = &self.tris[i];
            if tr.class & mask != 0 {
                return;
            }
            let [a, b, c] = tr.v;
            let e1 = sub(b, a);
            let e2 = sub(c, a);
            let p = cross(dv, e2);
            let det = dot(e1, p);
            if det.abs() < 1e-12 {
                return;
            }
            let inv = 1.0 / det;
            let tv = sub(start, a);
            let u = dot(tv, p) * inv;
            if !(0.0..=1.0).contains(&u) {
                return;
            }
            let q = cross(tv, e1);
            let v = dot(dv, q) * inv;
            if v < 0.0 || u + v > 1.0 {
                return;
            }
            let t = dot(e2, q) * inv;
            let ok = if min_t == 0.0 { t >= 0.0 } else { t > min_t };
            if ok && best.map_or(true, |(bt, _)| t < bt) {
                best = Some((t, i));
            }
        });
        best
    }
}

// ---------------------------------------------------------------------------
// The response FUN_001fb318 (world contact, a SUB_PHYSICS vehicle)
// ---------------------------------------------------------------------------

/// The roll share of a SUB_PHYSICS body's contact torque: the local z lane ×0.15
/// [M FUN_001fb318 0x1fba2c-0x1fba50: FUN_001f8508, `fStack_148 * 0.15`, FUN_001f84d0].
pub const SUB_ROLL_TORQUE: f32 = 0.15;

/// `FUN_001fb318(depth, weight, inst, info, normal, hit, 1, surface)` for a vehicle (kind < 4)
/// whose SUB_PHYSICS attr is 1 [M]. Changes the body's velocity, angular momentum and angular
/// velocity at once (the game writes +0x40/+0x50 and rebuilds +0x20/+0x30) and returns the
/// contact strength written to info+0x44, `min(|j|·0.5 / FORCE_LIMIT, 1)` (0 when the point
/// separates).
///
/// With r = hit − pos, u = ω×r + v, vn = (u·n)·weight:
/// - vn > 0: no response, strength 0 [M 0x1fb3e8 branch];
/// - denominator `((W(r×n))×r)·n + h`, h = `sqrt(max(r.x²+r.z² − |r.x n.x + r.z n.z|, 0)) / hx`;
/// - `j = −(slope·min(|vn|, MICRO) + 1)·(vn − depth·PF_SCALE) / den` (0 when den is 0);
/// - Δv = clamp(j, ±FORCE_LIMIT)·n − clamp_each(u·FRICTION_FACTOR, ±FRICTION_LIMIT);
/// - p += m·Δv; τ = r × m·Δv with its local z lane ×0.15; L += τ; v = p/m; ω = W·L.
pub fn respond(body: &mut Body, half_x: f32, depth: f32, weight: f32, n: V3, hit: V3, b: &Building) -> f32 {
    let r = sub(hit, body.pos);
    let u = add(cross(body.ang_vel, r), body.vel);
    let vn = dot(u, n) * weight;
    if 0.0 < vn {
        return 0.0;
    }
    let w = body.inv_inertia_world;
    let a = mat_vec(&w, cross(r, n));
    let bb = cross(a, r);
    let mut h = r[0] * r[0] + r[2] * r[2] - (r[0] * n[0] + r[2] * n[2]).abs();
    h = h.max(0.0);
    if 0.0 < h {
        h = h.sqrt() / half_x;
    }
    let den = dot(bb, n) + h;
    let j = if den == 0.0 {
        0.0
    } else {
        let e = b.restitution_slope() * vn.abs().min(b.micro_threshold);
        (-(e + 1.0) * (vn - depth * b.pf_scale)) / den
    };
    let jc = j.min(b.force_limit).max(-b.force_limit);
    let clamp = |x: f32| x.min(b.friction_limit).max(-b.friction_limit);
    let f = [clamp(u[0] * b.friction_factor), clamp(u[1] * b.friction_factor), clamp(u[2] * b.friction_factor)];
    let dv = sub(scale(n, jc), f);
    let dp = scale(dv, body.mass);
    body.vel = add(body.vel, dv);
    let mut tq = to_local(&body.rows, cross(r, dp));
    tq[2] *= SUB_ROLL_TORQUE;
    let tq = to_world(&body.rows, tq);
    body.ang_mom = add(body.ang_mom, tq);
    body.ang_vel = mat_vec(&w, body.ang_mom);
    ((j.abs() * 0.5) / b.force_limit).min(1.0)
}

// ---------------------------------------------------------------------------
// The damage hook FUN_001fc198
// ---------------------------------------------------------------------------

/// A vehicle's contact must be stronger than this to hurt (0.05 for other bodies)
/// [M FUN_001fc198 0x1fc19c-0x1fc1d4].
pub const DAMAGE_MIN_STRENGTH: f32 = 0.2;
/// `DAT_0033916c` × `DAT_00339170` = 5 × 6 [M .data; no writer in DRIVING.ELF].
pub const DAMAGE_SCALE: f32 = 5.0 * 6.0;
/// `DAT_0033915c` = 0.35, the second argument of the damage call [M .data].
pub const DAMAGE_ARG: f32 = 0.35;

/// The hit points `FUN_001fc198` hands the vehicle's damage call (vtbl+0x1c = `FUN_00189598`)
/// for a contact of `strength`: `2·strength·5·6` when above 0.2, else none [M]. The kind-1 gate
/// (vtbl+0x60 = `FUN_0018ad28`: a path-driven player takes none) is the caller's.
pub fn world_damage(strength: f32) -> Option<f32> {
    if strength <= DAMAGE_MIN_STRENGTH {
        return None;
    }
    Some((strength + strength) * DAMAGE_SCALE)
}

// ---------------------------------------------------------------------------
// The pass FUN_001fc2f0 for the player's SUB_PHYSICS vehicle
// ---------------------------------------------------------------------------

/// The surface byte whose hit sends mission event 0x15 instead of a response [M :287-289].
pub const SURFACE_EVENT: u8 = 0x0e;

/// One point's contact, the fields the COLLISION_INFO message (`FUN_00149d30(0x58, info)`)
/// carries that the port keeps.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Contact {
    /// 0-7 the hull corners, 8 the nose point, 9/10 the side points.
    pub point_index: usize,
    pub hit: WorldHit,
    /// info+0x44 (0 when the point separates).
    pub strength: f32,
    /// The damage handed to the vehicle (points 0-8 only [M :437 `iVar5 < 9`]).
    pub damage: Option<f32>,
}

/// What one pass produced.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CollideOut {
    pub contacts: Vec<Contact>,
    /// Hits on surface 0x0e (mission event 0x15), by point index.
    pub event_hits: Vec<usize>,
    /// The +0x29e flag (vtbl+0x1e4 = `FUN_001931b8`): cleared at the start of the pass for a
    /// vehicle, set when a front corner (2-5) or the nose point of the player hits a surface whose
    /// normal faces against the nose while |v| <= 7.5 [M :199-203, :290-321]. The SUB_PHYSICS
    /// pass reads it next tick (the angular stick impulse).
    pub nose_stuck: bool,
}

/// `FUN_001fc2f0` for the player (kind 1, so no frame skip and the +0x29e check) driving a
/// SUB_PHYSICS vehicle: the eleven points, the query, the response, the damage [M]. `half` =
/// the model header box half extents, `dt` = the frame step (`DAT_00339f50`).
pub fn collide_world(body: &mut Body, half: V3, mesh: &WorldMesh, b: &Building, dt: f32) -> CollideOut {
    let mut out = CollideOut::default();
    let start = body.pos;
    let offs = hull_offsets(&body.rows, half);
    let mut stop_nose_check = false;
    for (i, off) in offs.iter().enumerate() {
        let end = add(add(start, *off), scale(body.vel, dt));
        let Some(hit) = mesh.segment(start, end, QUERY_CLASS_MASK) else { continue };
        if hit.surface == SURFACE_EVENT {
            out.event_hits.push(i);
            continue;
        }
        if !stop_nose_check && ((2..=5).contains(&i) || i == 8) {
            if 7.5 < len(body.vel) {
                stop_nose_check = true;
            } else if dot(body.rows[2], hit.normal) <= 0.0 {
                out.nose_stuck = true;
            }
        }
        // SUB_PHYSICS keeps the penetration's y lane [M :333-343].
        let mut depth = len(sub(end, hit.point));
        let weight = if i < 9 { 1.0 } else { depth *= 0.25; 0.25 };
        let strength = respond(body, half[0], depth, weight, hit.normal, hit.point, b);
        let damage = if i < 9 { world_damage(strength) } else { None };
        out.contacts.push(Contact { point_index: i, hit, strength, damage });
    }
    out
}
