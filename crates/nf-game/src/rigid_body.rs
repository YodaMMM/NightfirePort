//! The rigid-body prop init math — `RB_Init` 0x1d7ad8, `RB_CuboidMomentsOfInertia` 0x1d7dc8,
//! `RB_Build` 0x1d7ba0, `RB_SetState` 0x1d7da0 [M all four decompiles].
//!
//! The game's only caller of all four is `RB_Create` 0x1d6bf0 (`parsemap_create_dynamic_objects`
//! 0x1d04f0 case 0xD8, placement record +0x2c = mass param, +0x34 = the type halfword): the
//! kind-0xD8 pushable/shootable props that ship on levels 01/03/04/06/0d/41 [D
//! docs/systems/levels.md §RB rows; docs/atlas/world-objects.md:54-59 is the full RB_Create
//! writeup]. `RB_Create`'s flow [M 001d6bf0:48-79]:
//!   `RB_Init(ext, *(u16*)(rec+0x34))` → ext+0x74 = obj+0x90 → mass = rec+0x2c (0 → 3.0, else as
//!   int/float) → extents a/b/c = glist+0x30−0x20 / +0x34−0x24 / +0x38−0x28 →
//!   `RB_CuboidMomentsOfInertia(mass, a, b, c, out)` → `RB_Build(mass, ext, 0x2d8890 box |
//!   0x2d6200 sphere verts, out)` by `ext+0xa8 == 0` → `RB_SetState(ext)`.
//!
//! The port draws kind-0xD8 props as static instances at the placement transform
//! (`crates/nf-view/src/scene.rs`, the models-misc comment: "0xD8 RB_Create 0x1d6bf0:52 (rigid
//! bodies evolve obj+0x90 once disturbed; at rest = placement)") — the at-rest state IS the
//! post-RB_Init/RB_Build/RB_SetState state. The disturbed-body evolution (`RB_Update` 0x1d6e38,
//! `RB_EulerEvolve` 0x1d7810, `RB_ApplyImpulses` 0x1d6ff8, `RB_RestingContact` 0x1d7168) is the
//! tracked gap (docs/systems/levels.md §7.1, the level reports) — this module ports the four
//! init functions so the evolution port has its base.
//!
//! All offsets are the RIGIDBODY ext block (obj+0xE0, 0x160 bytes [M RB_Init memset]).
//!
//! Collide_CylinderIntersect 0x1ecc00 [M] runs from RB_CollisionHandler 0x1d7a80 → RB_RestingContactRot 0x1d73a0:111
//! (cylinder HITTEST at RIGIDBODY+0xB0) — reached on levels 07000006/0700000d (barrel placements, type halfword
//! 0xFFFF [D]); owned when the RB evolution lands (docs/systems/levels.md §7.1). Coll_ResetHitHeap 0x1edda0 [M]
//! (the per-level HITDATA/TriHeap reset, ResetMap_GameInit 0x1d5080:35) is absorbed: the port's collision allocates
//! directly (world.rs CollisionWorld, nf-formats/collision.rs).

/// The RIGIDBODY struct size in bytes [M 001d7ad8: memset(param_1, 0, 0x160)].
pub const RB_SIZE: usize = 0x160;
/// The box collider vertex table `RB_Create` passes for type 0 [M 001d6bf0:74].
pub const BOX_VERTS: u32 = 0x2d8890;
/// The sphere collider vertex table for every other type [M 001d6bf0:77].
pub const SPHERE_VERTS: u32 = 0x2d6200;

/// The RIGIDBODY fields the four init functions write (offsets per the decompiles).
#[derive(Clone, Debug, PartialEq)]
pub struct RigidBody {
    /// +0xa8 (i16): the collider type [M RB_Init].
    pub ty: i16,
    /// +0x64..+0x74: the orientation quaternion, identity (1,0,0,0) after Init [M RB_Init
    /// writes +0x64 = 1.0f, +0x68/+0x6c/+0x70 = 0].
    pub quat: [f32; 4],
    /// +0x60: the collider vertex table (BOX_VERTS / SPHERE_VERTS) [M RB_Build].
    pub vert_table: u32,
    /// +0x78 / +0x7c: mass and 1/mass [M RB_Build].
    pub mass: f32,
    pub inv_mass: f32,
    /// +0x80/+0x84/+0x88: principal moments; +0x8c/+0x90/+0x94 their reciprocals (note the
    /// game's store order: +0x8c = 1/m0, +0x94 = 1/m1, +0x90 = 1/m2 [M RB_Build]).
    pub moments: [f32; 3],
    pub inv_moments: [f32; 3],
    /// +0x98/+0xa0/+0xa4: the per-type scalar triple (damping/buoyancy constants; unnamed in
    /// the decompile) [M RB_Init].
    pub s98: f32,
    pub sa0: f32,
    pub sa4: f32,
}

