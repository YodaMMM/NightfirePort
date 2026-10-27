//! Map static data (block 0x1A): object placements. Spec: docs/formats/placement.md.
//!
//! Record (*measured*: `parsemap_block_map_data_static` 0x1d1998, all 23,329 records on the disc):
//! ```text
//! +0x00 u16 object index in this map (used when hashcode = 0xFFFFFFFF)
//! +0x04 u32 hashcode (else: an entity hashcode resolved through the hash table, i.e. another map's object)
//! +0x08 u32 flags   (0x8000: create a world cel now; others are created by the dynamic-object code)
//! +0x0C f32[3] translation → cel+0x60       +0x18 f32[3] rotation angles → cel+0x70 (radians, `RotMatrix`)
//! +0x24..+0x47 not read by this function
//! +0x48 u32 n, then n × { u32 index, u32 value } parameters (passed to `parseentity_fixup_entity`)
//! size = 0x4C + 8n
//! ```

use crate::buf::{invalid, Buf, Result};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Placement {
    pub index: u16,
    pub hashcode: u32,
    pub flags: u32,
    pub translation: [f32; 3],
    pub rotation: [f32; 3],
    pub params: Vec<(u32, u32)>,
    /// The words at +0x24..+0x47, raw (`parsemap_create_dynamic_objects` passes +0x34..+0x3C to some creators as a
    /// scale vector, e.g. `Drone_AIVolume_Create`).
    pub extra: [u32; 9],
}

impl Placement {
    pub const CREATE_CEL: u32 = 0x8000;
    pub fn by_hash(&self) -> bool { self.hashcode != u32::MAX }
    pub fn creates_cel(&self) -> bool { self.flags & Self::CREATE_CEL != 0 }
    /// +0x34..+0x3C as floats.
    pub fn scale(&self) -> [f32; 3] { [4, 5, 6].map(|k| f32::from_bits(self.extra[k])) }
    /// The +0x24..+0x34 quaternion (x, y, z, w) — `parsemap_create_dynamic_objects` 0x1d04f0
    /// copies these four words into the `quaternion_tag` it hands the creators that take one
    /// (0x27 `GT_Create`, 0x34 `GunImp_Create`, 0xf2's inline `Quat_QuatToMat` 0x1e3e80) [M].
    pub fn quaternion(&self) -> [f32; 4] { [0, 1, 2, 3].map(|k| f32::from_bits(self.extra[k])) }
}

/// Parses a 0x1A payload; it must end exactly at the last record (true for all 2,230 maps).
pub fn parse(payload: &[u8]) -> Result<Vec<Placement>> {
    let b = Buf(payload);
    let mut out = Vec::new();
    let mut p = 0usize;
    while p < payload.len() {
        let n = b.u32(p + 0x48)? as usize;
        if n > 1024 { return invalid(format!("placement at {p:#x}: {n} parameters")); }
        let f = |o: usize| b.f32(p + o);
        let mut params = Vec::with_capacity(n);
        for k in 0..n { params.push((b.u32(p + 0x4c + 8 * k)?, b.u32(p + 0x50 + 8 * k)?)); }
        out.push(Placement {
            index: b.u16(p)?, hashcode: b.u32(p + 4)?, flags: b.u32(p + 8)?,
            translation: [f(0x0c)?, f(0x10)?, f(0x14)?], rotation: [f(0x18)?, f(0x1c)?, f(0x20)?], params,
            extra: { let mut e = [0u32; 9]; for (k, w) in e.iter_mut().enumerate() { *w = b.u32(p + 0x24 + 4 * k)?; } e },
        });
        p += 0x4c + 8 * n;
    }
    if p != payload.len() { return invalid("placement records overrun the block"); }
    Ok(out)
}

