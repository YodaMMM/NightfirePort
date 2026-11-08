//! The use key's object pick (`Player_Activate` 0x1a5170:100-108 → `Collide_SphereIntersect`
//! 0x1ec940), shared by every class arm so no arm keeps its own reach.
//!
//! The game builds ONE sphere — radius exactly 1.0 (asm 0x1a5278 `lui $at, 0x3f80`), centred at
//! `Player_GetHeadPos` + the viewer's direction row — and keeps an object only when it passes
//! both stages of the 0x101-mask pick [M]:
//!
//! 1. The candidate: `|centre − obj+0x80|² < (1.0 + obj+0x8C)²`, strict (`Collide_PickObj`
//!    0x1eae80's 0x100/0x101 tail; `Collide_Pick` 0x1eb320's linked-object arm). obj+0x80 is the
//!    object's WORLD SPHERE centre — the glist's local bounds centre posed through the object
//!    matrix (`Control_BuildWorldSph` 0x133070:33-44), radius glist+0x1c · |scale| (:48). It is
//!    not the matrix translation: a model whose origin sits at a corner (0700000b's floor grate
//!    #98) is measured from its middle.
//! 2. The refine: `Collide_Intersect` 0x1ebaa8 copies obj+0x90, inverts it, moves the pick centre
//!    into object space and, for mask 0x101, presets the node's distance to 1e8 (:186-189) before
//!    `Intersect_PointGeom` 0x1e92d8 tests the glist's collision triangles. A triangle registers
//!    when the signed plane distance lies in (0, 1.0] (front side only, `fVar25 = 0` because
//!    HITTEST+0x9a is 0 [M :116-121, :200-201]) and the clamped point-to-triangle distance is
//!    ≤ 1.0 [M :222-229]; with no triangle the node keeps 1e8 (:331), and 0x1ec940's finalize
//!    frees every 1e8 node before the class walk sees the list [M 0x1ec940 `if (fVar10 ==
//!    1e+08) LList_Add(free list)`]. No triangle, no activation.
//!
//! There is no facing cone in the pick itself; the only facing rules are per class: the lock
//! arm's `Mat_GetDir(lock+0x90) · (player+0xC0 − lock+0xC0) < 0` refusal [M 0x1a5170:158-166]
//! and `SP_Activate` 0x1dc338's switch-link dot (useobj.rs).
//!
//! An object whose collision shape the port does not hold (`None`) keeps the candidate stage
//! alone — a port-gap stand-in, not game behaviour: the game admits no glist-less object to the
//! 0x101 pick at all [M Collide_PickObj 0x1eae80:88-90].

use crate::world::{CollisionWorld, V3};

pub type Mat4 = [[f32; 4]; 4];

/// The pick sphere's radius [M asm 0x1a5278].
pub const RADIUS: f32 = 1.0;

/// The pick centre: the head plus the unit view direction [M 0x1a5170:100-108].
pub fn point(head: V3, look: V3) -> V3 { [head[0] + look[0], head[1] + look[1], head[2] + look[2]] }

/// Stage 1, the world-sphere candidate (strict `<` [M 0x1eae80]).
pub fn candidate(p: V3, centre: V3, radius: f32) -> bool {
    let d = [p[0] - centre[0], p[1] - centre[1], p[2] - centre[2]];
    let reach = RADIUS + radius;
    d[0] * d[0] + d[1] * d[1] + d[2] * d[2] < reach * reach
}