impl RigidBody {
    /// An all-zero body (the memset).
    pub fn zeroed() -> Self {
        RigidBody {
            ty: 0,
            quat: [0.0; 4],
            vert_table: 0,
            mass: 0.0,
            inv_mass: 0.0,
            moments: [0.0; 3],
            inv_moments: [0.0; 3],
            s98: 0.0,
            sa0: 0.0,
            sa4: 0.0,
        }
    }
}

/// `RB_Init` 0x1d7ad8 [M]: zero 0x160 bytes, then +0xa8 = type, +0x64 = 1.0f, +0x68/6c/70 = 0,
/// and the per-type triple: type 0 → (+0x98, +0xa0, +0xa4) = (−0.5, 1.0, 31.0); type 3 →
/// (−0.75, 0.0, 250.0 — 0x437a0000, review 2026-10-21: the branch said 252); any other type → (−0.75, 0.0, 25.0). (The game's odd non-void return —
/// the memset pointer on type 0, literal 3 otherwise — is an artifact, not behaviour.)
pub fn rb_init(ty: i16) -> RigidBody {
    let mut rb = RigidBody::zeroed();
    rb.ty = ty;
    rb.quat = [1.0, 0.0, 0.0, 0.0];
    if ty == 0 {
        rb.s98 = -0.5;
        rb.sa0 = 1.0;
        rb.sa4 = 31.0;
    } else if ty == 3 {
        rb.s98 = -0.75;
        rb.sa0 = 0.0;
        rb.sa4 = 250.0; // 0x437a0000 [M 0x1d7ad8:22]
    } else {
        rb.s98 = -0.75;
        rb.sa0 = 0.0;
        rb.sa4 = 25.0;
    }
    rb
}

/// `RB_CuboidMomentsOfInertia` 0x1d7dc8 [M 001d7dc8]: the textbook cuboid inertia about the
/// centre — out[0] = m·(b²+c²)/12, out[1] = m·(a²+c²)/12, out[2] = m·(a²+b²)/12, with the
/// game's literal 0.083333336 (= 1/12). `RB_Create` passes (mass, a, b, c) = (rec+0x2c mass,
/// glist Δx, Δy, Δz) [M 001d6bf0:70-73].
pub fn rb_cuboid_moments_of_inertia(mass: f32, a: f32, b: f32, c: f32) -> [f32; 3] {
    const TWELFTH: f32 = 0.083_333_336;
    [
        mass * (b * b + c * c) * TWELFTH,
        mass * (a * a + c * c) * TWELFTH,
        mass * (a * a + b * b) * TWELFTH,
    ]
}

/// `RB_Build` 0x1d7ba0 [M 001d7ba0]: validate the moments (each ≥ 0 and each ≤ the sum of the
/// other two — the physical-inertia triangle check), then store mass (+0x78), 1/mass (+0x7c),
/// the moments (+0x80/84/88) and their reciprocals (+0x8c = 1/m0, +0x94 = 1/m1, +0x90 = 1/m2),
/// and the collider vertex table (+0x60). Returns false when the check fails (the game returns
/// 0 and `RB_Create` leaves the body unbuilt).
pub fn rb_build(rb: &mut RigidBody, mass: f32, vert_table: u32, moments: [f32; 3]) -> bool {
    let [m0, m1, m2] = moments;
    if !(m0 >= 0.0 && m1 >= 0.0 && m2 >= 0.0) { return false; }
    if !(m2 <= m0 + m1 && m0 <= m1 + m2 && m1 <= m0 + m2) { return false; }
    rb.moments = moments;
    rb.inv_moments = [1.0 / m0, 1.0 / m1, 1.0 / m2];
    rb.vert_table = vert_table;
    rb.mass = mass;
    rb.inv_mass = 1.0 / mass;
    true
}