/// `RotMatrix` (ACTION.ELF 0x1e2368) followed by `Mat_SetTrans`: a row-vector matrix (`v' = v·M`, translation in
/// row 3), entries exactly as the decomp computes them: two `PS2Sinf3` calls (`(rx+π/2, rx, ry+π/2)` then
/// `(rz+π/2, rz, ry)`) with cosines as sin(angle + π/2), then the nine products/sums in the decompile's order on
/// the EE FPU. G-017 closed: the sines are the game's own `PS2Sinf` (`anim::sinf`, bit-exact on the recorded
/// calls and the interpreter samples); the escort divergence this swap used to flip has another root
/// cause — the game's own math is this path.
pub fn object_matrix(rot: [f32; 3], trans: [f32; 3]) -> [[f32; 4]; 4] {
    use crate::anim::sinf;
    use crate::ps2float as pf;
    let fadd = |a, b| pf::add_noguard(a, b, true);
    let fsub = |a: f32, b: f32| pf::add_noguard(a, -b, true);
    let (rx, ry, rz) = (rot[0], rot[1], rot[2]);
    let half = f32::from_bits(0x3fc9_0fdb);
    // PS2Sinf3(rx + π/2, rx, ry + π/2) → (cx, sx, cy); PS2Sinf3(rz + π/2, rz, ry) → (cz, sz, sy).
    let (cx, sx, cy) = (sinf(fadd(rx, half)), sinf(rx), sinf(fadd(ry, half)));
    let (cz, sz, sy) = (sinf(fadd(rz, half)), sinf(rz), sinf(ry));
    let m = pf::mul;
    let n = |v: f32| -v; // exact sign flip; the decompile folds the negations into the operands
    [
        [m(cy, cz), fadd(m(m(sx, sy), cz), m(cx, sz)), fadd(m(m(cx, n(sy)), cz), m(sx, sz)), 0.0],
        [m(cy, n(sz)), fsub(m(cx, cz), m(m(sx, sy), sz)), fsub(m(sx, cz), m(m(cx, n(sy)), sz)), 0.0],
        [sy, m(n(sx), cy), m(cx, cy), 0.0],
        [trans[0], trans[1], trans[2], 1.0],
    ]
}

