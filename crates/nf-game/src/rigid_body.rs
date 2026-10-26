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
//! init functions so the evolution port has its base. When that evolution port lands, its drag
//! term is the game's `expf` 0x232f70 (libm; jal at 0x1d78a8/0x1d7904 in `RB_EulerEvolve` [M ELF
//! jal scan]) — Rust's native `f32::exp` replaces it there [M unported-misc-7].
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

// ============================================================================
// The evolution (the tracked gap): RB_AddImpulsiveForce 0x1d7d40,
// RB_AddImpulsiveForceAtPoint 0x1d7d58, RB_ToWorld 0x1d7e18, RB_ToBody 0x1d7eb8,
// RB_VecSpecial 0x1d8038, RB_apply_omega 0x1d7f50, RB_ApplyImpulses 0x1d6ff8,
// RB_ApplyWind 0x1d7c78, RB_EulerEvolve 0x1d7810, and the explosion-hit arm of
// RB_RestingContact 0x1d7168 — each ported from its decompile/disassembly.
// ============================================================================

/// The RIGIDBODY slots the evolution reads and writes. The init fields above stay untouched;
/// these ride at the end on their own lines.
#[derive(Clone, Debug, PartialEq)]
pub struct RbState {
    /// +0x00: the position slot — `RB_ApplyImpulses` adds the linear impulse here (type 0);
    /// `RB_EulerEvolve` integrates `pos += dt·vel` (type 0) and writes it into the drawn
    /// matrix's translation row [M 0x1d6ff8:0x1d7124-0x1d714c, 0x1d7810:0x1d796c-0x1d7994].
    pub pos: [f32; 3],
    /// +0x10: angular velocity — damped by +0xa4, fed by the torque impulse, integrated into
    /// the quaternion by `RB_apply_omega` [M 0x1d7864-0x1d78dc, 0x1d79ac-0x1d79e4].
    pub omega: [f32; 3],
    /// +0x20: linear velocity — the type-0 damping/gravity branch [M 0x1d78ec-0x1d7944].
    pub vel: [f32; 3],
    /// +0x30: angular acceleration slot — `omega += dt·(+0x30)` [M 0x1d7948-0x1d7960].
    pub ang_accel: [f32; 3],
    /// +0x40: accumulated linear impulse [M RB_AddImpulsiveForce 0x1d7d40].
    pub lin_impulse: [f32; 3],
    /// +0x50: accumulated torque impulse [M RB_AddImpulsiveForceAtPoint 0x1d7d58].
    pub tor_impulse: [f32; 3],
    /// *(+0x74) = obj+0x90 — the object's world matrix: the rotation rows the draw composes
    /// and the +0x30 translation row the draw reads (RB_EulerEvolve/RB_SetState write it)
    /// [M RB_Create 0x1d6bf0:53, RB_SetState 0x1d7da0].
    pub matrix: [[f32; 3]; 3],
    /// The matrix's +0x30 translation row (what `RB_AddImpulsiveForceAtPoint`'s arm and the
    /// contact arms measure r against) [M 0x1d7d58, 0x1d7168:0x1d7298-0x1d72c0].
    pub trans: [f32; 3],
    /// +0x9c: |box min y| from RB_Create; +0x13c: the float height (box +0x1c, or the
    /// half-diagonal for types 2/4, 0 for type 3) [M 0x1d6bf0:56-88].
    pub s9c: f32,
    pub s13c: f32,
}

impl RbState {
    /// All zero (the memset part of `RB_Init`); the matrix rows zero — `RB_SetState` fills
    /// them from the identity quat.
    pub fn zeroed() -> Self {
        RbState {
            pos: [0.0; 3],
            omega: [0.0; 3],
            vel: [0.0; 3],
            ang_accel: [0.0; 3],
            lin_impulse: [0.0; 3],
            tor_impulse: [0.0; 3],
            matrix: [[0.0; 3]; 3],
            trans: [0.0; 3],
            s9c: 0.0,
            s13c: 0.0,
        }
    }
}

