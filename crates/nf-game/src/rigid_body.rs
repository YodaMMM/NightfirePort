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

// ============================================================================
// The wiring (rigid-body-wiring): the contact arms + the sim-object container.
// ============================================================================

/// One entry of the object's hit list (obj+0xD0) — the HITDATA subset the RB arms read:
/// +0x8 the scale, +0x10 the contact normal, +0x30 the velocity vector, +0x40 the contact
/// point, the other object's class byte (hit+0x58 → obj+0xFF) and its position (obj+0xC0)
/// [M RB_RestingContact 0x1d7168 reads the class byte at :0x1d7218, the other's obj+0xC0 at
/// LAB_001d7298, CalcTorque's normal at hit+0x10 (0x1d7658:11-13)].
#[derive(Clone, Debug, PartialEq)]
pub struct RbHit {
    pub scale: f32,
    pub normal: [f32; 3],
    pub vel: [f32; 3],
    pub point: [f32; 3],
    /// The other object's `obj+0xFF` class byte [M 0x1d7168:0x1d7218].
    pub other_class: u8,
    /// The other object's `obj+0xC0` (its matrix translation) for the position-correction arm.
    pub other_pos: [f32; 3],
}

/// `Vec_MakeOrthonormalBasis` 0x1e4b78 [M]: two unit vectors ⟂ n, picked by the smaller
/// first-component branch. n = hit+0x10..0x1c (the contact normal) at the CalcTorque call.
pub fn vec_make_orthonormal_basis(n: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    if n[0].abs() <= n[1].abs() {
        let inv = 1.0 / (n[2] * n[2] + n[1] * n[1]).sqrt();
        let b0 = [0.0, -n[2] * inv, n[1] * inv];
        let b1 = [n[1] * b0[2] - n[2] * b0[1], -n[0] * b0[2], n[0] * b0[1]];
        (b0, b1)
    } else {
        let inv = 1.0 / (n[2] * n[2] + n[0] * n[0]).sqrt();
        let b0 = [n[2] * inv, 0.0, -n[0] * inv];
        let b1 = [n[1] * b0[2], n[2] * b0[0] - n[0] * b0[2], -n[1] * b0[0]];
        (b0, b1)
    }
}

/// `RB_CalcTorque` 0x1d7658 [M decompile, literal]: r = point − trans; basis (b0, b1) from the
/// normal; the three component sums are dots of the basis vectors with *(rb+0x00) (the
/// position slot — the decompile loads `*param_1`, the RIGIDBODY base); per axis, when
/// |dot| > 0.05: +0x50 += (r × b)·(dot·−0.01), the normal axis at dot·−0.2. The 4th VU lane
/// of the accumulator is not ported (the port's slots are 3-wide).
pub fn rb_calc_torque(rb: &RigidBody, st: &mut RbState, normal: [f32; 3], point: [f32; 3]) {
    let _ = rb;
    let r = [point[0] - st.trans[0], point[1] - st.trans[1], point[2] - st.trans[2]];
    let (b0, b1) = vec_make_orthonormal_basis(normal);
    let dot = |a: [f32; 3]| a[0] * st.pos[0] + a[1] * st.pos[1] + a[2] * st.pos[2];
    let f0 = dot(b0);
    let f1 = dot(b1);
    let f2 = dot(normal);
    if f0.abs() > 0.05 {
        let t = cross3(r, b0);
        for i in 0..3 { st.tor_impulse[i] += t[i] * (f0 * -0.01); }
    }
    if f1.abs() > 0.05 {
        let t = cross3(r, b1);
        for i in 0..3 { st.tor_impulse[i] += t[i] * (f1 * -0.01); }
    }
    if f2.abs() > 0.05 {
        let t = cross3(r, normal);
        for i in 0..3 { st.tor_impulse[i] += t[i] * (f2 * -0.2); }
    }
}

/// The class-0 contact arm of `RB_RestingContact` 0x1d7168: LAB_001d7298 [M] — d =
/// other_pos(obj+0xC0) − hit point; when |d|² < 0.5 the drawn translation (obj+0x90+0x30)
/// += d. Classes 0/1/2/4 and every class above 6 except 0x1a reach it (the walk's dispatch);
/// class 3 and class 0x1a are skipped [M 0x1d7168:0x1d7218-0x1d72e4].
fn rb_contact_position_correction(st: &mut RbState, other_pos: [f32; 3], point: [f32; 3]) {
    let d = [other_pos[0] - point[0], other_pos[1] - point[1], other_pos[2] - point[2]];
    if d[0] * d[0] + d[1] * d[1] + d[2] * d[2] < 0.5 {
        for i in 0..3 { st.trans[i] += d[i]; }
    }
}