/// `RB_SetState` 0x1d7da0 [M 001d7da0]: rebuild the orientation matrix from the body quaternion
/// — `Quat_QuaternionToMatrix(matrix, quat)` with the game's arguments (matrix = *(ext+0x74) =
/// obj+0x90, quat = ext+0x64). `RB_Create` sets ext+0x74 = obj+0x90 at :53 and the quat is
/// identity until the evolution writes it, so the at-rest matrix is the identity rotation.
pub fn rb_set_state(rb: &RigidBody) -> [[f32; 3]; 3] {
    crate::player::quat_to_matrix(&rb.quat)
}

#[cfg(test)]
mod tests {
    use super::*;

    // [M 001d7ad8] the per-type scalar triple + identity quat.
    #[test]
    fn rb_init_type_table() {
        let t0 = rb_init(0);
        assert_eq!((t0.s98, t0.sa0, t0.sa4), (-0.5, 1.0, 31.0));
        assert_eq!(t0.quat, [1.0, 0.0, 0.0, 0.0]);
        let t3 = rb_init(3);
        assert_eq!((t3.s98, t3.sa0, t3.sa4), (-0.75, 0.0, 250.0)); // 0x437a0000 = 250.0
        let tx = rb_init(7);
        assert_eq!((tx.s98, tx.sa0, tx.sa4), (-0.75, 0.0, 25.0));
        // Sister (can fail): the type-0 +0x98 is the outlier — −0.5 sits ABOVE the −0.75 band.
        assert!(t0.s98 > t3.s98, "type 0's +0x98 (−0.5) vs the −0.75 band [M 001d7ad8]");
    }

    // [M 001d7dc8] Ix = m(b²+c²)/12 etc., the game's 0.083333336 literal.
    #[test]
    fn cuboid_moments_formula() {
        let m = rb_cuboid_moments_of_inertia(3.0, 2.0, 4.0, 6.0);
        let tw = 0.083_333_336_f32;
        assert_eq!(m[0], 3.0 * (16.0 + 36.0) * tw);
        assert_eq!(m[1], 3.0 * (4.0 + 36.0) * tw);
        assert_eq!(m[2], 3.0 * (4.0 + 16.0) * tw);
        // Sister (can fail): the axes are permuted — with b > a, Ix (b²+c²) exceeds Iy (a²+c²).
        assert!(m[0] > m[1], "Ix (b²+c²) > Iy (a²+c²) when b > a [M 001d7dc8]");
    }

    // [M 001d7ba0] the triangle check: accept a physical triple, reject degenerate ones.
    #[test]
    fn build_validates_moments() {
        let mut rb = rb_init(0);
        assert!(rb_build(&mut rb, 3.0, BOX_VERTS, [2.0, 3.0, 4.0]));
        assert_eq!(rb.mass, 3.0);
        assert_eq!(rb.inv_mass, 1.0 / 3.0);
        assert_eq!(rb.moments, [2.0, 3.0, 4.0]);
        assert_eq!(rb.inv_moments[2], 1.0 / 4.0); // +0x90 = 1/m2
        assert_eq!(rb.vert_table, BOX_VERTS);
        // Sisters (can fail): each rejected class — negative, and the triangle violation.
        let mut bad = rb_init(0);
        assert!(!rb_build(&mut bad, 3.0, BOX_VERTS, [-1.0, 3.0, 4.0]));
        assert!(!rb_build(&mut bad, 3.0, BOX_VERTS, [10.0, 1.0, 1.0])); // m0 > m1+m2
        assert_eq!(bad.mass, 0.0, "a rejected build stores nothing [M 001d7ba0]");
    }

    // [M 001d7da0] the identity quat → identity rotation (the at-rest state).
    #[test]
    fn set_state_identity() {
        let rb = rb_init(0);
        let m = rb_set_state(&rb);
        assert_eq!(m[0][0], 1.0);
        assert_eq!(m[1][1], 1.0);
        assert_eq!(m[2][2], 1.0);
        // Sister (can fail): off-diagonal cells are 0 — a wrong convention would rotate.
        assert_eq!(m[0][1], 0.0);
        assert_eq!(m[2][0], 0.0);
    }
}