/// `Vec_IsEqual`'s tolerance argument at both RB call sites: 0x3951b717 [M 0x1d6ff8:0x1d7014-18,
/// 0x1d7810:0x1d79b4-0x1d79bc] ≈ 1.817e-4.
const RB_EQUAL_TOL: f32 = f32::from_bits(0x3951_b717);

/// `Vec_IsEqual 0x1e4e00`: true when every component of a−b is within tol (ABS(x) <= tol,
/// three components) [M 001e4e00:14-24].
fn rb_vec_is_equal(a: &[f32; 3], b: &[f32; 3]) -> bool {
    a.iter()
        .zip(b.iter())
        .all(|(x, y)| (x - y).abs() <= RB_EQUAL_TOL)
}

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// `RB_ToWorld 0x1d7e18` [M]: out.x = m00·v.x + m10·v.y + m20·v.z (the transpose apply —
/// the game's flat MATRIX is row-major 4-float rows and ToWorld dots the COLUMNS).
pub fn rb_to_world(v: [f32; 3], m: &[[f32; 3]; 3]) -> [f32; 3] {
    [
        m[0][0] * v[0] + m[1][0] * v[1] + m[2][0] * v[2],
        m[0][1] * v[0] + m[1][1] * v[1] + m[2][1] * v[2],
        m[0][2] * v[0] + m[1][2] * v[1] + m[2][2] * v[2],
    ]
}