/// The general affine inverse of a row-vector matrix (`Mat_Inverse` — the game inverts the
/// object matrix as stored, scale included).
pub fn affine_inverse(m: &Mat4) -> Mat4 {
    let a = m;
    let det = a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
        - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
        + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0]);
    let inv_det = if det.abs() > 1e-12 { 1.0 / det } else { 0.0 };
    let mut o = [[0.0f32; 4]; 4];
    o[0][0] = (a[1][1] * a[2][2] - a[1][2] * a[2][1]) * inv_det;
    o[0][1] = (a[0][2] * a[2][1] - a[0][1] * a[2][2]) * inv_det;
    o[0][2] = (a[0][1] * a[1][2] - a[0][2] * a[1][1]) * inv_det;
    o[1][0] = (a[1][2] * a[2][0] - a[1][0] * a[2][2]) * inv_det;
    o[1][1] = (a[0][0] * a[2][2] - a[0][2] * a[2][0]) * inv_det;
    o[1][2] = (a[0][2] * a[1][0] - a[0][0] * a[1][2]) * inv_det;
    o[2][0] = (a[1][0] * a[2][1] - a[1][1] * a[2][0]) * inv_det;
    o[2][1] = (a[0][1] * a[2][0] - a[0][0] * a[2][1]) * inv_det;
    o[2][2] = (a[0][0] * a[1][1] - a[0][1] * a[1][0]) * inv_det;
    for c in 0..3 { o[3][c] = -(a[3][0] * o[0][c] + a[3][1] * o[1][c] + a[3][2] * o[2][c]); }
    o[3][3] = 1.0;
    o
}

/// Stage 2, the refine: the pick sphere moved into object space against the object-space
/// collision triangles [M 0x1ebaa8 → 0x1e92d8].
pub fn refine(p: V3, matrix: &Mat4, tris: &[[V3; 3]]) -> bool {
    let local = nf_formats::placement::transform(&affine_inverse(matrix), p);
    CollisionWorld::sphere_meets_tris(local, RADIUS, tris)
}

/// Both stages: does the use pick at `p` keep this object? `sphere` = (obj+0x80, obj+0x8C),
/// `matrix` = obj+0x90, `shape` = the glist's object-space collision triangles.
pub fn picks(p: V3, sphere: (V3, f32), matrix: &Mat4, shape: Option<&[[V3; 3]]>) -> bool {
    candidate(p, sphere.0, sphere.1) && shape.map_or(true, |t| refine(p, matrix, t))
}

/// `picks`, with the shape looked up by glist hash in the collision world (hash 0 = none).
pub fn picks_in(world: &CollisionWorld, p: V3, sphere: (V3, f32), matrix: &Mat4, hash: u32) -> bool {
    let shape = if hash != 0 { world.shape(hash) } else { None };
    picks(p, sphere, matrix, shape)
}

/// One placed object's pick data: (obj+0x80, obj+0x8C), obj+0x90 and the glist hash.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Target {
    pub sphere: (V3, f32),
    pub matrix: Mat4,
    pub hash: u32,
}

impl Target {
    pub fn picks(&self, world: &CollisionWorld, p: V3) -> bool { picks_in(world, p, self.sphere, &self.matrix, self.hash) }
    /// obj+0xC0, the matrix translation (the lock arm's facing input [M 0x1a5170:158-166]).
    pub fn origin(&self) -> V3 { [self.matrix[3][0], self.matrix[3][1], self.matrix[3][2]] }
}

/// A placement-created object's pick data. The matrix is the created object's obj+0x90
/// (`control_create_object` → `View_RotTransMatrix` for the dynamic creates [M 0x1344e0]);
/// the sphere is `Control_BuildWorldSph` 0x133070's: the glist's local centre through it, the
/// local radius × |scale| (:33-48); no model → obj+0x80 = obj+0xC0, radius 0.1 (:23-31). The
/// hash keys the collision world's shape table (the synthetic map/index key for objects
/// without a hashcode, as `CollisionWorld::new` stores them).
pub fn placed(level: &nf_formats::level::Level, m: usize, p: &nf_formats::placement::Placement) -> Target {
    let matrix = nf_formats::placement::view_rot_trans_matrix(p.rotation, p.translation);
    let obj = if p.by_hash() { level.by_hash.get(&p.hashcode).map(|&(mi, oi)| (mi, oi, &level.maps[mi].objects[oi])) }
        else { level.maps[m].objects.get(p.index as usize).map(|o| (m, p.index as usize, o)) };
    match obj {
        Some((mi, oi, o)) => {
            let s: f32 = p.scale().iter().map(|v| v.abs()).product();
            let hash = if o.hashcode == u32::MAX { 0x8000_0000 | ((mi as u32) << 16) | oi as u32 } else { o.hashcode };
            Target { sphere: (nf_formats::placement::transform(&matrix, o.centre), o.radius * s), matrix, hash }
        }
        None => Target { sphere: (p.translation, 0.1), matrix, hash: 0 },
    }
}