/// `Quat_QuatToMat` (ACTION.ELF 0x1e3e80), ported store for store: with (x, y, z, w) = the
/// quaternion's four floats, 2y/2z as doubling adds (`fVar1`/`fVar2` [M :18,:20]), `(2x)·x` and
/// `(2x)·w` (`fVar6`/`fVar8` [M :21-22]), the nine rotation entries in the decompile's order —
/// the adds/subs through the EE FPU's no-guard model (`add_noguard`, the same ops the verified
/// `drone::quat_quat_to_mat` replay uses) and the products chopped [M :23-31]:
/// m[0]=1−(2y²+2z²), m[1]=2xy+2zw, m[2]=2xz−2yw; m[4]=2xy−2zw, m[5]=1−(2x²+2z²), m[6]=2yz+2xw;
/// m[8]=2xz+2yw, m[9]=2yz−2xw, m[10]=1−(2x²+2y²) — read as the row-vector 3×3 (the game's MATRIX
/// rows at +0x00/+0x10/+0x20). The bottom row is the caller's: the 0xf2 dispatcher stores the
/// placement translation there [M 0x1d04f0:476-477] and `GunImp_Create` via `Mat_SetTrans`
/// [M 0x131f20:67].
pub fn quat_quat_to_mat(q: [f32; 4]) -> [[f32; 4]; 4] {
    use crate::ps2float as pf;
    let (x, y, z, w) = (q[0], q[1], q[2], q[3]);
    let add = |a: f32, b: f32| pf::add_noguard(a, b, true);
    let sub = |a: f32, b: f32| pf::add_noguard(a, -b, true);
    let mul = pf::mul;
    let a = add(y, y); // fVar1 = 2y [M :18]
    let b = add(z, z); // fVar2 = 2z [M :20]
    let c = mul(add(x, x), x); // fVar6 = (2x)·x [M :21]
    let d = mul(add(x, x), w); // fVar8 = (2x)·w [M :22]
    [
        [sub(1.0, add(mul(a, y), mul(b, z))), add(mul(a, x), mul(b, w)), sub(mul(b, x), mul(a, w)), 0.0],
        [sub(mul(a, x), mul(b, w)), sub(1.0, add(c, mul(b, z))), add(mul(b, y), d), 0.0],
        [add(mul(b, x), mul(a, w)), sub(mul(b, y), d), sub(1.0, add(c, mul(a, y))), 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// The placement classes whose creator builds the object's matrix with `View_RotTransMatrix`
/// 0x1e5168 (the Y·Z·X composition), i.e. every `parsemap_create_dynamic_objects` 0x1d04f0 case
/// whose creator reaches `control_create_object` 0x1344e0 with the quaternion arg = 0 — and all
/// 91 `control_create_object` call sites in ACTION.ELF pass a literal 0 (the `Quat_QuatToMat`
/// arm has no live caller), so the matrix is always `View_RotTransMatrix(obj+0x50, obj+0x30,
/// obj+0x90)` [M 0x1344e0:26-29]. Per-case evidence (creator → CCO(0)):
/// 0x20 `Break_Create` 0x12c230:12; 0x21 `Ripples_Create` 0x1acc90:14; 0x22 `Ladder_Create`
/// 0x1966c0:11; 0x27 `GT_Create` 0x17ffa0:69-71 (via `Control_CreateObjEx` 0x1343c8, which
/// forwards 0); 0x28 `Sensor_Create` 0x1af010:54; 0x29 `Switch_Create` 0x1adcc0:15; 0x2b
/// `LeafGen_Create` 0x1acbd0:11; 0x2c `Lightning_Create` 0x18cde8:16; 0x2e `Lock_Create`
/// 0x1ae1f8:17 (the pre-spline seed — swing/sliding doors and framed locks overwrite obj+0x90
/// from `Spline_Interp` path frames afterwards [M 0x135440:53-58, 0x136100:83-91]); 0x2f
/// `Copter_Create` 0x17d340:46; 0x30 `Monitor_Create` 0x1b08b0:12; 0x31 `FuseBox_Create`
/// 0x1b0448:12; 0x32 `RainBox_Create` 0x18d840:12; 0x33 `Hint_Create` 0x1b3248:26; 0x35
/// `PCQWorm_Create` 0x1b07a8:12-13; 0x36 `Sub_Create` 0x1ad6c0:17-18; 0x37 `MiniSub_Create`
/// 0x182130:13-14; 0x38 `Create_SpaceLaser` 0x1aa8b8:13-14; 0x39 `Create_SpaceMissile`
/// 0x1a9ef0:14-15; 0x3a `Grapple_Create` 0x198298:11; 0x3b `DynamicObject_Create` 0x18dbf8:13;
/// 0x3e `Wire_Create` 0x1981d8:13; 0x3f `CreepWall_Create` 0x196fc8:12; 0x4a `Apocalypse_Create`
/// 0x18dcc0:14; 0x71 `rotor_init` 0x18f4b8:13; 0xd2 `Shooter_Create` 0x17d030:10; 0xd4
/// `Cloud_Create` 0x18d620:13; 0xd5 `OneSided_Create` 0x1e7ce8:9; 0xd8 `RB_Create` 0x1d6bf0:42;
/// 0xd9 `SP_CreateScriptPlayer` 0x1db9a8:19; 0xdb `Door_Create` 0x136100:21 (pre-spline seed);
/// 0xdc/0xe8/0xea-0xef/0xf4 `Trigger_Create` 0x1b1720:52; 0xde `Searchlight_Create` 0x1cb4f8:25;
/// 0xe0 `Creature_Create` 0x134ab8:55-56; 0xe1 `Destroy_Create` 0x12c318:16; 0xe2 `SS_Create`
/// 0x1a9c58:16; 0xe3 `Flicker_Create` 0x1acb00:43; 0xe4 `Hurt_Create` 0x17e870:17; 0xe7
/// `ThirdCam_Create` 0x1bc510:11; 0xf0 `Pickup_Create` 0x18dda0:108-113 (the `Mat_Copy`
/// alternate is dead — the dispatcher passes MATRIX* 0 [M 0x1d04f0:459-462]); 0xf3
/// `Env_Tree_Create` 0x17c200:13; 0xf5/0xf8 `Drone_AIVolume_Create` 0x1376c0:52-64; 0xf9
/// `SoundTrigger_Create` 0x1b2628:41; 0xfa `ThirdIcon_Create` 0x197070:16; 0xfb
/// `MusicTrigger_Create` 0x1b30e0:13; 0xfc `Corona_Create` 0x18da00:11; 0xfd `CamSubject_Create`
/// 0x1bc5d8:11; 0xfe `Mine_Create` 0x17ce28:16.
///
/// The classes NOT in this set, and why:
/// - `creates_cel` rows (0x8000) never reach the dispatcher: `parsemap_block_map_data_static`
///   0x1d1998 stores the vectors raw and `parseentity_transform_bounding_box` 0x1d0fb0:28-29
///   builds the world bbox with `RotMatrix` + `Mat_SetTrans` — the X·Y·Z composition.
/// - 0x26 `MP_RegisterMPObject` 0x185990:64-65 builds with `RotMatrix` + `Mat_SetTrans`.
/// - 0x2a `View_AddSkyObj` 0x1e79a0 stores the vectors; `View_DrawSky` 0x1e5ad0:184,194 applies
///   `RotMatrix` at draw time.
/// - 0x0f `Drone_Create` 0x136be0 / 0xf1 `DroneSpawner_Create` 0x1378d0 pass NULL pos/rot to
///   CCO — the object matrix is the identity; the placement vectors live in the ext.
/// - 0x34 `GunImp_Create` 0x131f20:66-69 and 0xf2 (the dispatcher builds the emitter matrix
///   inline, 0x1d04f0:474-478) build with `Quat_QuatToMat` 0x1e3e80 on the placement's +0x24
///   quaternion — `object_matrix_for_class` returns that matrix (translation = the placement
///   position, as `Mat_SetTrans` [M 0x1e3478] / the dispatcher's row-3 stores write it); for 0x34
///   the prebuilt matrix then overwrites the created object's at `Control_CreateObjEx` 0x1343c8:18-23.
/// - 0x24/0x25/0x4e/0xe5/0xe6/0xe9/0xdf build no object matrix at all; 0xdf `Env_Create`
///   0x17b968:98-105 uses `RotMatrix` only to derive the wind direction vector.
/// Unlisted ids fall to the dispatcher's default (no creator, 0x1d04f0:544-545).
pub fn object_matrix_for_class(class: u32, creates_cel: bool, rot: [f32; 3], trans: [f32; 3], quat: [f32; 4]) -> [[f32; 4]; 4] {
    let view = !creates_cel
        && matches!(class,
            0x20 | 0x21 | 0x22 | 0x27 | 0x28 | 0x29 | 0x2b | 0x2c | 0x2e | 0x2f | 0x30 | 0x31
            | 0x32 | 0x33 | 0x35 | 0x36 | 0x37 | 0x38 | 0x39 | 0x3a | 0x3b | 0x3e | 0x3f | 0x41
            | 0x4a | 0x71 | 0xd2 | 0xd4 | 0xd5 | 0xd8 | 0xd9 | 0xdb | 0xdc | 0xde | 0xe0 | 0xe1
            | 0xe2 | 0xe3 | 0xe4 | 0xe7 | 0xe8 | 0xea..=0xef | 0xf0 | 0xf3 | 0xf4 | 0xf5 | 0xf8
            | 0xf9 | 0xfa | 0xfb | 0xfc | 0xfd | 0xfe);
    if view {
        view_rot_trans_matrix(rot, trans)
    } else if !creates_cel && matches!(class, 0x34 | 0xf2) {
        let mut m = quat_quat_to_mat(quat);
        m[3] = [trans[0], trans[1], trans[2], 1.0];
        m
    } else {
        object_matrix(rot, trans)
    }
}

pub fn transform(m: &[[f32; 4]; 4], v: [f32; 3]) -> [f32; 3] {
    [0, 1, 2].map(|c| v[0] * m[0][c] + v[1] * m[1][c] + v[2] * m[2][c] + m[3][c])
}

/// `View_RotTransMatrix` (ACTION.ELF 0x1e5168): the placement-matrix builder the game's DYNAMIC
/// object creations use — `SP_CreateScriptPlayer` 0x1db9a8 → `control_create_object` 0x1344e0
/// (param_4 == 0 → `View_RotTransMatrix(obj+0x50 (the stored rotation), obj+0x30 (the position),
/// obj+0x24 (the matrix))` [M]; the same creator serves the 0xd9 ScriptPlayer placements via
/// `parsemap_create_dynamic_objects` 0x1d04f0 case 0xd9). NOT the same composition as `RotMatrix`
/// 0x1e2368 — with a pitched rotation (rx ≠ 0) the two place the object differently, which is the
/// vent-grate bug: the wall grates of 07000014 (rot (−π/2, ±π/2, 0)) drew their panel 1.44 units
/// proud of the wall under the RotMatrix reading; this composition lays the panel FLUSH in the
/// wall plane (verified against the duct mouths at (30.10, 3.58, 46.42) / (51.58, 3.58, 44.94)).
/// The rows below follow the decompile with the output slots pinned from the DISASSEMBLY (the
/// decompiler's local names do not line up with PS2Sinf3's output pointers; the two calls write
/// (cx, sx, cy) to sp+0/4/8 and (cz, sz, sy) to sp+0xc/0x10/0x14 — jal 0x11fc20 with a0/a1/a2 =
/// sp, sp+4, sp+8 and sp+0xc, sp+0x10, sp+0x14, inputs f12 = rx+π/2 / rz+π/2, f14 = ry+π/2 / ry,
/// f13 = rx / rz):
/// row0 = [cy·cz, sz, −cz·sy], row1 = [sx·sy − cx·cy·sz, cx·cz, cy·sx + cx·sy·sz],
/// row2 = [cx·sy + cy·sx·sz, −cz·sx, cx·cy − sx·sy·sz], trans = the placement translation.
/// For rx = rz = 0 this collapses to the RotMatrix rows (every yaw-only placement draws
/// identically — the whole existing test battery is untouched by the swap).
pub fn view_rot_trans_matrix(rot: [f32; 3], trans: [f32; 3]) -> [[f32; 4]; 4] {
    use crate::anim::sinf;
    use crate::ps2float as pf;
    let fadd = |a, b| pf::add_noguard(a, b, true);
    let fsub = |a: f32, b: f32| pf::add_noguard(a, -b, true);
    let (rx, ry, rz) = (rot[0], rot[1], rot[2]);
    let half = f32::from_bits(0x3fc9_0fdb); // π/2 — the same literal the asm loads (0x3fc90fdb)
    let (cx, sx, cy) = (sinf(fadd(rx, half)), sinf(rx), sinf(fadd(ry, half)));
    let (cz, sz, sy) = (sinf(fadd(rz, half)), sinf(rz), sinf(ry));
    let m = pf::mul;
    [
        [m(cy, cz), sz, -(m(cz, sy)), 0.0],
        [fsub(m(sx, sy), m(m(cx, cy), sz)), m(cx, cz), fadd(m(cy, sx), m(m(cx, sy), sz)), 0.0],
        [fadd(m(cx, sy), m(m(cy, sx), sz)), -(m(cz, sx)), fsub(m(cx, cy), m(m(sx, sy), sz)), 0.0],
        [trans[0], trans[1], trans[2], 1.0],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(index: u16, hash: u32, flags: u32, t: [f32; 3], r: [f32; 3], params: &[(u32, u32)]) -> Vec<u8> {
        let mut v = vec![0u8; 0x4c];
        v[0..2].copy_from_slice(&index.to_le_bytes());
        v[4..8].copy_from_slice(&hash.to_le_bytes());
        v[8..12].copy_from_slice(&flags.to_le_bytes());
        for k in 0..3 {
            v[0x0c + 4 * k..0x10 + 4 * k].copy_from_slice(&t[k].to_le_bytes());
            v[0x18 + 4 * k..0x1c + 4 * k].copy_from_slice(&r[k].to_le_bytes());
        }
        v[0x48..0x4c].copy_from_slice(&(params.len() as u32).to_le_bytes());
        for (i, x) in params { v.extend(i.to_le_bytes()); v.extend(x.to_le_bytes()); }
        v
    }

    #[test]
    fn parses_records_with_parameters() {
        let d = [record(3, u32::MAX, 0xc047, [1.0, 2.0, 3.0], [0.0; 3], &[(0, 7), (5, 9)]), record(0, 0x0200_0644, 0x4e, [0.0; 3], [0.0, 1.5, 0.0], &[])].concat();
        let p = parse(&d).unwrap();
        assert_eq!(p.len(), 2);
        assert_eq!((p[0].index, p[0].creates_cel(), p[0].by_hash(), p[0].params.clone()), (3, true, false, vec![(0, 7), (5, 9)]));
        assert_eq!((p[1].hashcode, p[1].creates_cel(), p[1].rotation[1]), (0x0200_0644, false, 1.5));
        assert!(parse(&d[..d.len() - 1]).is_err());
    }

    #[test]
    fn matrix_matches_the_decomp_for_single_axes() {
        let close = |a: [f32; 3], b: [f32; 3]| a.iter().zip(&b).all(|(x, y)| (x - y).abs() < 1e-6);
        let h = std::f32::consts::FRAC_PI_2;
        // The 0-rotation matrix is the game's, not the platform's: PS2Sinf(π/2) = 0x3F800001 (one ulp above 1.0),
        // and every cosine is sinf(angle + π/2), so all three diagonal entries are products of two 1.0000001s.
        let m = object_matrix([0.0; 3], [1.0, 2.0, 3.0]);
        assert!(close(transform(&m, [1.0, 0.0, 0.0]), [2.0, 2.0, 3.0]));
        assert_eq!(m[0][0].to_bits(), 0x3f80_0002); // cy·cz, both PS2Sinf(π/2) = 1.0000001
        assert_eq!(m[1][1].to_bits(), 0x3f80_0002); // 1.0000002 − 0
        assert_eq!(m[2][2].to_bits(), 0x3f80_0002); // cx·cy
        assert_eq!(m[0][1].to_bits(), 0); // (sx·sy)·cz + cx·sz = 0 + 0
        // Identity and translation — NOT exact in the game either (the diagonal is 1.0000002).
        assert!(close(transform(&object_matrix([0.0; 3], [1.0, 2.0, 3.0]), [1.0, 0.0, 0.0]), [2.0, 2.0, 3.0]));
        // Entries from RotMatrix with only y = π/2: m[2][0] = sin y = PS2Sinf(π/2) = 1.0000001, m[0][2] = −sin y;
        // the π adds reduce to exact zeros (cy = PS2Sinf(π) = 0.0).
        let m = object_matrix([0.0, h, 0.0], [0.0; 3]);
        assert!(close(transform(&m, [1.0, 0.0, 0.0]), [0.0, 0.0, -1.0]));
        assert!(close(transform(&m, [0.0, 0.0, 1.0]), [1.0, 0.0, 0.0]));
        assert_eq!(m[2][0].to_bits(), 0x3f80_0001); // sinf(π/2)
        assert_eq!(m[2][2].to_bits(), 0); // cx·cy = 1.0000001 · 0.0
        // Only z = π/2: x → (cz, sz, ..) = y.
        let m = object_matrix([0.0, 0.0, h], [0.0; 3]);
        assert!(close(transform(&m, [1.0, 0.0, 0.0]), [0.0, 1.0, 0.0]));
    }

    /// `View_RotTransMatrix` 0x1e5168 composes Ry·Rz·Rx (column reading of the four 16-byte
    /// blocks); `RotMatrix` 0x1e2368 composes Rx·Ry·Rz. Same PS2Sinf3 values, different product
    /// grouping — the test rebuilds both closed forms from the game's own sinf and pins the
    /// port's two builders entry-for-entry, then checks the class predicate.
    #[test]
    fn view_rows_match_the_decompose_and_the_class_predicate_pins_the_builders() {
        use crate::ps2float as pf;
        let fadd = |a, b| pf::add_noguard(a, b, true);
        let fsub = |a: f32, b: f32| pf::add_noguard(a, -b, true);
        let mul = pf::mul;
        let sinf = crate::anim::sinf;
        let half = f32::from_bits(0x3fc9_0fdb);
        let closed = |rot: [f32; 3]| {
            let (x, y, z) = (rot[0], rot[1], rot[2]);
            // The two PS2Sinf3 calls, verbatim argument lists [M 0x1e5168:33-36 = 0x1e2368:28-31].
            let (cx, sx, cy) = (sinf(fadd(x, half)), sinf(x), sinf(fadd(y, half)));
            let (cz, sz, sy) = (sinf(fadd(z, half)), sinf(z), sinf(y));
            // View_RotTransMatrix's twelve stores [M 0x1e5168:37-48], read as rows.
            let view = [
                [mul(cy, cz), sz, -(mul(cz, sy))],
                [fsub(mul(sx, sy), mul(mul(cx, cy), sz)), mul(cx, cz), fadd(mul(cy, sx), mul(mul(cx, sy), sz))],
                [fadd(mul(cx, sy), mul(mul(cy, sx), sz)), -(mul(cz, sx)), fsub(mul(cx, cy), mul(mul(sx, sy), sz))],
            ];
            // RotMatrix's nine stores [M 0x1e2368:32-40], read as rows.
            let rot = [
                [mul(cy, cz), fadd(mul(mul(sx, sy), cz), mul(cx, sz)), fadd(mul(mul(cx, -sy), cz), mul(sx, sz))],
                [mul(cy, -sz), fsub(mul(cx, cz), mul(mul(sx, sy), sz)), fsub(mul(sx, cz), mul(mul(cx, -sy), sz))],
                [sy, mul(-sx, cy), mul(cx, cy)],
            ];
            (view, rot)
        };
        // A pitched + yawed + rolled rotation — the rows where the two builders disagree.
        let r = [0.37, 1.1, -0.53];
        let (view, rot) = closed(r);
        for i in 0..3 { for j in 0..3 {
            assert_eq!(view_rot_trans_matrix(r, [0.0; 3])[i][j].to_bits(), view[i][j].to_bits(), "view[{i}][{j}]");
            assert_eq!(object_matrix(r, [0.0; 3])[i][j].to_bits(), rot[i][j].to_bits(), "rot[{i}][{j}]");
        } }
        // And the two really do differ for that rotation (else the switch would be moot).
        assert!(view.iter().zip(&rot).any(|(a, b)| a.iter().zip(b).any(|(x, y)| x.to_bits() != y.to_bits())));
        // The class predicate [M the per-case catalog on object_matrix_for_class]: dynamic
        // dispatcher cases with a live matrix build → View; the static path, the pinned
        // RotMatrix exceptions and the unlisted ids → RotMatrix; the quaternion classes 0x34/0xf2
        // → `Quat_QuatToMat` on the +0x24 quaternion [M 0x1d04f0:474-478, 0x131f20:66-67,
        // 0x1343c8:18-23] (the old RotMatrix expectation here predated the quat port and
        // contradicted the decompile — fixed, see the WORKLOG).
        let q = [0.1, 0.2, 0.3, 0.9];
        for class in [0x20, 0x22, 0x28, 0x29, 0x2e, 0x31, 0x3a, 0x3b, 0x71, 0xd2, 0xd4, 0xd9, 0xdb, 0xdc, 0xe2, 0xea, 0xf0, 0xfc, 0xfe] {
            assert_eq!(object_matrix_for_class(class, false, r, [1.0, 2.0, 3.0], q), view_rot_trans_matrix(r, [1.0, 2.0, 3.0]), "{class:#x} must use View_RotTransMatrix");
        }
        for (class, cel) in [(0x19, true), (0x47, true), (0x0f, false), (0x26, false), (0x2a, false), (0xf1, false), (0x24, false), (0xdf, false), (0xf6, false), (0x99, false)] {
            assert_eq!(object_matrix_for_class(class, cel, r, [1.0, 2.0, 3.0], q), object_matrix(r, [1.0, 2.0, 3.0]), "{class:#x} (cel={cel}) must use RotMatrix");
        }
        for class in [0x34, 0xf2] {
            let mut want = quat_quat_to_mat(q);
            want[3] = [1.0, 2.0, 3.0, 1.0];
            assert_eq!(object_matrix_for_class(class, false, r, [1.0, 2.0, 3.0], q), want, "{class:#x} must use Quat_QuatToMat on the +0x24 quaternion");
            // A creates_cel row of a quaternion class never reaches the dispatcher either.
            assert_eq!(object_matrix_for_class(class, true, r, [1.0, 2.0, 3.0], q), object_matrix(r, [1.0, 2.0, 3.0]), "{class:#x} (cel) must use RotMatrix");
        }
        // A creates_cel row of a View class never reaches the dispatcher: the static path wins.
        assert_eq!(object_matrix_for_class(0xd9, true, r, [0.0; 3], q), object_matrix(r, [0.0; 3]));
    }

    /// `Quat_QuatToMat` 0x1e3e80: the nine entries replay the decompile's arithmetic on a
    /// non-trivial quat, and a pure-yaw quat closes onto the yaw-only RotMatrix rows (the port's
    /// base matrix) — the identity every yaw-only 0x34/0xf2 disc row relies on.
    #[test]
    fn quat_to_mat_matches_the_decompile_and_closes_on_yaw() {
        use crate::ps2float as pf;
        let q = [0.16, -0.33, 0.52, 0.77];
        let (x, y, z, w) = (q[0], q[1], q[2], q[3]);
        let add = |a: f32, b: f32| pf::add_noguard(a, b, true);
        let sub = |a: f32, b: f32| pf::add_noguard(a, -b, true);
        let mul = pf::mul;
        // The decompile's locals [M 0x1e3e80:15-31], verbatim.
        let (f1, f2) = (add(y, y), add(z, z));
        let (f6, f8) = (mul(add(x, x), x), mul(add(x, x), w));
        let want = [
            [sub(1.0, add(mul(f1, y), mul(f2, z))), add(mul(f1, x), mul(f2, w)), sub(mul(f2, x), mul(f1, w))],
            [sub(mul(f1, x), mul(f2, w)), sub(1.0, add(f6, mul(f2, z))), add(mul(f2, y), f8)],
            [add(mul(f2, x), mul(f1, w)), sub(mul(f2, y), f8), sub(1.0, add(f6, mul(f1, y)))],
        ];
        let m = quat_quat_to_mat(q);
        for i in 0..3 { for j in 0..3 { assert_eq!(m[i][j].to_bits(), want[i][j].to_bits(), "[{i}][{j}]"); } }
        assert_eq!(m[3], [0.0, 0.0, 0.0, 1.0]);
        // Pure yaw θ about +y: (0, sin(θ/2), 0, cos(θ/2)) — the entries replay the exact op
        // chains (f1 = 2y, f2 = 2z = 0), closing onto the row-vector RotY rows.
        let th = 0.8f32;
        let (s, c) = ((th * 0.5).sin(), (th * 0.5).cos());
        let (f1, sc) = (add(s, s), mul(add(s, s), c));
        let my = quat_quat_to_mat([0.0, s, 0.0, c]);
        assert_eq!(my[0][0].to_bits(), sub(1.0, mul(f1, s)).to_bits()); // 1 − 2sin²(θ/2)
        assert_eq!(my[0][2].to_bits(), sub(0.0, sc).to_bits()); // −2sin(θ/2)cos(θ/2)
        assert_eq!(my[2][0].to_bits(), sc.to_bits());
        assert_eq!(my[1][1].to_bits(), 1.0f32.to_bits());
        assert_eq!((my[0][1], my[1][0], my[1][2], my[2][1]), (0.0, 0.0, 0.0, 0.0));
        // The unit quat's rotation-sized entries.
        let m2 = quat_quat_to_mat([0.5, 0.5, 0.5, 0.5]);
        for row in &m2 { for v in &row[..3] { assert!(v.abs() <= 1.0 + 1.0e-6, "rotation-sized entries only: {v}"); } }
    }

    /// Yaw-only rows (rx = rz = 0): the builders agree to the PS2Sinf(π/2) = 1.0000001 ulp —
    /// bit-equal on every entry except [0][2] and [2][0], where View carries one cosine factor
    /// and RotMatrix two (cx·cz = 1.0000002), and [1][0], which is +0.0 under View and −0.0
    /// under RotMatrix (cy·−sz vs the subtrahend form). Transformed geometry moves ≤ 10⁻⁷ per
    /// unit — the scale the vent-grates swap already accepted battery-wide.
    #[test]
    fn yaw_only_rows_agree_between_the_builders_to_the_cosine_ulp() {
        let r = [0.0, 0.8, 0.0];
        let (v, o) = (view_rot_trans_matrix(r, [1.0, 2.0, 3.0]), object_matrix(r, [1.0, 2.0, 3.0]));
        for (i, j) in [(0, 0), (0, 1), (1, 1), (1, 2), (2, 1), (2, 2), (3, 0), (3, 1), (3, 2)] {
            assert_eq!(v[i][j].to_bits(), o[i][j].to_bits(), "[{i}][{j}] must be bit-equal for yaw-only rows");
        }
        assert_eq!(v[1][0].to_bits(), 0); // +0.0 — the subtrahend form
        assert_eq!(o[1][0].to_bits(), 0x8000_0000); // −0.0 — cy·−sz
        assert!(((v[0][2] - o[0][2]).abs() as f64) < 2.0e-6 && ((v[2][0] - o[2][0]).abs() as f64) < 2.0e-6);
        for p in [[1.0, 0.5, -0.25], [10.0, -3.0, 2.0]] {
            let (a, b) = (transform(&v, p), transform(&o, p));
            for k in 0..3 { assert!((a[k] - b[k]).abs() < 2.0e-6, "point {p:?} axis {k}"); }
        }
    }
}