/// `RB_RestingContact` 0x1d7168 [M], the type-0 hit-list walk (the port's hit records — the
/// game's obj+0xD0 list). Per hit [M 0x1d71f8-0x1d72e4]: when `*(short*)(rb+0xAA) != 0x400`
/// run `RB_CalcTorque(rb, hit)`; then the class dispatch on the other object's obj+0xFF:
/// 5 → the explosion arm (impulse = (hit+0x8)·0.5·vel into +0x40, r×impulse into +0x50 —
/// the ported `rb_explosion_hit_arm`; the Sound_Play3D is the audio layer, collected not
/// played), 3 → nothing, 0x1a (another RB body) → nothing, classes 0/1/2/4 and every class
/// above 6 except 0x1a → the position correction (LAB_001d7298: when
/// |other_pos − hit.point|² < 0.5, trans += that delta). The list tail's
/// `rb+0xC0 != 0x2d6200` grounding nudge is inert on this path (nothing in the ported path
/// writes rb+0xC0 — it stays zero == the 0x2d6200 table, so `Vec_IsEqual` gates it off
/// [M 0x1d72e4-0x1d7344]; the writer is the unported cylinder pass — the report).
pub fn rb_resting_contact(
    rb: &RigidBody,
    st: &mut RbState,
    s_aa: u16,
    hits: &[RbHit],
    sounds: &mut Vec<(u32, [f32; 3])>,
) {
    for hit in hits {
        if s_aa != 0x400 {
            rb_calc_torque(rb, st, hit.normal, hit.point);
        }
        match hit.other_class {
            5 => {
                rb_explosion_hit_arm(st, hit.vel, hit.scale, hit.point);
                // `Sound_Play3D(100.0, −1.0, −1.0, *(rb+0x158), obj+0xC0, …)` [M 0x1d7220] —
                // the caller maps the collected code onto the port's sound pool.
                sounds.push((0x42c8_0000, st.trans));
            }
            3 => {}
            0x1a => {}
            _ => rb_contact_position_correction(st, hit.other_pos, hit.point),
        }
    }
}

/// `RB_RestingContactRot` 0x1d73a0 [M], the type != 0 hit-list walk — the arm the census
/// barrels (type 0xFFFF) take. Per hit, class 5 only [M 0x1d7428-0x1d74c8]: Sound_Play3D
/// (collected, not played), impulse = (hit+0x8)·0.5·vel, +0x40 += impulse and +0x50 +=
/// r×impulse with the PER-TYPE masking of the cross lanes: type 3 zeroes the xyz lanes,
/// type 4 zeroes more, type 1 zeroes the second lane; every other type (incl. 0xFFFF/−1)
/// adds the full cross. Other classes fall through — Rot has NO position-correction arm
/// [M the decompile's only else arm walks to the next hit]. The cylinder interior (the
/// +0xD0/+0xE0/+0x100 test points, the `Collide_CylinderIntersect` sweep and its
/// uprighting tail) is the documented remainder: it needs the port's cylinder sweep (the
/// remainder report's item 2).
pub fn rb_resting_contact_rot(
    rb: &RigidBody,
    st: &mut RbState,
    hits: &[RbHit],
    sounds: &mut Vec<(u32, [f32; 3])>,
) {
    for hit in hits {
        if hit.other_class == 5 {
            let k = hit.scale * 0.5;
            let imp = [hit.vel[0] * k, hit.vel[1] * k, hit.vel[2] * k];
            let r = [hit.point[0] - st.trans[0], hit.point[1] - st.trans[1], hit.point[2] - st.trans[2]];
            let cross = cross3(r, imp);
            let masked = match rb.ty {
                3 | 4 => [0.0f32; 3], // the lane masks zero the whole vector [M 0x1d7478+]
                1 => [cross[0], 0.0, cross[2]], // the second lane zeroed [M auStack_120._4_4_ = 0]
                _ => cross,
            };
            for i in 0..3 {
                st.tor_impulse[i] += masked[i];
                st.lin_impulse[i] += imp[i];
            }
            sounds.push((0x42c8_0000, st.trans));
        }
    }
}

/// `RB_CollisionHandler` 0x1d7a80 [M]: type 0 → RestingContact, every other type →
/// RestingContactRot; the caller then applies obj+0xFA |= 0x20.
pub fn rb_collision_handler(
    rb: &RigidBody,
    st: &mut RbState,
    s_aa: u16,
    hits: &[RbHit],
    sounds: &mut Vec<(u32, [f32; 3])>,
) {
    if rb.ty == 0 {
        rb_resting_contact(rb, st, s_aa, hits, sounds);
    } else {
        rb_resting_contact_rot(rb, st, hits, sounds);
    }
}

