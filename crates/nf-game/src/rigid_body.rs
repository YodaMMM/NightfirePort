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
//! 1 = param 2 [D; L rb+0xa8]); owned when the RB evolution lands (docs/systems/levels.md §7.1). Coll_ResetHitHeap 0x1edda0 [M]
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

/// `FRAME_RATE_MUL` @0x30d0d8 = 1.0 (NTSC) [L ee_start_05.bin; the up-righting tail loads it
/// at 0x1d7a20-0x1d7a24 (lui 0x31 / lwc1 −0x2f28)].
const FRAME_RATE_MUL: f32 = 1.0;

/// The angular damping of `RB_EulerEvolve` [M disasm 0x1d7864-0x1d78dc]: avg = (I0+I1+I2)·0.333
/// (0x3eaa7efa); e = expf((−(avg/+0xa4)/avg)·dt); k = −((1 − e)/dt); then the VU pair
/// `VMULx.xyz vf3, vf2, vf1x` / `VADD vf2, vf2, vf3` (0x4bc110d8 / 0x4be310a8) — omega += omega·k,
/// i.e. omega *= 1 − (1 − e^(−dt/a))/dt. At dt = 1/60 and +0xa4 = 25 that is ×0.96 per tick
/// (the rate (1−e)/dt ≈ 1/a per tick, NOT the e^(−dt/a) ≈ ×0.99933 a plain exponential gives).
fn rb_damp_omega(rb: &RigidBody, omega: &mut [f32; 3], dt: f32) {
    let avg = (rb.moments[0] + rb.moments[1] + rb.moments[2]) * f32::from_bits(0x3eaa_7efa);
    let e = ((-(avg / rb.sa4) / avg) * dt).exp();
    let k = -((1.0 - e) / dt);
    for o in omega.iter_mut() {
        *o += *o * k;
    }
}

/// The type != 0 up-righting tail of `RB_EulerEvolve` [M disasm 0x1d79f4-0x1d7a58]: after the
/// matrix rebuild, up = `Mat_GetUp` (row 1, +0x10..+0x18 [M 0x1e33b8]); the torque slot +0x50
/// += (−up.z·(+0x84)·FRAME_RATE_MUL, 0, up.x·(+0x84)·FRAME_RATE_MUL) (VADD 0x4be110a8; the w
/// lane is stack junk, not ported — the slots are 3-wide) — the up × (0,1,0) spring that the
/// NEXT tick's `RB_ApplyImpulses` turns into omega, pulling the body back to its rest rotation.
/// The game then rebuilds the matrix again from the unchanged quat (0x1d7a54).
fn rb_upright_tail(rb: &RigidBody, st: &mut RbState) {
    if rb.ty == 0 {
        return;
    }
    let up = st.matrix[1];
    let i1 = rb.moments[1]; // +0x84 [M 0x1d7a14]
    st.tor_impulse[0] += -up[2] * i1 * FRAME_RATE_MUL;
    st.tor_impulse[1] += 0.0;
    st.tor_impulse[2] += up[0] * i1 * FRAME_RATE_MUL;
}