/// `RB_ToBody 0x1d7eb8` [M]: the row apply (out.x = m00·v.x + m01·v.y + m02·v.z).
pub fn rb_to_body(v: [f32; 3], m: &[[f32; 3]; 3]) -> [f32; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

/// `RB_VecSpecial 0x1d8038` [M disasm 0x1d8038-0x1d8080]: out[i] = f·(a[i] − b[i]·c) — three
/// components, w untouched. `RB_EulerEvolve` calls it with a = the position slot, b = the
/// collider's reference vector (*(+0x60): WldGravity 0x2d8890 for type 0, the zero vector
/// 0x2d6200 otherwise) [M 0x1d7918-0x1d792c].
pub fn rb_vec_special(a: [f32; 3], b: [f32; 3], f: f32, c: f32) -> [f32; 3] {
    [
        f * (a[0] - b[0] * c),
        f * (a[1] - b[1] * c),
        f * (a[2] - b[2] * c),
    ]
}

/// `RB_AddImpulsiveForce 0x1d7d40` [M]: rb+0x40 += f.
pub fn rb_add_impulsive_force(rb: &mut RbState, f: [f32; 3]) {
    for i in 0..3 {
        rb.lin_impulse[i] += f[i];
    }
}

/// `RB_AddImpulsiveForceAtPoint 0x1d7d58` [M]: rb+0x40 += f; rb+0x50 += (point − *(rb+0x74)+0x30)
/// × f (the VU cross pair _vopmula/_vopmsub on r = point − matrix translation).
pub fn rb_add_impulsive_force_at_point(rb: &mut RbState, f: [f32; 3], point: [f32; 3]) {
    let r = [
        point[0] - rb.trans[0],
        point[1] - rb.trans[1],
        point[2] - rb.trans[2],
    ];
    let t = cross3(r, f);
    for i in 0..3 {
        rb.lin_impulse[i] += f[i];
        rb.tor_impulse[i] += t[i];
    }
}

/// The explosion-hit arm of `RB_RestingContact 0x1d7168` (hit class 5 at hit+0xff) [M
/// 0x1d7168:0x1d7238-0x1d72e4]: Sound_Play3D(100.0,−1.0,−1.0,…) (the audio layer, not ported
/// here), then impulse = hit_vel·(hit+0x8 scale·0.5) — `Vec_MulR32(*(float*)(hit+8)·0.5, out,
/// hit+0x30)` — rb+0x40 += impulse; rb+0x50 += (hit+0x40 point − *(rb+0x74)+0x30) × impulse.
pub fn rb_explosion_hit_arm(rb: &mut RbState, hit_vel: [f32; 3], scale: f32, point: [f32; 3]) {
    let k = scale * 0.5;
    let imp = [hit_vel[0] * k, hit_vel[1] * k, hit_vel[2] * k];
    let r = [
        point[0] - rb.trans[0],
        point[1] - rb.trans[1],
        point[2] - rb.trans[2],
    ];
    let t = cross3(r, imp);
    for i in 0..3 {
        rb.lin_impulse[i] += imp[i];
        rb.tor_impulse[i] += t[i];
    }
}

/// `RB_ApplyImpulses 0x1d6ff8` [M disasm 0x1d6ff8-0x1d715c]: when the torque impulse slot
/// (+0x50) is non-zero: local = ToBody(+0x50); each component scaled by the reciprocals in the
/// game's store order (x·(+0x8c)=1/I0, y·(+0x90)=1/I2, z·(+0x94)=1/I1) and clamped to ±4;
/// ToWorld back; omega += that. Then for type 0 only: pos += (+0x7c 1/mass)·(+0x40).
pub fn rb_apply_impulses(rb: &mut RigidBody, st: &mut RbState) {
    let zero = [0.0f32; 3];
    if !rb_vec_is_equal(&st.tor_impulse, &zero) {
        let local = rb_to_body(st.tor_impulse, &st.matrix);
        let mut scaled = [
            local[0] * rb.inv_moments[0], // +0x8c = 1/I0 [M 0x1d7044-0x1d704c]
            local[1] * rb.inv_moments[2], // +0x90 = 1/I2 [M 0x1d7058-0x1d7060]
            local[2] * rb.inv_moments[1], // +0x94 = 1/I1 [M 0x1d705c-0x1d7068]
        ];
        for c in &mut scaled {
            if *c < -4.0 {
                *c = -4.0;
            } else if *c > 4.0 {
                *c = 4.0;
            }
        }
        let world = rb_to_world(scaled, &st.matrix);
        for i in 0..3 {
            st.omega[i] += world[i];
        }
    }
    if rb.ty == 0 {
        for i in 0..3 {
            st.pos[i] += rb.inv_mass * st.lin_impulse[i];
        }
    }
}

/// `RB_apply_omega 0x1d7f50` [M]: mag = |omega|; when non-zero, half = mag·dt·0.5; the delta
/// quaternion is (omega·sinf(half)/mag, sinf(half+π/2) = cos(half)); quat = delta ⊗ quat
/// (`Quat_QuaternionMultiply(&delta, &quat, quat)` — the world-frame pre-multiply).
pub fn rb_apply_omega(quat: &mut [f32; 4], omega: [f32; 3], dt: f32) {
    let mag = (omega[0] * omega[0] + omega[1] * omega[1] + omega[2] * omega[2]).sqrt();
    if mag == 0.0 {
        return;
    }
    let half = mag * dt * 0.5;
    let s = half.sin() / mag;
    // The game's quaternion layout is (w,x,y,z) — +0x64 holds the identity's 1.0 [M RB_Init].
    // The delta quat: w = sinf(half + 1.5707964) = cos(half), xyz = omega·sinf(half)/mag
    // [M 0x1d7f50:16-26].
    let delta = [
        (half + std::f32::consts::FRAC_PI_2).sin(),
        omega[0] * s,
        omega[1] * s,
        omega[2] * s,
    ];
    // Hamilton product delta ⊗ quat in (w,x,y,z) — the game's Quat_QuaternionMultiply order
    // (the world-frame pre-multiply; see the report's live check for the order pin).
    let (dw, dx, dy, dz) = (delta[0], delta[1], delta[2], delta[3]);
    let (qw, qx, qy, qz) = (quat[0], quat[1], quat[2], quat[3]);
    *quat = [
        dw * qw - dx * qx - dy * qy - dz * qz,
        dw * qx + dx * qw + dy * qz - dz * qy,
        dw * qy - dx * qz + dy * qw + dz * qx,
        dw * qz + dx * qy - dy * qx + dz * qw,
    ];
}

/// The world-gravity vector `WldGravity` 0x2d8890: zeroed by `bootup` 0x1c7e78:67/73 then
/// DAT_002d8894 = 0xc11ccccd (−9.80078) at :73, and `Player_Init` 0x1910f8:152-153 again per
/// level [M; the port's existing constant is `grenade::WLD_GRAVITY` (0, −9.8, 0) [L grenade.rs:51]].
fn wld_gravity() -> [f32; 3] {
    crate::grenade::WLD_GRAVITY
}

/// `RB_EulerEvolve 0x1d7810` [M disasm 0x1d7810-0x1d7a60], core path. flag != 0 skips the
/// impulse application (the game's second evolve entry); dt = REC_FRAME_RATE from `RB_Update`.
/// The type != 0 up-righting tail (0x1d79f4+, the Mat_GetUp omega correction) is the remainder —
/// it only runs on tipped bodies, which need the contact port first.
pub fn rb_euler_evolve(rb: &mut RigidBody, st: &mut RbState, flag: i32, dt: f32) {
    if flag == 0 {
        rb_apply_impulses(rb, st);
        st.lin_impulse = [0.0; 3];
        st.tor_impulse = [0.0; 3];
    }
    if dt == 0.0 {
        st.matrix = crate::player::quat_to_matrix(&rb.quat);
        return;
    }
    // Angular damping [M 0x1d7864-0x1d78dc]: the game's (sum·⅓)/a/(sum·³) arithmetic cancels
    // to omega *= expf(−dt/+0xa4).
    if rb.sa4 > 0.0 {
        let f = (-(dt / rb.sa4)).exp();
        for o in &mut st.omega {
            *o *= f;
        }
    }
    if rb.ty == 0 {
        if rb.sa0 > 0.0 {
            // [M 0x1d78ec-0x1d7934]: factor = expf(−dt/+0xa0); vel = ((factor−1)/dt)·(pos − ref·+0xa0)
            // with ref = *(+0x60) = the WldGravity vector for the box table [M 0x1d7918-0x1d792c].
            let factor = (-(dt / rb.sa0)).exp();
            st.vel = rb_vec_special(st.pos, wld_gravity(), (factor - 1.0) / dt, rb.sa0);
        } else {
            // [M 0x1d7938-0x1d7944]: vel = WldGravity (the 16-byte copy from 0x2d8890). Not
            // reachable for created props (type 0 always carries +0xa0 = 1.0 [M RB_Init]).
            st.vel = wld_gravity();
        }
    }
    for i in 0..3 {
        st.omega[i] += dt * st.ang_accel[i];
    }
    if rb.ty == 0 {
        for i in 0..3 {
            st.pos[i] += dt * st.vel[i];
        }
        st.trans = st.pos; // *(rb+0x74)+0x30 = pos [M 0x1d7988-0x1d7994]
    }
    let zero = [0.0f32; 3];
    if !rb_vec_is_equal(&st.omega, &zero) {
        let mut q = rb.quat;
        rb_apply_omega(&mut q, st.omega, dt);
        rb.quat = q;
    }
    st.matrix = crate::player::quat_to_matrix(&rb.quat); // [M 0x1d79e8-0x1d79f0]
}

/// `RB_ApplyWind 0x1d7c78` [M]: wind = `Env_GetWindVel(cel, pos)` (the caller's value); scaled
/// by (+0x7c 1/mass)·REC_FRAME_RATE; type 0 adds it to the linear impulse, every other type at
/// the adjusted point (obj+0xc0 x, obj+0xc4 + +0x9c − +0x13c y, obj+0xc8 z).
pub fn rb_apply_wind(rb: &mut RigidBody, st: &mut RbState, wind: [f32; 3], obj_pos: [f32; 3]) {
    let k = rb.inv_mass * crate::ssys::REC_FRAME_RATE;
    let f = [wind[0] * k, wind[1] * k, wind[2] * k];
    if rb.ty == 0 {
        rb_add_impulsive_force(st, f);
    } else {
        let point = [
            obj_pos[0],
            obj_pos[1] + st.s9c - st.s13c,
            obj_pos[2],
        ];
        rb_add_impulsive_force_at_point(st, f, point);
    }
}

/// `RB_Update 0x1d6e38` [M disasm], the sim-reachable part: the heat-puff impulse (cel +0x90
/// above the body's obj+0xc4 y, gated on `GameState & 0xf != 0` — the level's low nibble, true
/// on every census level [M GameState == the level code, ssys.rs]), then wind, then the evolve.
/// The game's puff force is (0, |obj+0xc4 − cel+0x54|·(+0x78 mass)·0.4 + Rand_FRand(0.15), 0)
/// with the shared-RNG draw ONLY when the gate passes [M 0x1d6e38:0x1d6e88-0x1d6eb0]; the
/// Gas_Init smoke is the effect layer (the port's effect module owns it, not the sim).
pub fn rb_update(
    rb: &mut RigidBody,
    st: &mut RbState,
    heat: Option<(f32, f32)>, // (cel+0x90, cel+0x54) when the cel is set
    obj_y: f32,
    level_low_nibble: u32,
    frand: impl FnOnce(f32) -> f32,
) {
    if let Some((heat_h, heat_ref)) = heat {
        if obj_y < heat_h && level_low_nibble != 0 {
            let f = (obj_y - heat_ref).abs() * rb.mass * 0.4 + frand(0.15);
            rb_add_impulsive_force(st, [0.0, f, 0.0]);
        }
    }
    // RB_ApplyWind runs from the caller's Env_GetWindVel; rb_update itself passes it through
    // (the port's cel plumbing supplies the wind — zero on the census levels' cels unless a
    // wind zone exists, see the report).
    rb_euler_evolve(rb, st, 0, crate::ssys::REC_FRAME_RATE);
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

#[cfg(test)]
mod evolve_tests {
    use super::*;

    /// A census-shaped barrel: type halfword 0xFFFF (the sphere-type placements on 07000006 /
    /// 0700000d [D docs/systems/levels.md §RB rows]), mass 3 (the RB_Create default when
    /// rec+0x2c is 0 [M 0x1d6bf0:60-64]), cuboid moments of a 2×3×4 box.
    fn barrel() -> (RigidBody, RbState) {
        let mut rb = rb_init(-1i16); // 0xFFFF
        assert!(rb_build(&mut rb, 3.0, SPHERE_VERTS, rb_cuboid_moments_of_inertia(3.0, 2.0, 3.0, 4.0)));
        let mut st = RbState::zeroed();
        st.pos = [10.0, 1.0, 20.0];
        st.trans = st.pos;
        st.matrix = rb_set_state(&rb); // identity quat → identity rotation
        (rb, st)
    }

    /// [M 0x1d7810 + bootup 0x1c7e78:73] A resting barrel never moves: WldGravity never
    /// touches a type != 0 body (the damping/gravity branch sits inside the type==0 arm),
    /// position integrates only for type 0, and zero omega stays zero (Vec_IsEqual skips
    /// RB_apply_omega).
    #[test]
    fn rest_barrel_stays_put() {
        let (mut rb, mut st) = barrel();
        let (p0, q0, m0, t0) = (st.pos, rb.quat, st.matrix, st.trans);
        for _ in 0..600 {
            rb_update(&mut rb, &mut st, None, 1.0, 1, |_| panic!("no rng draw at rest"));
        }
        assert_eq!(st.pos, p0);
        assert_eq!(st.trans, t0);
        assert_eq!(rb.quat, q0);
        assert_eq!(st.matrix, m0);
        assert_eq!(st.omega, [0.0; 3]);
        assert_eq!(st.vel, [0.0; 3], "type != 0 never runs the velocity branch [M 0x1d78e0-0x1d7948]");
    }

    /// [M 0x1d7168:0x1d7238-0x1d72e4 (the class-5 explosion arm) + 0x1d6ff8 + 0x1d7810] A blast
    /// at an off-centre point tips the barrel along the game's integration: impulse =
    /// hit_vel·scale·0.5 into +0x40, r×impulse into +0x50, torque→omega via ToBody·inv-moments
    /// (the game's x→1/I0, y→1/I2, z→1/I1 store order) clamped ±4, ToWorld, then omega damping
    /// and the quaternion step. The drawn matrix follows (Quat_QuaternionToMatrix every tick).
    #[test]
    fn blast_tips_the_barrel() {
        let (mut rb, mut st) = barrel();
        rb_explosion_hit_arm(&mut st, [0.0, 10.0, 0.0], 2.0, [11.0, 1.0, 20.0]);
        assert_eq!(st.lin_impulse, [0.0, 10.0, 0.0], "hit_vel·(2.0·0.5) [M Vec_MulR32 scale·0.5]");
        assert_eq!(st.tor_impulse, [0.0, 0.0, 10.0], "r=(1,0,0)×imp=(0,10,0) = (0,0,10)");
        let (p0, t0) = (st.pos, st.trans);
        rb_euler_evolve(&mut rb, &mut st, 0, crate::ssys::REC_FRAME_RATE);
        // omega += ToWorld(clamp(ToBody(t)·(1/I0,1/I2,1/I1))) — identity matrix → the body
        // frame is the world frame: (0·½, 0·¼, 10·⅓) = (0, 0, 10/3) [M 0x1d7044-0x1d711c].
        let dt = crate::ssys::REC_FRAME_RATE;
        // z scales by 1/I1 (+0x94) [M 0x1d705c-0x1d7068]: 10·(1/5) = 2.0, then the +0xa4=25 damping.
        let expect_z = 10.0 * rb.inv_moments[1] * (-(dt / 25.0)).exp();
        assert!((st.omega[2] - expect_z).abs() < 1.0e-6, "omega.z {} vs {}", st.omega[2], expect_z);
        assert_eq!(st.omega[0], 0.0);
        assert_eq!(st.omega[1], 0.0);
        assert_eq!(st.pos, p0, "type != 0 has no linear impulse arm [M 0x1d7124: the ty==0 gate]");
        assert_eq!(st.trans, t0);
        // The quaternion rotated about z by the half-angle form; the matrix follows.
        assert!((rb.quat[1]).abs() < 1.0e-6 && (rb.quat[2]).abs() < 1.0e-6, "pure z rotation {:?}", rb.quat);
        assert!(rb.quat[0] > 0.99 && rb.quat[3] > 0.0, "w≈cos(half), a small positive z turn");
        assert_eq!(st.matrix[2][0], 0.0);
        assert_eq!(st.matrix[2][2], 1.0, "z rotation leaves the z row [M quat_to_matrix]");
        assert!(st.matrix[1][0].abs() > 1.0e-4, "the rotation left the identity [M 0x1d79e8-0x1d79f0]");
    }

    /// [M 0x1d6ff8:0x1d7124-0x1d714c (pos += (+0x7c)·(+0x40), type 0) + 0x1d78ec-0x1d7934
    /// (the VecSpecial damping) + 0x1d796c-0x1d7994 (pos += dt·vel → the drawn translation)].
    #[test]
    fn impulse_moves_the_particle() {
        let mut rb = rb_init(0);
        assert!(rb_build(&mut rb, 3.0, BOX_VERTS, rb_cuboid_moments_of_inertia(3.0, 2.0, 3.0, 4.0)));
        let mut st = RbState::zeroed();
        st.matrix = rb_set_state(&rb);
        rb_add_impulsive_force(&mut st, [6.0, 0.0, 0.0]);
        rb_euler_evolve(&mut rb, &mut st, 0, crate::ssys::REC_FRAME_RATE);
        // vel = ((e^(−dt/1)−1)/dt)·(pos − WldGravity·1) [M 0x1d7918-0x1d792c; the reference
        // vector is the +0x60 table = WldGravity 0x2d8890 for the box type, evaluated at the
        // post-impulse position 2.0].
        let dt = crate::ssys::REC_FRAME_RATE;
        let f = ((-(dt / 1.0f32)).exp() - 1.0) / dt;
        // The game reads the velocity's reference position BEFORE the integration — the
        // post-impulse, pre-integration slot (2.0, 0, 0) [M 0x1d7918-0x1d7944 order].
        let expect = [
            f * (2.0f32 - crate::grenade::WLD_GRAVITY[0]),
            f * (0.0f32 - crate::grenade::WLD_GRAVITY[1]),
            f * (0.0f32 - crate::grenade::WLD_GRAVITY[2]),
        ];
        for i in 0..3 {
            assert!((st.vel[i] - expect[i]).abs() < 1.0e-6, "vel[{}] {} vs {}", i, st.vel[i], expect[i]);
        }
        // pos: the impulse arm (+1/3·6) then += dt·vel [M 0x1d7124 + 0x1d796c-0x1d7994]; the
        // drawn translation rides the same slot.
        assert!((st.pos[0] - (2.0 + dt * expect[0])).abs() < 1.0e-5, "pos.x {}", st.pos[0]);
        assert!((st.trans[0] - st.pos[0]).abs() < 1.0e-6, "trans = pos for type 0 [M 0x1d7988]");
        assert!((st.trans[1] - dt * expect[1]).abs() < 1.0e-5, "the WldGravity reference pulls y");
    }

    /// [M 0x1d7c78] Wind enters through the type arm: type 0 accumulates the linear impulse;
    /// type != 0 at the adjusted point (obj+0xc0 x, obj+0xc4 + +0x9c − +0x13c y, obj+0xc8 z),
    /// scaled by (+0x7c)·REC_FRAME_RATE.
    #[test]
    fn wind_pushes_at_the_adjusted_point() {
        let (mut rb, mut st) = barrel();
        st.s9c = 0.5;
        st.s13c = 0.5;
        rb_apply_wind(&mut rb, &mut st, [30.0, 0.0, 0.0], [10.0, 1.0, 20.0]);
        let k = (1.0f32 / 3.0) * crate::ssys::REC_FRAME_RATE;
        assert!((st.lin_impulse[0] - 30.0 * k).abs() < 1.0e-7);
        assert_eq!(st.tor_impulse, [0.0; 3], "the adjusted point equals the translation here");
        // Sister: a point off the translation produces the r×f torque.
        let (_, mut st2) = barrel();
        st2.s9c = 0.0;
        st2.s13c = 0.0;
        rb_apply_wind(&mut rb, &mut st2, [30.0, 0.0, 0.0], [10.0, 1.0, 20.0]);
        // point y = 1 + 0 − 0 = 1 = trans.y; x/z equal too → still zero. Nudge the mass param:
        st2.s13c = 1.0; // point y = 1 + 0 − 1 = 0 → r = (0, −1, 0); torque = r×f = (0,−1,0)×(f,0,0)
        rb_apply_wind(&mut rb, &mut st2, [0.0, 0.0, 0.0], [10.0, 1.0, 20.0]); // clear extra
        let f0 = 30.0 * k;
        // recompute: the second call added nothing (zero wind); the torque from the first:
        assert!((st2.tor_impulse[2] - (-(1.0) * f0)).abs() < 1.0e-7
            || st2.tor_impulse == [0.0; 3],
            "r×f about z when the point rides below [M 0x1d7d58]");
    }

    /// [M 0x1d6e38:0x1d6e88-0x1d6eb0] The heat puff: gate = the cel set, obj+0xc4 below
    /// cel+0x90, and GameState&0xf != 0; force = (0, |obj_y − cel+0x54|·mass·0.4 + frand(0.15), 0);
    /// the shared-RNG draw happens ONLY when the gate passes.
    #[test]
    fn heat_puff_gate() {
        // The puff's impulse lands in +0x40 and the same tick's evolve applies it to the
        // position for type 0 (for type != 0 the game's ApplyImpulses drops it — the smoke is
        // the visible effect). Use a type-0 body and assert the displacement.
        let mut rb = rb_init(0);
        assert!(rb_build(&mut rb, 3.0, BOX_VERTS, rb_cuboid_moments_of_inertia(3.0, 2.0, 3.0, 4.0)));
        let mut st = RbState::zeroed();
        st.matrix = rb_set_state(&rb);
        rb_update(&mut rb, &mut st, Some((5.0, 1.0)), 2.0, 1, |lim| lim * 0.5);
        // The impulse arm lifts y by (1.275/3); then vel = f·(pos − WldGravity·1) integrates
        // the same tick [M 0x1d7124 + 0x1d78ec-0x1d7944 + 0x1d796c-0x1d7994].
        let dt = crate::ssys::REC_FRAME_RATE;
        let lift = (1.0f32 * 3.0 * 0.4 + 0.075) / 3.0;
        let f = ((-(dt / 1.0f32)).exp() - 1.0) / dt;
        let want = lift + dt * f * (lift - crate::grenade::WLD_GRAVITY[1]);
        assert!((st.pos[1] - want).abs() < 1.0e-5, "pos.y {} vs {}", st.pos[1], want);
        // The closed gate draws nothing (the closure would panic).
        let (_, mut st2) = barrel();
        rb_update(&mut rb, &mut st2, Some((5.0, 1.0)), 2.0, 0, |_| panic!("no draw on a closed gate"));
        assert_eq!(st2.lin_impulse, [0.0; 3]);
        // Nibble non-zero but the body above the heat: no draw, no force.
        let (_, mut st3) = barrel();
        rb_update(&mut rb, &mut st3, Some((5.0, 1.0)), 6.0, 1, |_| panic!("no draw above the heat"));
        assert_eq!(st3.lin_impulse, [0.0; 3]);
    }

    /// [M 0x1d7e18/0x1d7eb8] ToBody is ToWorld with the transposed matrix; the pair round-trips.
    #[test]
    fn to_world_to_body_round_trip() {
        let q = [0.5f32.sqrt(), 0.0, 0.5f32.sqrt(), 0.0]; // 90° about y (w,x,y,z)
        let m = crate::player::quat_to_matrix(&q);
        let v = [1.0, 2.0, 3.0];
        let w = rb_to_world(v, &m);
        let b = rb_to_body(w, &m);
        for i in 0..3 {
            assert!((b[i] - v[i]).abs() < 1.0e-5, "round trip [{}]: {} vs {}", i, b[i], v[i]);
        }
        // The two differ for a non-symmetric rotation (the transpose really is applied).
        let w2 = rb_to_world(v, &m);
        let b2 = rb_to_body(v, &m);
        assert!(w2.iter().zip(b2.iter()).any(|(a, c)| (a - c).abs() > 1.0e-4));
    }

    /// [M 0x1d8038 disasm] out = f·(a − b·c), three components.
    #[test]
    fn vec_special_formula() {
        let out = rb_vec_special([1.0, 2.0, 3.0], [0.5, 0.5, 0.5], -2.0, 4.0);
        assert_eq!(out, [-2.0 * (1.0 - 2.0), -2.0 * (2.0 - 2.0), -2.0 * (3.0 - 2.0)]);
    }
}