/// The pick data of the placed use classes that carry no matrix of their own, parallel to
/// their lists (`Drones::locks`, `monitors.list`, `props.gunimps`).
#[derive(Clone, Debug, Default)]
pub struct Placed {
    pub locks: Vec<Target>,
    pub monitors: Vec<Target>,
    pub gunimps: Vec<Target>,
}

impl Placed {
    /// Resolve each list entry's placement (indices into the main map's placements).
    pub fn build(level: &nf_formats::level::Level, m: usize, locks: &[usize], monitors: &[usize], gunimps: &[usize]) -> Placed {
        let ps = &level.maps[m].placements;
        let one = |i: &usize| ps.get(*i).map(|p| placed(level, m, p)).unwrap_or_default();
        Placed {
            locks: locks.iter().map(one).collect(),
            monitors: monitors.iter().map(one).collect(),
            gunimps: gunimps.iter().map(one).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: Mat4 = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];

    /// A 2×2 floor plate with its origin at the corner (0,0,0), facing up (+y).
    fn plate() -> Vec<[V3; 3]> {
        vec![[[0.0, 0.0, 0.0], [0.0, 0.0, 2.0], [2.0, 0.0, 2.0]], [[0.0, 0.0, 0.0], [2.0, 0.0, 2.0], [2.0, 0.0, 0.0]]]
    }

    #[test]
    fn the_candidate_is_measured_from_the_posed_sphere_centre_not_the_origin() {
        // The plate's sphere: local centre (1, 0, 1), radius 1.42; posed at (10, 0, 0).
        let mut m = ID;
        m[3] = [10.0, 0.0, 0.0, 1.0];
        let c = nf_formats::placement::transform(&m, [1.0, 0.0, 1.0]);
        // A use point 0.5 above the plate's far corner (12, 0.5, 2): 2.88 from the origin
        // (outside 1 + 1.42), 1.5 from the centre (inside).
        let p = [11.9, 0.5, 1.9];
        assert!(picks(p, (c, 1.42), &m, Some(&plate())));
        assert!(!candidate(p, [10.0, 0.0, 0.0], 1.42), "sister: the origin-measured test refuses it");
    }

    #[test]
    fn the_refine_keeps_only_a_point_in_front_of_a_face_within_the_radius() {
        let t = plate();
        assert!(refine([1.0, 0.6, 1.0], &ID, &t), "0.6 above the face");
        assert!(!refine([1.0, 1.2, 1.0], &ID, &t), "1.2 above: past the radius");
        assert!(!refine([1.0, -0.4, 1.0], &ID, &t), "behind the face: the front side only");
        assert!(!refine([1.0, 0.0, 1.0], &ID, &t), "in the plane: the distance must be > 0");
        // The scale-aware inverse: a plate scaled ×2 about its origin, posed at (0, 5, 0).
        let mut s = ID;
        s[0][0] = 2.0; s[1][1] = 2.0; s[2][2] = 2.0; s[3] = [0.0, 5.0, 0.0, 1.0];
        assert!(refine([3.0, 5.8, 3.0], &s, &t), "local (1.5, 0.4, 1.5)");
        assert!(!refine([3.0, 7.4, 3.0], &s, &t), "local y 1.2");
    }
}