#[derive(Clone)]
/// The kind-0xD8 sim object (`RB_Create` 0x1d6bf0's): the placement's body with its runtime
/// slots and the obj-side fields the game's object carries. `st.pos`/`st.trans` start at the
/// placement translation — the at-rest state IS the static draw [D the props draw correctly
/// as statics today; the drawn translation follows rb+0x00 [M EulerEvolve], so the game's
/// rest-state identity pins the init; the literal create-time store is a live check].
pub struct RbObj {
    /// The placement index (the control walk's key).
    pub placement: u16,
    /// obj+0xD8: the placement's glist (the draw/collide shape).
    pub glist: u32,
    /// obj+0xFA: 0x44 at create [M RB_Create :56], |= 0x20 per update/handler [M RB_Update tail].
    pub flags: u16,
    /// rb+0xAA: 0x101 default, 0 for types 2/3/4/others per RB_Create's tail [M 0x1d6bf0:80-96].
    pub s_aa: u16,
    /// rb+0x154 / rb+0x158 (the light flag byte, the sound-volume param word) [M :100-104].
    pub s154: u8,
    pub s158: u32,
    /// obj+0xE0 → the RIGIDBODY: the init/build fields and the runtime slots.
    pub rb: RigidBody,
    pub st: RbState,
    /// obj+0xD0: this tick's hit list (the collide pass fills it; the handler drains it).
    pub hits: Vec<RbHit>,
}

impl RbObj {
    /// obj+0x90: the drawn/collided matrix — the quaternion rows (`rb_set_state`/EulerEvolve)
    /// with the +0x30 translation row = `st.trans` [M RB_SetState 0x1d7da0, EulerEvolve :96].
    pub fn matrix(&self) -> nf_formats::anim::Mat4 {
        let m = &self.st.matrix;
        [
            [m[0][0], m[0][1], m[0][2], 0.0],
            [m[1][0], m[1][1], m[1][2], 0.0],
            [m[2][0], m[2][1], m[2][2], 0.0],
            [self.st.trans[0], self.st.trans[1], self.st.trans[2], 1.0],
        ]
    }
}

#[derive(Clone)]
/// All kind-0xD8 bodies of the main map, in placement order (`parsemap_create_dynamic_objects`
/// 0x1d04f0 case 0xD8 — the main map's placements only, the parsemap-epilogue rule the port's
/// other creators follow).
#[derive(Default)]
pub struct Rbodies {
    pub list: Vec<RbObj>,
}