/// `RB_EulerEvolve 0x1d7810` [M disasm 0x1d7810-0x1d7a78]. flag != 0 skips the impulse
/// application (the game's second evolve entry); dt = REC_FRAME_RATE from `RB_Update`. dt == 0
/// branches straight to the matrix rebuild (0x1d785c → 0x1d79e8) — the up-righting tail still
/// runs on that path.
pub fn rb_euler_evolve(rb: &mut RigidBody, st: &mut RbState, flag: i32, dt: f32) {
    if flag == 0 {
        rb_apply_impulses(rb, st);
        st.lin_impulse = [0.0; 3];
        st.tor_impulse = [0.0; 3];
    }
    if dt == 0.0 {
        st.matrix = crate::player::quat_to_matrix(&rb.quat);
        rb_upright_tail(rb, st);
        return;
    }
    // Angular damping, gated on 0 < +0xa4 [M 0x1d7864-0x1d7870].
    if rb.sa4 > 0.0 {
        rb_damp_omega(rb, &mut st.omega, dt);
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
    rb_upright_tail(rb, st);
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

/// `RB_Update 0x1d6e38` [M disasm], the sim-reachable part: the buoyancy puff (cel +0x90 above
/// the body's obj+0xc4 y, gated on `GameState & 0xf != 0` — the level's low nibble, true
/// on every census level [M GameState == the level code, ssys.rs]), then wind, then the evolve.
///
/// The force block is BUOYANCY, not heat: cel+0x90 is the water surface (`build_alloc_cel`
/// 0x1c93f8 defaults it to −500.0 = 0xc3fa0000; `parseentity_fixup_entity` 0x1d00c0 sets
/// cel+0x90 = cel+0x54 for room class 0xC023 — the port's verified fix-ladder doc), and the
/// puff `(0, |obj_y − cel+0x54|·(+0x78 mass)·0.4 + Rand_FRand(0.15), 0)` grows with the depth
/// below that surface (cel+0x90 == cel+0x54 for a water cel), with the `Gas_Init` bubbles as
/// the effect layer [M 0x1d6e38:0x1d6e88-0x1d6eb0; the shared-RNG draw ONLY when the gate
/// passes]. It is inert for the hanging lamps: the linear impulse is consumed only by the
/// type-0 arm of `RB_ApplyImpulses` 0x1d6ff8.
///
/// The lamp mover is the wind: `RB_ApplyWind` 0x1d7c78 lands the force AT A POINT for
/// type != 0 (obj+0xc0 x, obj+0xc4 + rb+0x9c − rb+0x13c y, obj+0xc8 z) — torque, the only
/// per-frame torque a freely hanging Rot body gets.
pub fn rb_update(
    rb: &mut RigidBody,
    st: &mut RbState,
    buoy: Option<(f32, f32)>, // (cel+0x90 water surface, cel+0x54 glist bbox max y) when the cel is set
    obj_y: f32,
    level_low_nibble: u32,
    wind: [f32; 3],    // the caller's `Env_GetWindVel(cel, obj+0xc0)` [M 0x1d7c78]
    obj_pos: [f32; 3], // obj+0xc0 (the wind's application point base)
    frand: impl FnOnce(f32) -> f32,
) {
    if let Some((surface, top)) = buoy {
        if obj_y < surface && level_low_nibble != 0 {
            let f = (obj_y - top).abs() * rb.mass * 0.4 + frand(0.15);
            rb_add_impulsive_force(st, [0.0, f, 0.0]);
        }
    }
    rb_apply_wind(rb, st, wind, obj_pos);
    rb_euler_evolve(rb, st, 0, crate::ssys::REC_FRAME_RATE);
}

// ============================================================================
// The cel wind (`Env_Create` 0x17b968 / `Env_Update` 0x17bc88 / `Env_GetWindVel` 0x17c048).
// One wind struct W per room cel (`cel+0x2c`), written by placement kind 0xDF and animated
// once per frame. `RB_ApplyWind` 0x1d7c78 is the only rigid-body consumer [M decompile grep:
// `Env_GetWindVel` appears in RB_ApplyWind and Gas_Init's record only].
// ============================================================================

/// The game's wind struct W (the 0x50-byte `Env_Create` allocation) — the fields the RB path
/// reads, plus the cel-side wind origin. [M 0x17b968 (create), 0x17bc88 (update), 0x17c048 (read).]
#[derive(Clone, Debug, PartialEq)]
pub struct EnvWind {
    /// W+0x00: the wind direction — `RotMatrix(placement rotation)` 0x1e2368's dir row
    /// (`Mat_GetDir` 0x1e3398 = the +0x20 row): `(sin ry, −sin rx·cos ry, cos rx·cos ry)`,
    /// roll-independent, unit.
    pub dir: [f32; 3],
    /// cel+0x80: the cel's world sphere centre — the wind origin the falloff measures from
    /// [M Env_GetWindVel 0x17c048 `_vsub(pos, cel+0x80)`; parseentity_transform_bounding_box
    /// 0x1d0fb0, the port's verified CelNode.sphere centre].
    pub centre: [f32; 3],
    /// W+0x30: the gust strength (animated). W+0x34: the target (Env_Create sets both to
    /// `param3·0.1`; 0 = no wind, the whole cel skipped).
    pub strength: f32,
    pub target: f32,
    /// W+0x38: the gust phase, −1..1 (Env_Create starts it at −1 = 0xbf800000).
    pub phase: f32,
    /// W+0x44: `1/(pCelList record+0x1c radius)` — the cel's own glist sphere radius [M 0x17b968].
    pub falloff: f32,
    /// W+0x4d: the grow/decay latch, starts 1 [M 0x17b968 `pauVar8[4][0xd] = 1`].
    pub grow: u8,
}

/// `Env_Create` 0x17b968's wind half [M]: strength = target = `param3·0.1` (the u32 param read
/// `*(uint *)(level_tag+0x38)`, ×0.1); phase = −1.0; grow = 1; falloff = `1/(record+0x1c)`
/// (the caller resolves the cel's glist radius); dir = the placement rotation's dir row
/// [M RotMatrix 0x1e2368 + Mat_GetDir 0x1e3398]. The drop-side bytes (W+0x4b..0x4e from params
/// 0/1/2, the W+0x48..0x4a bytes, W+0x3c/W+0x40 floats from params 7/8) are the weather layer
/// the port's `env::Drops` owns — not carried here.
pub fn env_wind_create(params: &[(u32, u32)], rot: [f32; 3], falloff: f32) -> EnvWind {
    let pv = |k: u32| params.iter().find(|(i, _)| *i == k).map_or(0u32, |&(_, v)| v);
    let s = (pv(3) as f32) * 0.1;
    let (rx, ry) = (rot[0], rot[1]);
    // RotMatrix 0x1e2368's dir row: PS2Sinf3(rx+π/2, rx, ry+π/2) then the second triple; the
    // dir row = (sin ry, −sin rx·cos ry, cos rx·cos ry) [M param_2[8..10] stores]. The port's
    // bit-exact sinf (the game's PS2 sinf) and the same π/2 literal 1.5707964.
    let half = f32::from_bits(0x3fc9_0fdb);
    let sx = nf_formats::anim::sinf(rx);
    let cx = nf_formats::anim::sinf(rx + half);
    let sy = nf_formats::anim::sinf(ry);
    let cy = nf_formats::anim::sinf(ry + half);
    EnvWind {
        dir: [sy, -sx * cy, cx * cy],
        centre: [0.0; 3],
        strength: s,
        target: s,
        phase: -1.0,
        falloff,
        grow: 1,
    }
}

/// `Env_Update` 0x17bc88's per-cel gust step [M]: with `fVar4 = Rand_Rand(20000)·REC_FRAME_RATE
/// ·1.5384616e-05` (ONE draw per frame for all cels — the game draws it before the cel walk):
/// phase += fVar4·REC_FRAME_RATE, reset to −1.0 past 1.0; then the strength cycle — grow latch
/// set: `K += K + fVar4` (doubling) until K > target → latch 0; latch clear: `K *= 1 − fVar4`
/// until K < target·0.25 → latch 1. Target 0 cels are skipped by the walk [M the decompile's
/// `+0x34 == 0.0` branch].
pub fn env_update_gust(w: &mut EnvWind, gust: u32) {
    if w.target == 0.0 {
        return;
    }
    let f = gust as f32 * crate::ssys::REC_FRAME_RATE * 1.538_461_6e-5;
    w.phase += f * crate::ssys::REC_FRAME_RATE;
    if w.phase > 1.0 {
        w.phase = -1.0;
    }
    if w.grow == 0 {
        w.strength *= 1.0 - f;
        if w.strength < w.target * 0.25 {
            w.grow = 1;
        }
    } else {
        w.strength += w.strength + f;
        if w.strength > w.target {
            w.grow = 0;
        }
    }
}

/// `Env_GetWindVel` 0x17c048 [M]: zero when the cel carries no wind or the target is 0
/// (`Vec_Zero`); else `out = dir · (K · (dot(pos − cel+0x80, dir)·falloff + 1) · |phase| · 0.5)`
/// — the dot over the unit dir, the falloff from the cel's own sphere radius, no clamp (the
/// decompile has none; the term can go negative behind the origin).
pub fn env_get_wind_vel(w: &EnvWind, pos: [f32; 3]) -> [f32; 3] {
    if w.target == 0.0 {
        return [0.0; 3];
    }
    let d = [pos[0] - w.centre[0], pos[1] - w.centre[1], pos[2] - w.centre[2]];
    let dot = d[0] * w.dir[0] + d[1] * w.dir[1] + d[2] * w.dir[2];
    let s = w.strength * (dot * w.falloff + 1.0) * w.phase.abs() * 0.5;
    [w.dir[0] * s, w.dir[1] * s, w.dir[2] * s]
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
/// barrels and lamps (type 1, param 2 [D; L rb+0xa8 = 1]) take. Per hit, class 5 only [M 0x1d7428-0x1d74c8]: Sound_Play3D
/// (collected, not played), impulse = (hit+0x8)·0.5·vel, +0x50 += r×impulse (no +0x40 write)
/// with the PER-TYPE masking of the cross lanes: type 2 keeps x, type 3 keeps y, type 4 keeps
/// z, type 1 (the census lamps and barrels) zeroes y; every other type adds the full cross. Other classes fall through — Rot has NO position-correction arm
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
            // The per-type lane masks on the cross at sp+0x20/+0x24/+0x28 [M disasm
            // 0x1d753c-0x1d757c]: type 2 zeroes y,z; type 3 zeroes z,x; type 4 zeroes y,x;
            // type 1 zeroes y (the census lamps/barrels — live lamp #40's +0x50 after the arm
            // carried y = 0 [L]); every other type keeps the full cross.
            let masked = match rb.ty {
                2 => [cross[0], 0.0, 0.0],
                3 => [0.0, cross[1], 0.0],
                4 => [0.0, 0.0, cross[2]],
                1 => [cross[0], 0.0, cross[2]],
                _ => cross,
            };
            // Only the torque slot is written (VADD into +0x50 at 0x1d7580-0x1d758c); Rot never
            // touches the +0x40 linear accumulator [M disasm 0x1d74dc-0x1d758c].
            for i in 0..3 {
                st.tor_impulse[i] += masked[i];
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
    /// The glist bounding sphere in object space (entity params +0x08 centre, +0x14 radius):
    /// `Control_BuildWorldSph` 0x133070 poses the centre by the live obj+0x90 into obj+0x80 and
    /// keeps the radius at obj+0x8C — the sphere `Collide_PickObj` 0x1eae80 tests [M].
    pub sph_centre: [f32; 3],
    pub sph_radius: f32,
    /// obj+0x20: the room cel — `build_LinkToRoom` 0x1c9680 at create. The port resolves the
    /// room cel whose glist bbox holds the placement translation once at load and keeps it
    /// (the lamps' cel never changes: a Rot body never translates [M EulerEvolve's type-0
    /// position arm]; a walking-body re-eval is the unported remainder) [S the static pick].
    /// `wind`: the cel's `Env_Create` wind (Rbodies::winds index) — None = no wind struct.
    pub wind: Option<usize>,
    /// The cel's buoyancy fields: (cel+0x90 water surface, cel+0x54 glist bbox max y) — None =
    /// no cel resolved [M parseentity_fixup_entity 0x1d00c0 / parsemap_block_map_data_static
    /// 0x1d1998 via the port's verified fix-ladder doc].
    pub buoy: Option<(f32, f32)>,
}

impl RbObj {
    /// obj+0x80 / obj+0x8C: the live world sphere — the glist centre through the body's current
    /// rotation rows plus the translation row (row-vector v·M), the glist radius [M
    /// Control_BuildWorldSph 0x133070].
    pub fn world_sphere(&self) -> ([f32; 3], f32) {
        let (c, m, t) = (self.sph_centre, &self.st.matrix, self.st.trans);
        let w = [
            t[0] + c[0] * m[0][0] + c[1] * m[1][0] + c[2] * m[2][0],
            t[1] + c[0] * m[0][1] + c[1] * m[1][1] + c[2] * m[2][1],
            t[2] + c[0] * m[0][2] + c[1] * m[1][2] + c[2] * m[2][2],
        ];
        (w, self.sph_radius)
    }

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
/// other creators follow), plus the room cels' wind structs (`Env_Create` 0x17b968 — placement
/// kind 0xDF, one per cel, animated once per frame).
#[derive(Default)]
pub struct Rbodies {
    pub list: Vec<RbObj>,
    /// The cels' wind structs, in 0xDF placement order; `RbObj::wind` indexes here.
    pub winds: Vec<EnvWind>,
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
        let Some(m) = level.main_map() else { return Rbodies { list, winds: Vec::new() } };
        // The room cels + their wind: the main map's cel placements (the parse_cel_graph room
        // half — flags16 0xc023/0xc047, the object's raw glist bbox = the cel's +0x40..0x58 [M
        // parsemap_block_map_data_static 0x1d1998; build_FindCel 0x1c8cb0 tests this box]), the
        // water surface (cel+0x90: the bbox max y for 0xC023, −500.0 otherwise [M
        // parseentity_fixup_entity 0x1d00c0, the port's verified fix-ladder doc]) and the world
        // sphere centre (cel+0x80: the bbox corners through the placement matrix, then the AABB
        // centre [M parseentity_transform_bounding_box 0x1d0fb0]). The 0xDF placements build the
        // wind structs in placement order; a cel already carrying one is skipped (Env_Create's
        // `cel+0x2c != 0` early-out [M 0x17b968]).
        struct WindCel {
            min: [f32; 3],
            max: [f32; 3],
            water: f32,
            wind: Option<usize>,
        }
        let mut cels: Vec<WindCel> = Vec::new();
        let mut winds: Vec<EnvWind> = Vec::new();
        for p in &level.maps[m].placements {
            if !p.creates_cel() { continue; }
            let flags16 = (p.flags & 0xffff) as u16;
            if !matches!(flags16, 0xc023 | 0xc047) { continue; }
            let resolved = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() } else { Some((m, p.index as usize)) };
            let Some((map, object)) = resolved else { continue };
            let Some(o) = level.maps[map].objects.get(object) else { continue };
            cels.push(WindCel { min: o.min, max: o.max, water: if flags16 == 0xc023 { o.max[1] } else { -500.0 }, wind: None });
        }
        for p in level.maps[m].placements.iter().filter(|p| p.flags & 0xff == 0xdf) {
            let hit = cels.iter_mut().position(|c| c.min[0] <= p.translation[0] && p.translation[0] <= c.max[0]
                && c.min[1] <= p.translation[1] && p.translation[1] <= c.max[1]
                && c.min[2] <= p.translation[2] && p.translation[2] <= c.max[2]);
            let Some(ci) = hit else { continue };
            if cels[ci].wind.is_some() { continue; }
            // The falloff = 1/(pCelList record+0x1c) — the WIND CEL's glist sphere radius [M
            // 0x17b968 reads `*(cel+0x38) + 0x1c` — cel+0x38 is the record the CEL placement
            // bound at 0x1d1998, not the 0xDF's own object]. The record's sphere is the cel
            // object's glist sphere (the port's Object::radius — the same source RB_Create's
            // +0x13c copies [M Control_BuildWorldSph 0x133070]); the wind origin (cel+0x80)
            // is the placement matrix through the cel bbox (the AABB centre, CelNode.sphere's
            // rule). The cel object re-resolves: the cel the 0xDF lands in is the `ci`-th
            // room cel — its placement is the matching cel placement.
            let mut cel_obj = None;
            let mut k = 0usize;
            for q in &level.maps[m].placements {
                if !q.creates_cel() { continue; }
                let flags16 = (q.flags & 0xffff) as u16;
                if !matches!(flags16, 0xc023 | 0xc047) { continue; }
                let resolved = if q.by_hash() { level.by_hash.get(&q.hashcode).copied() } else { Some((m, q.index as usize)) };
                if let Some((map, object)) = resolved {
                    if let Some(o) = level.maps[map].objects.get(object) {
                        if k == ci { cel_obj = Some((q.rotation, q.translation, o.min, o.max, o.radius)); break; }
                        k += 1;
                    }
                }
            }
            let Some((rot, ctrans, cmin, cmax, cradius)) = cel_obj else { continue };
            if !(cradius > 0.0) { continue; }
            let mtx = nf_formats::placement::object_matrix(rot, ctrans);
            let mut lo = [f32::MAX; 3];
            let mut hi = [f32::MIN; 3];
            for cx in [cmin[0], cmax[0]] { for cy in [cmin[1], cmax[1]] { for cz in [cmin[2], cmax[2]] {
                let w = nf_formats::placement::transform(&mtx, [cx, cy, cz]);
                for a in 0..3 { lo[a] = lo[a].min(w[a]); hi[a] = hi[a].max(w[a]); }
            } } }
            let centre = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5, (lo[2] + hi[2]) * 0.5];
            let mut w = crate::rigid_body::env_wind_create(&p.params, p.rotation, 1.0 / cradius);
            w.centre = centre;
            cels[ci].wind = Some(winds.len());
            winds.push(w);
        }
        for (pi, p) in level.maps[m].placements.iter().enumerate() {
            if p.flags & 0xff != 0xd8 { continue; }
            // The placement → object resolve, the doors' `object(p)` convention: by-hash through
            // the level's table, else the placement's own index.
            let resolved = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() } else { Some((m, p.index as usize)) };
            let Some((map, object)) = resolved else { continue };
            let Some(o) = level.maps[map].objects.get(object) else { continue };
            // The record words: `RB_Create` reads the mass at level_tag+0x2c and the type
            // halfword at level_tag+0x34 [M 0x1d6bf0:52-56]. `level_tag` is the 0xB0 block
            // `parsemap_block_map_data_dynamic` memsets per placement and fills with each
            // (key, value) pair at +0x2C + 4·key [M 0x1d1b68:64-75] — param k sits at +0x2C+4k:
            // p0 = the mass word, p2 = the type halfword, p3 = the +0x154 byte, p4 = the +0x158
            // word [L a break at RB_Create 0x1d6bf0: 07000004's lamps #39/#40 arrive with
            // +0x2c = 1, +0x34 = 1, +0x38 = 1, +0x3c = 0xFFFF (rb+0xa8 = 1, rb+0x78 mass = 1.0
            // read back after the create); 07000006's barrel with +0x2c = 8, +0x34 = 1, +0x38 = 0,
            // +0x3c = 0xFFFF].
            let pv = |k: u32| p.params.iter().find(|(j, _)| *j == k).map(|&(_, v)| v).unwrap_or(0);
            let ty = (pv(2) & 0xffff) as u16 as i16;
            let mass_word = pv(0);
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
            // obj+0x20: the room cel holding the placement translation (build_LinkToRoom
            // 0x1c9680's create-time pick; build_FindCel 0x1c8cb0's bbox test over the room
            // cels' raw glist bbox [M 0x1c8cb0: the +0x40..0x58 test]). Multiple matches: the
            // game's collision-ray tie-break is not ported — the first parse-order room cel
            // wins [S unverified tie-break; 07000004/06/01 resolve uniquely (the wind probe)].
            let hit = cels.iter().position(|c| c.min[0] <= st.pos[0] && st.pos[0] <= c.max[0]
                && c.min[1] <= st.pos[1] && st.pos[1] <= c.max[1]
                && c.min[2] <= st.pos[2] && st.pos[2] <= c.max[2]);
            let (wind, buoy) = hit.map(|ci| (cels[ci].wind, Some((cels[ci].water, cels[ci].max[1])))).unwrap_or((None, None));
            list.push(RbObj {
                placement: pi as u16,
                glist: o.hashcode,
                flags: 0x44,
                s_aa,
                s154: (pv(3) & 0xff) as u8,
                s158: if pv(4) == 0 { 0xffff } else { pv(4) },
                rb,
                st,
                hits: Vec::new(),
                sph_centre: o.centre,
                sph_radius: o.radius,
                wind,
                buoy,
            });
        }
        Rbodies { list, winds }
    }

    /// `Env_Update` 0x17bc88's cel walk [M]: every wind cel steps once per frame from the ONE
    /// `Rand_Rand(20000)` the frame drew (no extra draws). Called at the env slot of the tick
    /// (the game's `Game_Run` order: `Env_Update` before the object handler).
    pub fn tick_winds(&mut self, gust: u32) {
        for w in &mut self.winds {
            crate::rigid_body::env_update_gust(w, gust);
        }
    }

    /// `RB_Update` 0x1d6e38 [M] per body: the buoyancy gate (the body's own cel's water
    /// surface, `None` when the cel did not resolve), the cel wind (`Env_GetWindVel` 0x17c048
    /// — zero without a wind cel), the evolve, and the flags bookkeeping the sim reads
    /// (obj+0xFA |= 0x20 [M :tail]; the +0x100 light-position sync is the visual layer —
    /// not ported).
    pub fn tick(&mut self, level_low_nibble: u32, rng: &mut crate::drone::Rng) {
        for j in 0..self.list.len() {
            self.tick_one(j, level_low_nibble, rng);
        }
    }

    /// One body's `RB_Update` — the control walk's per-object call: each kind-0xD8 object runs
    /// its own update once per frame (the dispatch pair at 0x26f9c8 is per object [M]).
    pub fn tick_one(&mut self, j: usize, level_low_nibble: u32, rng: &mut crate::drone::Rng) {
        let Some(o) = self.list.get_mut(j) else { return };
        // Env_GetWindVel(cel, obj+0xc0) [M 0x1d7c78] — obj+0xc0 is the object's world position;
        // the port's stand-in is the drawn translation (identical for the Rot census bodies,
        // which never translate [M EulerEvolve]) [S].
        let wind = match o.wind {
            Some(wi) => crate::rigid_body::env_get_wind_vel(&self.winds[wi], o.st.trans),
            None => [0.0; 3],
        };
        let obj_y = o.st.trans[1];
        let buoy = o.buoy;
        let obj_pos = o.st.trans;
        crate::rigid_body::rb_update(&mut o.rb, &mut o.st, buoy, obj_y, level_low_nibble, wind, obj_pos, |m| rng.frand(m));
        o.flags |= 0x20;
    }

    /// `RB_CollisionHandler` 0x1d7a80 [M] per body over its hit list; the sounds collect for
    /// the caller's pool.
    pub fn handle_collisions(&mut self, sounds: &mut Vec<(u32, [f32; 3])>) {
        for j in 0..self.list.len() {
            self.handle_collisions_one(j, sounds);
        }
    }

    /// One body's `RB_CollisionHandler` over its own hit list (the control walk's per-object call).
    pub fn handle_collisions_one(&mut self, j: usize, sounds: &mut Vec<(u32, [f32; 3])>) {
        let Some(o) = self.list.get_mut(j) else { return };
        if o.hits.is_empty() { return; }
        let hits = std::mem::take(&mut o.hits);
        rb_collision_handler(&o.rb, &mut o.st, o.s_aa, &hits, sounds);
        o.flags |= 0x20;
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

    /// A sphere-type body with an unmasked type halfword 0xFFFF (−1: none of the Rot arm's lane
    /// masks applies — the census bodies themselves are type 1, param 2 [D; L]), mass 3 (the RB_Create default when
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
            rb_update(&mut rb, &mut st, None, 1.0, 1, [0.0; 3], [0.0; 3], |_| panic!("no rng draw at rest"));
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
        // z scales by 1/I1 (+0x94) [M 0x1d705c-0x1d7068]: 10·(1/5) = 2.0, then the +0xa4=25
        // damping omega += omega·(−(1−e)/dt) [M 0x1d78b0-0x1d78dc].
        let avg = (rb.moments[0] + rb.moments[1] + rb.moments[2]) * f32::from_bits(0x3eaa_7efa);
        let e = ((-(avg / 25.0) / avg) * dt).exp();
        let w0 = 10.0 * rb.inv_moments[1];
        let expect_z = w0 + w0 * -((1.0 - e) / dt);
        assert!((st.omega[2] - expect_z).abs() < 1.0e-6, "omega.z {} vs {}", st.omega[2], expect_z);
        assert_eq!(st.omega[0], 0.0);
        assert_eq!(st.omega[1], 0.0);
        assert_eq!(st.pos, p0, "type != 0 has no linear impulse arm [M 0x1d7124: the ty==0 gate]");
        assert_eq!(st.trans, t0);
        // The quaternion rotated about z by the half-angle form; the matrix follows.
        assert!((rb.quat[1]).abs() < 1.0e-6 && (rb.quat[2]).abs() < 1.0e-6, "pure z rotation {:?}", rb.quat);
        assert!(rb.quat[0] > 0.99 && rb.quat[3] > 0.0, "w≈cos(half), a small positive z turn");
        assert_eq!(st.matrix[2][0], 0.0);
        assert!((st.matrix[2][2] - 1.0).abs() < 1.0e-6, "z rotation leaves the z row [M quat_to_matrix]");
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

    /// [M 0x1d6e38:0x1d6e88-0x1d6eb0] The buoyancy puff: gate = the cel set, obj+0xc4 below
    /// cel+0x90 (the water surface — build_alloc_cel 0x1c93f8 defaults −500.0;
    /// parseentity_fixup_entity 0x1d00c0 sets it to cel+0x54 for room class 0xC023), and
    /// GameState&0xf != 0; force = (0, |obj_y − cel+0x54|·mass·0.4 + frand(0.15), 0) — the
    /// depth-proportional lift with the Gas_Init bubbles; the shared-RNG draw happens ONLY
    /// when the gate passes.
    #[test]
    fn buoyancy_puff_gate() {
        // The puff's impulse lands in +0x40 and the same tick's evolve applies it to the
        // position for type 0 (for type != 0 the game's ApplyImpulses drops it — the smoke is
        // the visible effect). Use a type-0 body and assert the displacement.
        let mut rb = rb_init(0);
        assert!(rb_build(&mut rb, 3.0, BOX_VERTS, rb_cuboid_moments_of_inertia(3.0, 2.0, 3.0, 4.0)));
        let mut st = RbState::zeroed();
        st.matrix = rb_set_state(&rb);
        rb_update(&mut rb, &mut st, Some((5.0, 1.0)), 2.0, 1, [0.0; 3], [0.0; 3], |lim| lim * 0.5);
        // The impulse arm lifts y by (1.275/3); then vel = f·(pos − WldGravity·1) integrates
        // the same tick [M 0x1d7124 + 0x1d78ec-0x1d7944 + 0x1d796c-0x1d7994].
        let dt = crate::ssys::REC_FRAME_RATE;
        let lift = (1.0f32 * 3.0 * 0.4 + 0.075) / 3.0;
        let f = ((-(dt / 1.0f32)).exp() - 1.0) / dt;
        let want = lift + dt * f * (lift - crate::grenade::WLD_GRAVITY[1]);
        assert!((st.pos[1] - want).abs() < 1.0e-5, "pos.y {} vs {}", st.pos[1], want);
        // The closed gate draws nothing (the closure would panic).
        let (_, mut st2) = barrel();
        rb_update(&mut rb, &mut st2, Some((5.0, 1.0)), 2.0, 0, [0.0; 3], [0.0; 3], |_| panic!("no draw on a closed gate"));
        assert_eq!(st2.lin_impulse, [0.0; 3]);
        // Nibble non-zero but the body above the heat: no draw, no force.
        let (_, mut st3) = barrel();
        rb_update(&mut rb, &mut st3, Some((5.0, 1.0)), 6.0, 1, [0.0; 3], [0.0; 3], |_| panic!("no draw above the heat"));
        assert_eq!(st3.lin_impulse, [0.0; 3]);
    }

    /// [M 0x17b968 + 0x1e2368 + 0x1e3398] Env_Create's wind half: strength = target = p3·0.1,
    /// phase −1, grow 1, and the dir row of RotMatrix(rotation) — for a zero rotation the
    /// dir row is (0, 0, 1) (sin 0 = 0, cos 0 = 1).
    #[test]
    fn env_wind_create_parses_the_placement() {
        let w = env_wind_create(&[(0, 1), (1, 1), (2, 4), (3, 6)], [0.0; 3], 0.033);
        assert_eq!(w.strength, 0.6);
        assert_eq!(w.target, 0.6, "Env_Create stores the SAME p3·0.1 into +0x30 and +0x34 [M 0x17b968]");
        assert_eq!(w.phase, -1.0, "the phase literal 0xbf800000 [M 0x17b968]");
        assert_eq!(w.grow, 1);
        assert_eq!(w.falloff, 0.033);
        // The PS2 sinf's cos(0) lands at 1.0000002 — the game's RotMatrix reads the same
        // value from PS2Sinf3, so the dir row is NOT an exact unit vector [M 0x1e2368 + the
        // port's bit-exact sinf].
        assert_eq!(w.dir[0], 0.0);
        assert_eq!(w.dir[1], 0.0);
        assert!((w.dir[2] - 1.0).abs() < 1.0e-5, "the zero rotation's dir row [M RotMatrix 0x1e2368], got {:?}", w.dir);
        // Sister (can fail): a yaw-only rotation swings the dir row's x (sin ry).
        let wy = env_wind_create(&[], [0.0, 1.5707964, 0.0], 1.0);
        assert!(wy.dir[0] > 0.99, "dir.x = sin(ry) at yaw π/2, got {:?}", wy.dir);
        assert!(wy.dir[2].abs() < 1.0e-3);
    }

    /// [M 0x17bc88] The gust cycle: the phase sweeps up (rand-scaled), resets to −1 past 1.0;
    /// the strength doubles while growing (K = 2K + f) past the target into the decay
    /// (K ×= 1 − f) until 0.25×target re-arms the grow. The zero-target wind never moves.
    #[test]
    fn gust_cycle_grows_and_decays() {
        let mut w = env_wind_create(&[(3, 6)], [0.0; 3], 1.0);
        // The grow arm: from 0.6 one doubling lands above the target → latch 0.
        env_update_gust(&mut w, 20000);
        assert!(w.strength > 0.6, "2K + f past the target, got {}", w.strength);
        assert_eq!(w.grow, 0, "K > target latches decay [M 0x17bc88]");
        // The decay arm: many small steps pull K below 0.25×target → latch 1 again.
        let mut steps = 0;
        while w.grow == 0 && steps < 20000 {
            env_update_gust(&mut w, 20000);
            steps += 1;
        }
        assert_eq!(w.grow, 1, "decay re-arms the grow below 0.25×target after {} steps", steps);
        assert!(w.strength < 0.15 + 1.0e-3);
        // The phase: f·REC_FRAME_RATE per step; the reset lands at −1.0 exactly.
        let mut w2 = env_wind_create(&[(3, 6)], [0.0; 3], 1.0);
        let f = 20000.0f32 * crate::ssys::REC_FRAME_RATE * 1.538_461_6e-5;
        let per = f * crate::ssys::REC_FRAME_RATE;
        let steps = ((2.0 / per) as u32) + 1;
        for _ in 0..steps { env_update_gust(&mut w2, 20000); }
        assert_eq!(w2.phase, -1.0, "the phase reset [M 0x17bc88's 0xbf800000 store]");
        // Sister (can fail): the zero-target wind is skipped by the walk entirely.
        let mut w0 = EnvWind { dir: [0.0, 0.0, 1.0], centre: [0.0; 3], strength: 0.0, target: 0.0, phase: -1.0, falloff: 1.0, grow: 1 };
        env_update_gust(&mut w0, 20000);
        assert_eq!(w0.strength, 0.0);
        assert_eq!(w0.phase, -1.0);
    }

    /// [M 0x17c048] Env_GetWindVel: zero without a target; else dir · (K·(dot·falloff + 1)
    /// ·|phase|·0.5) with the dot over (pos − cel+0x80).
    #[test]
    fn wind_velocity_formula() {
        let mut w = env_wind_create(&[(3, 6)], [0.0; 3], 0.1);
        w.centre = [10.0, 0.0, 0.0];
        w.phase = 1.0;
        // pos 10 ahead of the centre along +z: dot = 5 → K·(0.5 + 1)·1·0.5 along +z.
        let v = env_get_wind_vel(&w, [10.0, 0.0, 5.0]);
        let want = 0.6 * (5.0 * 0.1 + 1.0) * 0.5;
        assert!((v[2] - want).abs() < 1.0e-6, "wind.z {} vs {}", v[2], want);
        assert_eq!(v[0], 0.0);
        assert_eq!(v[1], 0.0);
        // Sister (can fail): behind the origin past the falloff the factor goes negative —
        // the game does not clamp [M 0x17c048 has no max].
        let vb = env_get_wind_vel(&w, [10.0, 0.0, -15.0]);
        assert!(vb[2] < 0.0, "dot = −1.5 → the bracket −0.15+1 stays positive; at −25 it flips");
        let vb2 = env_get_wind_vel(&w, [10.0, 0.0, -25.0]);
        assert!(vb2[2] < 0.0, "dot·falloff = −2.5 → the bracket is negative [M no clamp]");
        // The zero-target cel reads zero.
        w.target = 0.0;
        assert_eq!(env_get_wind_vel(&w, [10.0, 0.0, 5.0]), [0.0; 3]);
    }

    /// [M 0x1d6e38 order] The buoyancy puff, then RB_ApplyWind, then the evolve — the wind
    /// torque reaches a type-1 body through rb_update's wind argument (the lamp path).
    #[test]
    fn update_applies_the_wind_before_the_evolve() {
        let (mut rb, mut st) = barrel();
        rb.ty = 1;
        st.s13c = 1.0; // the wind point y = obj_y + 0 − 1 = 0 → r = (0, −1, 0) [M 0x1d7c78's point]
        rb_update(&mut rb, &mut st, None, 1.0, 1, [3.0, 0.0, 0.0], [10.0, 1.0, 20.0], |_| panic!("no rng draw without buoyancy"));
        // The at-point arm put r×f into +0x50 and the evolve consumed it (the accumulator
        // drains [M EulerEvolve flag 0]); the up-righting tail then leaves the NEXT tick's
        // spring torque in +0x50 [M 0x1d79f4-0x1d7a58] — tiny after one tick.
        assert_eq!(st.lin_impulse, [0.0; 3], "the evolve drains +0x40");
        assert!(st.tor_impulse[0].abs() < 1.0e-3 && st.tor_impulse[1] == 0.0 && st.tor_impulse[2].abs() < 1.0e-3,
            "only the spring's residue stays, {:?}", st.tor_impulse);
        assert!(st.omega[0] != 0.0 || st.omega[2] != 0.0, "a horizontal wind at a point offset tips the body");
        assert!(rb.quat != [1.0, 0.0, 0.0, 0.0], "the lamp's quat left rest");
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

    /// [M disasm 0x1d7864-0x1d78dc] The angular damping is omega += omega·(−(1 − e^(−dt/a))/dt):
    /// ×0.96 per tick at +0xa4 = 25 (any type other than 0 and 3). Sister (can fail): the plain e^(−dt/a)
    /// form (×0.99933 per tick) is far weaker.
    #[test]
    fn angular_damping_is_the_games_rate_form() {
        let (mut rb, mut st) = barrel();
        st.omega = [1.0, 0.0, 0.0];
        rb_euler_evolve(&mut rb, &mut st, 1, crate::ssys::REC_FRAME_RATE);
        let f = st.omega[0];
        assert!((f - 0.96).abs() < 2.0e-4, "per-tick factor {f} (the game's 1 − (1−e)/dt ≈ 0.96)");
        assert!(f < (-(crate::ssys::REC_FRAME_RATE / 25.0f32)).exp() - 0.03, "sister: not the plain exponential");
    }

    /// [M disasm 0x1d79f4-0x1d7a58] The type != 0 tail: a body tilted about x by θ gets
    /// +0x50 += (−up.z·I1, 0, up.x·I1) — the restoring torque; type 0 gets none.
    #[test]
    fn upright_tail_restores_a_tilted_body() {
        let (mut rb, mut st) = barrel();
        let th = 0.2f32;
        rb.quat = [(th * 0.5).cos(), (th * 0.5).sin(), 0.0, 0.0];
        rb_euler_evolve(&mut rb, &mut st, 1, crate::ssys::REC_FRAME_RATE);
        let up = st.matrix[1];
        assert!((up[2] - th.sin()).abs() < 1.0e-5, "row 1 = Mat_GetUp tilts toward +z, {up:?}");
        let i1 = rb.moments[1];
        assert!((st.tor_impulse[0] + th.sin() * i1).abs() < 1.0e-5, "x torque −up.z·I1, got {:?}", st.tor_impulse);
        assert_eq!(st.tor_impulse[1], 0.0);
        assert!(st.tor_impulse[2].abs() < 1.0e-6);
        // The next evolve turns it into omega opposing the tilt (θ shrinks).
        rb_euler_evolve(&mut rb, &mut st, 0, crate::ssys::REC_FRAME_RATE);
        assert!(st.omega[0] < 0.0, "the spring drives the tilt back, omega {:?}", st.omega);
        // Sister (can fail): a type-0 body never gets the tail.
        let mut r0 = rb_init(0);
        assert!(rb_build(&mut r0, 3.0, BOX_VERTS, rb_cuboid_moments_of_inertia(3.0, 2.0, 3.0, 4.0)));
        r0.quat = rb.quat;
        let mut s0 = RbState::zeroed();
        rb_euler_evolve(&mut r0, &mut s0, 1, 0.0);
        assert_eq!(s0.tor_impulse, [0.0; 3], "type 0 has no up-righting tail [M 0x1d79f8 gate]");
    }

    /// [M disasm 0x1d753c-0x1d757c, 0x1d7580] The Rot class-5 arm's lane masks per type, and no
    /// +0x40 write.
    #[test]
    fn rot_arm_lane_masks() {
        let hit = RbHit { scale: 2.0, normal: [0.0; 3], vel: [0.0, 1.0, 1.0], point: [11.0, 2.0, 20.0],
            other_class: 5, other_pos: [0.0; 3] };
        // r = (1, 1, 0), imp = (0, 1, 1) → r×imp = (1, −1, 1).
        let full = [1.0f32, -1.0, 1.0];
        for (ty, want) in [(-1i16, full), (1, [1.0, 0.0, 1.0]), (2, [1.0, 0.0, 0.0]), (3, [0.0, -1.0, 0.0]), (4, [0.0, 0.0, 1.0])] {
            let (mut rb, mut st) = barrel();
            rb.ty = ty;
            let mut snd = Vec::new();
            rb_resting_contact_rot(&rb, &mut st, std::slice::from_ref(&hit), &mut snd);
            assert_eq!(st.tor_impulse, want, "type {ty}");
            assert_eq!(st.lin_impulse, [0.0; 3], "Rot never writes +0x40 (type {ty})");
            assert_eq!(snd.len(), 1);
        }
    }

    /// [M 0x1d8038 disasm] out = f·(a − b·c), three components.
    #[test]
    fn vec_special_formula() {
        let out = rb_vec_special([1.0, 2.0, 3.0], [0.5, 0.5, 0.5], -2.0, 4.0);
        assert_eq!(out, [-2.0 * (1.0 - 2.0), -2.0 * (2.0 - 2.0), -2.0 * (3.0 - 2.0)]);
    }
}