impl Rbodies {
    /// `RB_Create` 0x1d6bf0 [M], per main-map 0xD8 placement:
    /// type = rec+0x34 u16 (`extra[4]` low half); mass = rec+0x2c (`extra[2]`) — 0 → 3.0,
    /// sign-bit set → the raw word as f32, else the i32 as f32 [M :60-64]; extents
    /// a/b/c = glist max.x−min.x / max.y−min.y / max.z−min.z [M :66-72]; +0x9c = |min.y|
    /// [M :73]; the vert table by type (0 → BOX_VERTS 0x2d8890, else SPHERE_VERTS 0x2d6200)
    /// [M :74-78]; +0xAA = 0x101, then 0 for types 2/3/4 and the sqrt-half-diagonal into
    /// +0x13c for the others (type 3 keeps glist+0x1c) [M :80-96]; +0x154 = rec+0x38 byte,
    /// +0x158 = rec+0x3c (0 → 0xFFFF) [M :100-104]. obj+0xFA |= 0x44 [M :56].
    /// The build-failure path (the moments' triangle check) leaves the body unbuilt and the
    /// object absent from the sim list — the game's `control_create_object` object exists but
    /// the port models the evolved body only ([S] the degenerate census case: no 0xD8
    /// placement on the disc fails the check).
    pub fn create(level: &nf_formats::level::Level) -> Rbodies {
        let mut list = Vec::new();
        let Some(m) = level.main_map() else { return Rbodies { list } };
        for (pi, p) in level.maps[m].placements.iter().enumerate() {
            if p.flags & 0xff != 0xd8 { continue; }
            // The placement → object resolve, the doors' `object(p)` convention: by-hash through
            // the level's table, else the placement's own index.
            let resolved = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() } else { Some((m, p.index as usize)) };
            let Some((map, object)) = resolved else { continue };
            let Some(o) = level.maps[map].objects.get(object) else { continue };
            // The record words: `RB_Create` reads the mass at rec+0x2c and the type halfword at
            // rec+0x34 [M 0x1d6bf0:52-56]. The placement's param words are the (slot, value)
            // pairs the parse carries (`Placement.params`, slot k = the record word +0x24+4k —
            // the same words `extra` mirrors; the parse leaves `extra` zeroed on these rows, so
            // the read goes through `params`). Slot 2 = +0x2c (the mass word), slot 4 = +0x34
            // (the type halfword): 07000004's lamps carry (2,1) and (4,65535) [D], 07000006's
            // barrels (0,8) and (4,65535) [D].
            let pv = |k: u32| p.params.iter().find(|(j, _)| *j == k).map(|&(_, v)| v).unwrap_or(0);
            let ty = (pv(4) & 0xffff) as u16 as i16;
            let mass_word = pv(2);
            let mass = if mass_word == 0 { 3.0 }
                else if mass_word & 0x8000_0000 != 0 { mass_word as f32 }
                else { (mass_word as i32) as f32 };
            let ext = [
                o.max[0] - o.min[0],
                o.max[1] - o.min[1],
                o.max[2] - o.min[2],
            ];
            let moments = rb_cuboid_moments_of_inertia(mass, ext[0], ext[1], ext[2]);
            let mut rb = rb_init(ty);
            let table = if ty == 0 { BOX_VERTS } else { SPHERE_VERTS };
            let _built = rb_build(&mut rb, mass, table, moments);
            let mut st = RbState::zeroed();
            st.pos = p.translation;
            st.trans = p.translation;
            st.matrix = rb_set_state(&rb);
            st.s9c = o.min[1].abs();
            st.s13c = o.radius; // glist+0x1C [M Control_BuildWorldSph 0x133070's +0x10/+0x1C sphere]
            // rb+0xAA + the +0x13c overrides [M 0x1d6bf0:80-96]
            let (s_aa, s13c) = rb_create_tail(ty, ext);
            if let Some(v) = s13c { st.s13c = v; }
            list.push(RbObj {
                placement: pi as u16,
                glist: o.hashcode,
                flags: 0x44,
                s_aa,
                s154: (pv(5) & 0xff) as u8,
                s158: if pv(6) == 0 { 0xffff } else { pv(6) },
                rb,
                st,
                hits: Vec::new(),
            });
        }
        Rbodies { list }
    }

    /// `RB_Update` 0x1d6e38 [M] per body: the heat-puff gate (the port passes no cel — the
    /// port has no per-object room cel yet [S], so no draw and no puff), zero wind (the
    /// `Env_GetWindVel` census is open [S]), the evolve, and the flags/light bookkeeping the
    /// sim reads (obj+0xFA |= 0x20 [M :tail]; the +0x100 light-position sync is the visual
    /// layer — not ported).
    pub fn tick(&mut self, level_low_nibble: u32, rng: &mut crate::drone::Rng) {
        for o in &mut self.list {
            let obj_y = o.st.trans[1];
            crate::rigid_body::rb_update(&mut o.rb, &mut o.st, None, obj_y, level_low_nibble, |m| rng.frand(m));
            o.flags |= 0x20;
        }
    }

    /// `RB_CollisionHandler` 0x1d7a80 [M] per body over its hit list; the sounds collect for
    /// the caller's pool.
    pub fn handle_collisions(&mut self, sounds: &mut Vec<(u32, [f32; 3])>) {
        for o in &mut self.list {
            if o.hits.is_empty() { continue; }
            let hits = std::mem::take(&mut o.hits);
            rb_collision_handler(&o.rb, &mut o.st, o.s_aa, &hits, sounds);
            o.flags |= 0x20;
        }
    }
}

/// `RB_Create`'s +0xAA/+0x13c tail [M 0x1d6bf0:80-96]: 0x101 default (type 0 keeps it, and
/// +0x13c keeps the glist radius — the jump to LAB_001d6df0 skips the override); type 3 → 0
/// (keeping +0x13c); type 2/4 → 0 with +0x13c = min(Δx, Δz); any other non-zero type → 0
/// with +0x13c = sqrt(Δx² + Δz²)·0.5.
fn rb_create_tail(ty: i16, ext: [f32; 3]) -> (u16, Option<f32>) {
    match ty {
        0 => (0x101, None),
        3 => (0, None),
        2 | 4 => (0, Some(if ext[2] < ext[0] { ext[2] } else { ext[0] })),
        _ => (0, Some((ext[0] * ext[0] + ext[2] * ext[2]).sqrt() * 0.5)),
    }
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
