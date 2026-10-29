//! Bullets against drones: the hit boxes (`AnimGetCollData` 0x1c4ac8), the ray test (`Collide_Jointy` 0x1ec180) and the
//! damage (`NDrone2_HitDamage` 0x145558). Spec: docs/systems/drone-damage.md.

use crate::drone::Data;
use crate::world::V3;
use nf_formats::anim::{self, Mat4};
use nf_formats::hitbox::CollisionBone;
use nf_formats::tuning::Tuning;

/// One oriented box: rows 0–2 the other bone's axes, row 3 the centre; half-extents in those axes; the part (bone).
#[derive(Clone, Copy, Debug)]
pub struct HitBox { pub m: Mat4, pub half: [f32; 3], pub part: i16 }

/// The drone's object matrix as nf-view draws it: yaw about +Y, x/z from the drone, y from its feet plus the hip height
/// (`Drone+0xA0` = hip − 0.02, at least 0.2; 1.0 without a hip).
pub fn drone_height(hip: f32) -> f32 { if hip == 0.0 { 1.0 } else { (hip - 0.02).max(0.2) } }

/// `AnimGetCollData`: boxes from the bone world matrices (`palette`, world space) per collision-bone entry. Entry 0's
/// centre is the bone plus (0, bone.y − other.y, 0); the others sit midway between bone and other. The box axes are a
/// copy of the other bone's matrix. Extents × skin scale.
pub fn boxes(table: &[CollisionBone], palette: &[Mat4], skin: &nf_formats::character::Skin) -> Vec<HitBox> {
    let mut out = Vec::with_capacity(table.len());
    for (k, e) in table.iter().enumerate() {
        let b = e.bone as usize;
        let o = if e.other < 0 { match skin.parent(b) { Some(p) => p, None => continue } } else { e.other as usize };
        let (Some(mb), Some(mo)) = (palette.get(b), palette.get(o)) else { continue };
        let mut m = *mo;
        let (tb, to) = ([mb[3][0], mb[3][1], mb[3][2]], [mo[3][0], mo[3][1], mo[3][2]]);
        let c = if k == 0 { [tb[0], tb[1] + (tb[1] - to[1]), tb[2]] } else { [0, 1, 2].map(|i| (tb[i] + to[i]) * 0.5) };
        m[3] = [c[0], c[1], c[2], 1.0];
        out.push(HitBox { m, half: [e.half[0] * skin.scale[0], e.half[1] * skin.scale[1], e.half[2] * skin.scale[2]], part: e.bone });
    }
    out
}

/// World bone matrices of a posed skeleton: `local` with the root translation zeroed (`AnimSeqTick` takes it as root
/// motion), then placed by `object` (row vectors: world = model · object).
pub fn palette(local: &mut [([f32; 3], [f32; 4])], skin: &nf_formats::character::Skin, object: &Mat4) -> Vec<Mat4> {
    if let Some(r) = local.first_mut() { r.0 = [0.0; 3]; }
    anim::world_pose(local, skin).iter().map(|w| anim::mul(w, object)).collect()
}

/// `Intersect_RayBox` in the box's frame: the segment `start + t·ray`, t in [0, 1], against [−half, half]; the entry t
/// (0 when the start is inside).
pub fn ray_box(start: V3, ray: V3, b: &HitBox) -> Option<f32> {
    let c = [b.m[3][0], b.m[3][1], b.m[3][2]];
    let (mut t0, mut t1) = (0f32, 1f32);
    for k in 0..3 {
        let axis = [b.m[k][0], b.m[k][1], b.m[k][2]];
        let l2 = axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2];
        if l2 <= 0.0 { return None; }
        let p = ((start[0] - c[0]) * axis[0] + (start[1] - c[1]) * axis[1] + (start[2] - c[2]) * axis[2]) / l2.sqrt();
        let d = (ray[0] * axis[0] + ray[1] * axis[1] + ray[2] * axis[2]) / l2.sqrt();
        let h = b.half[k];
        if d.abs() < 1e-12 {
            if p < -h || p > h { return None; }
        } else {
            let (a, z) = ((-h - p) / d, (h - p) / d);
            let (lo, hi) = if a < z { (a, z) } else { (z, a) };
            t0 = t0.max(lo);
            t1 = t1.min(hi);
            if t0 > t1 { return None; }
        }
    }
    Some(t0)
}

/// `Collide_Jointy` for a ray: the first box in list order the segment hits (not the nearest), with its t.
pub fn first_box_hit(start: V3, ray: V3, boxes: &[HitBox]) -> Option<(i16, f32)> {
    boxes.iter().find_map(|b| ray_box(start, ray, b).map(|t| (b.part, t)))
}

/// Bond's skin record hash (`AnimObj+0x94 → +0` [L ee_start_05.bin @ AnimObject 0x16cb8a0:
/// 0x5000007]; scale (1.0, 1.0, 1.0) [L same]). Not the astronaut 0x5000078, so
/// `CollisionBones::for_skin` picks `HumanCollisionBones`.
pub const BOND_SKIN: u32 = 0x0500_0007;

/// The player's skin hash, per level — `Player_Init` 0x1910f8:94-108's switch on the level id
/// (`GameState+0x12`), the value handed to `AnimObjectNew` [M]: 0x0500009 on 07000001-07000004,
/// 0x050000d on 07000007/07000008, 0x050007c on 0700001b, 0x0500007 everywhere else. The disc
/// carries each per its own levels (0x05000009 in 01-04's own archives, 0x050000d in 05-08's,
/// 0x0500007c in 16/1b/43 [D nf-extract bin-list census, kind-3 entries]); on 07000048 none of
/// the player skins is in the level's own archive — the game's session hashtable still holds
/// 0x05000007 from the previous level's containers [M AnimLoadFile 0x1d8248 registers and nothing
/// unregisters], so the port's loader merges it the same way (nf-view's load).
pub fn player_skin_hash(level: u32) -> u32 {
    match level {
        0x0700_0001..=0x0700_0004 => 0x0500_0009,
        0x0700_0007 | 0x0700_0008 => 0x0500_000d,
        0x0700_001b => 0x0500_007c,
        _ => BOND_SKIN,
    }
}

/// Bond's standing idle pose, root-local frame [L]: the AnimObject+0xB8 palette of the 07000001 /
/// 07000005 / 0700000a level-start EE snapshots with the object matrix (obj+0x90) undone — the
/// three levels agree to ±0.011 on every axis (the idle sway; all three are the idle's first
/// phase). Each row: (bone, local translation, local rotation rows). The rotation is the palette
/// matrix's 3×3 with the object matrix right-multiplied out; the rebuild is
/// `world_rows = rows · Ryaw`, `world_trans = pos + t · Ryaw` (row-vector convention, the game's
/// `psiBuildMatrixPalette` 0x111498 composes the local pose with obj+0x90 the same way).
/// The anim-driven palette (walking, crouching) is not modelled — this is the idle stance.
#[rustfmt::skip]
pub(crate) const BOND_IDLE_POSE: [(i16, [f32; 3], [[f32; 3]; 3]); 20] = [
    ( 0, [0.0000, 0.0109, 0.0000], [[0.7277, -0.0418, 0.6847], [0.1719, 0.9774, -0.1230], [-0.6641, 0.2072, 0.7184]]),
    ( 1, [-0.0153, 0.0683, 0.0197], [[0.7267, 0.0363, 0.6860], [-0.1763, 0.9750, 0.1351], [-0.6639, -0.2191, 0.7150]]),
    ( 4, [-0.0900, 0.5049, 0.1191], [[0.8618, 0.1477, 0.4853], [-0.3143, 0.9063, 0.2824], [-0.3981, -0.3959, 0.8275]]),
    ( 5, [-0.1239, 0.5449, 0.1671], [[0.9338, 0.2650, 0.2406], [-0.3129, 0.9308, 0.1892], [-0.1738, -0.2519, 0.9520]]),
    (19, [0.0615, 0.3772, 0.2472], [[-0.0012, -0.2600, 0.9656], [0.6518, 0.7321, 0.1980], [-0.7584, 0.6296, 0.1685]]),
    (20, [0.0758, 0.2789, 0.5224], [[-0.6074, 0.3519, 0.7122], [0.6737, 0.7032, 0.2272], [-0.4208, 0.6178, -0.6642]]),
    (21, [-0.1123, 0.3838, 0.7060], [[-0.8306, 0.4493, 0.3290], [-0.4447, -0.8908, 0.0940], [0.3353, -0.0683, 0.9396]]),
    (23, [-0.2304, 0.4311, 0.7701], [[0.0830, 0.9957, 0.0402], [-0.9380, 0.0644, 0.3406], [0.3366, -0.0660, 0.9393]]),
    (31, [-0.2504, 0.3748, -0.0176], [[0.6330, 0.3029, -0.7125], [-0.3775, 0.9242, 0.0575], [0.6759, 0.2326, 0.6993]]),
    (32, [-0.4514, 0.2607, 0.1769], [[-0.8627, -0.3721, -0.3424], [-0.4043, 0.9143, 0.0249], [0.3038, 0.1599, -0.9392]]),
    (35, [-0.1961, 0.3576, 0.2428], [[0.0141, 0.0041, -0.9999], [-0.9975, 0.0694, -0.0138], [0.0694, 0.9976, 0.0051]]),
    (39, [-0.2181, 0.3812, 0.3792], [[-0.3646, 0.0301, -0.9307], [-0.9286, 0.0627, 0.3659], [0.0694, 0.9976, 0.0051]]),
    (49, [-0.1599, -0.0928, -0.0344], [[0.3020, -0.2935, 0.9070], [0.4764, 0.8706, 0.1231], [-0.8257, 0.3949, 0.4027]]),
    (50, [-0.3769, -0.4662, -0.0938], [[0.2848, -0.2926, 0.9128], [-0.2035, 0.9121, 0.3559], [-0.9367, -0.2872, 0.2002]]),
    (51, [-0.2153, -0.9009, -0.2974], [[0.2077, 0.0005, 0.9782], [-0.0002, 1.0000, -0.0005], [-0.9782, -0.0001, 0.2077]]),
    (52, [-0.3396, -0.9670, -0.2931], [[0.2077, 0.0006, 0.9782], [-0.0002, 1.0000, -0.0005], [-0.9782, -0.0001, 0.2077]]),
    (53, [0.0379, -0.1041, 0.1517], [[0.9783, 0.1212, 0.1682], [0.0336, 0.7078, -0.7056], [-0.2046, 0.6959, 0.6884]]),
    (54, [0.0397, -0.4038, 0.4664], [[0.9840, 0.1086, 0.1415], [-0.1132, 0.9933, 0.0251], [-0.1378, -0.0407, 0.9896]]),
    (55, [0.1249, -0.8976, 0.3924], [[0.9930, -0.0389, 0.1117], [0.0361, 0.9990, 0.0272], [-0.1127, -0.0230, 0.9934]]),
    (56, [0.1278, -0.9710, 0.5125], [[0.9935, -0.0125, 0.1129], [0.0121, 0.9999, 0.0040], [-0.1129, -0.0026, 0.9936]]),
];

/// Bond's crouch idle pose, root-local frame — the SAME derivation as `BOND_IDLE_POSE`, from the
/// crouch idle's own anim: the SP crouch anim-set descriptor [D 0x2b91d8 = index 1 of the 16
/// @0x2b91b0, `PlayerAnimSetInitCrouch` 0x18fea0's SP arm] → idle script 0x060009ff → sequence
/// 0x0400084e (skeleton 0, 2 frames) frame 1, through the port's own player
/// (`anim::local_pose` + `anim::world_pose`, the same pipeline the standing idle validates against
/// the snapshot table's rotations exactly). The root bone's translation (the anim's root motion)
/// is anchored out — the game moves it into the object (obj+0x30 rides the stance's cc:
/// `CROUCH_ROOT_HEIGHT` 0.55 [L, M AnimObj+0xCC]), which the port's `Bond::pos` already carries.
/// The legs fold up under the hips (bones 51/52 at y −0.30/−0.43 vs the idle's −0.90/−0.97) and
/// the head origin drops to +0.405 (idle +0.545); the composed head box lands ~0.5 lower in the
/// world once the dropped root is applied. The [L] crouched-palette crosscheck is requested in the
/// report's live checks (no crouched snapshot exists in work/).
#[rustfmt::skip]
pub(crate) const BOND_CROUCH_POSE: [(i16, [f32; 3], [[f32; 3]; 3]); 20] = [
    ( 0, [0.0000, 0.0000, 0.0000], [[0.7351, 0.3174, 0.5990], [0.0831, 0.8347, -0.5444], [-0.6728, 0.4500, 0.5872]]),
    ( 1, [-0.0201, 0.0590, -0.0065], [[0.8359, -0.0184, 0.5486], [-0.3593, 0.7372, 0.5722], [-0.4150, -0.6754, 0.6096]]),
    ( 4, [-0.2147, 0.3524, 0.2584], [[0.8663, 0.4968, 0.0520], [-0.4957, 0.8679, -0.0332], [-0.0616, 0.0030, 0.9981]]),
    ( 5, [-0.2470, 0.4050, 0.2937], [[0.9880, 0.1423, 0.0594], [-0.1251, 0.9650, -0.2306], [-0.0901, 0.2204, 0.9712]]),
    (19, [0.0035, 0.2655, 0.3567], [[0.7808, -0.5511, 0.2942], [0.2740, 0.7253, 0.6315], [-0.5615, -0.4125, 0.7174]]),
    (20, [0.2434, 0.1099, 0.4183], [[-0.8789, 0.0577, 0.4735], [0.3857, 0.6700, 0.6343], [-0.2806, 0.7401, -0.6111]]),
    (21, [-0.0149, 0.1349, 0.5312], [[-0.5943, -0.0077, 0.8042], [0.4661, 0.8116, 0.3523], [-0.6554, 0.5842, -0.4787]]),
    (23, [-0.1038, 0.1527, 0.6384], [[-0.7508, -0.4429, 0.4901], [0.0728, 0.6819, 0.7278], [-0.6565, 0.5821, -0.4797]]),
    (31, [-0.2967, 0.2740, 0.0568], [[0.0201, 0.5284, 0.8487], [-0.8436, 0.4646, -0.2693], [-0.5366, -0.7106, 0.4551]]),
    (32, [-0.2732, 0.1254, -0.2051], [[0.5521, 0.8255, -0.1176], [-0.8157, 0.5054, -0.2814], [-0.1729, 0.2513, 0.9523]]),
    (35, [-0.4220, -0.1035, -0.1390], [[0.7023, 0.7074, 0.0789], [-0.6682, 0.6934, -0.2696], [-0.2455, 0.1366, 0.9597]]),
    (39, [-0.5380, -0.1820, -0.1339], [[0.3970, 0.9174, -0.0290], [-0.8844, 0.3739, -0.2794], [-0.2455, 0.1366, 0.9597]]),
    (49, [-0.1503, -0.1200, 0.0219], [[0.7838, -0.3119, 0.5370], [0.5534, 0.7431, -0.3761], [-0.2818, 0.5920, 0.7551]]),
    (50, [-0.4015, -0.4362, 0.1863], [[0.7472, -0.3305, 0.5765], [-0.6423, -0.1367, 0.7541], [-0.1704, -0.9338, -0.3144]]),
    (51, [-0.0771, -0.3006, -0.1782], [[0.7029, 0.0400, 0.7101], [-0.7028, 0.1923, 0.6848], [-0.1092, -0.9805, 0.1633]]),
    (52, [-0.0593, -0.4343, -0.2189], [[0.7815, -0.0978, 0.6162], [-0.0316, 0.9802, 0.1956], [-0.6231, -0.1723, 0.7629]]),
    (53, [0.0494, -0.0338, 0.1847], [[0.9747, 0.0984, 0.2007], [0.2215, -0.3051, -0.9262], [-0.0299, 0.9472, -0.3192]]),
    (54, [-0.0290, 0.1078, 0.5879], [[0.9830, 0.0584, 0.1740], [-0.0780, 0.9911, 0.1078], [-0.1662, -0.1195, 0.9788]]),
    (55, [0.0403, -0.3808, 0.4738], [[0.9930, -0.0389, 0.1117], [0.0361, 0.9990, 0.0272], [-0.1127, -0.0230, 0.9934]]),
    (56, [0.0432, -0.4542, 0.5939], [[0.9956, -0.0049, 0.0933], [0.0092, 0.9989, -0.0462], [-0.0930, 0.0468, 0.9946]]),
];

/// Bond's 16 body boxes (`AnimGetCollData` 0x1c4ac8 over `HumanCollisionBones` 0x2c6be8 [D]) at the
/// [L] standing idle pose, riding Bond's yaw: the game's guard-round test is `Collide_Intersect`
/// 0x1ebaa8:101-105's `Collide_Jointy` gate (the player obj carries glist and obj+0xFA bit 3 —
/// both live [L ee_start_05.bin obj 0x16cefe0: glist 0x17381d0, +0xFA = 0x8]) with kind 0x201, so
/// every round tests these boxes — the old upright cylinder was the documented deviation.
/// Unchanged from the game: the box order (list order wins, `Collide_Jointy` 0x1ec180 returns on
/// the first box hit), the part ids (bone a), the half extents × the skin scale, the box matrix =
/// the other bone's rotation, entry 0's centre rule (the bone plus (0, bone.y − other.y, 0)).
/// Not modelled: the palette's walk/crouch animation (the boxes hold the idle stance at any yaw).
pub fn bond_boxes(d: &Data<'_>, yaw: f32, pos: V3, crouch: bool) -> Vec<HitBox> {
    let sk = player_skin_hash(d.level);
    let Some(skin) = d.skins.get(&sk) else { return Vec::new() };
    let table = d.hitboxes.for_skin(sk);
    // bond-hitbox-pose-2: the crouch pose's root-local frame assumes the game's SETTLED CROUCHED
    // root — floor + cc + 0.051 = 0.607 [L the truck ride] vs the standing 1.086 — so the
    // palette rides ROOT_HEIGHT − CROUCH_ROOT_HEIGHT = 0.4847 below `pos`. The -pose branch
    // carried it by moving Bond::pos; that dropped the CAMERA 0.4847 instantly on top of
    // head()'s eased crouch_drop (~0.93 in one tick) and re-seated the walk's floor rest, so
    // the offset lives HERE: `pos`, the capsule and head() stay exactly master's [M master's
    // toggle writes crouched + cc only; the ease ±0.0225 → 0.45 is the whole view change].
    let pos = if crouch {
        [pos[0], pos[1] - (crate::player::ROOT_HEIGHT - crate::player::CROUCH_ROOT_HEIGHT), pos[2]]
    } else {
        pos
    };
    let om = nf_formats::placement::object_matrix([0.0, yaw, 0.0], pos);
    // The yaw matrix's 3×3 (row-vector rows; object_matrix's rows 0..2, the translation is row 3).
    let ry = [[om[0][0], om[0][1], om[0][2]], [om[1][0], om[1][1], om[1][2]], [om[2][0], om[2][1], om[2][2]]];
    let mut palette = vec![[[0.0, 0.0, 0.0, 0.0]; 4]; 60];
    for (bone, t, r) in if crouch { BOND_CROUCH_POSE } else { BOND_IDLE_POSE } {
        let mut m = [[0.0f32; 4]; 4];
        for i in 0..3 {
            for j in 0..3 { m[i][j] = r[i][0] * ry[0][j] + r[i][1] * ry[1][j] + r[i][2] * ry[2][j]; }
        }
        let tv = [0, 1, 2].map(|j| pos[j] + t[0] * ry[0][j] + t[1] * ry[1][j] + t[2] * ry[2][j]);
        m[3] = [tv[0], tv[1], tv[2], 1.0];
        palette[bone as usize] = m;
    }
    boxes(table, &palette, skin)
}

/// What `NDrone2_HitDamage` needs from the drone.
#[derive(Clone, Copy, Debug)]
pub struct Target { pub damageable: bool, pub armour: u8, pub mode: u8, pub skin: u32, pub kind: u16 }

/// `NDrone2_HitDamage`'s result: health lost, the value recorded in `+0x150`, and whether `+0x3A` (head shot, silent
/// kill) was set.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Damage { pub loss: f32, pub recorded: f32, pub head: bool }

/// `NDrone2_HitDamage` (0x145558) for a hit of `dmg` (HIT+8 · `Drone+0x14C`) on `part` with `weapon`.
/// `difficulty` = `GameState+0x28`. A drone with `+0x14` = 0 loses nothing; unless forced it only records `dmg`.
pub fn hit_damage(t: &Tuning, target: Target, dmg: f32, part: i16, weapon: u16, forced: bool, difficulty: u32, level: u32) -> Damage {
    use nf_formats::ps2float::mul;
    let mut part = if part < 0 { 1 } else { part };
    if part == 5 && matches!(weapon, 1 | 0x4a..=0x4d) { part = 1; }
    if level == 0x0700_001b && part != 5 { part = 1; }
    if !target.damageable && !forced { return Damage { loss: 0.0, recorded: dmg, head: false }; }
    let a = target.armour;
    let mut head = false;
    let mut d = dmg;
    match part {
        5 => {
            let f = if target.mode == 0xd || target.skin == 0x0500_00ba || (target.kind == 0x10 && level == 0x0700_0014) {
                t.DroneDamage_Torso + t.DroneDamage_Torso
            } else { head = true; t.DroneDamage_Head };
            d = mul(d, f);
            if a & 4 != 0 { d = mul(d, t.DroneArmour_Helmet); }
        }
        0x14 | 0x15 | 0x17 | 0x20 | 0x23 | 0x27 => {
            d = mul(d, t.DroneDamage_Arms);
            if a & 8 != 0 { d = mul(d, t.DroneArmour_Combat); }
            if a & 2 != 0 { d = mul(d, t.DroneArmour_Jacket); }
        }
        0x31..=0x38 => {
            d = mul(d, t.DroneDamage_Legs);
            if a & 8 != 0 { d = mul(d, t.DroneArmour_Combat); }
        }
        _ => {
            d = mul(d, t.DroneDamage_Torso);
            if a & 8 != 0 { d = mul(d, t.DroneArmour_Combat); }
            if a & 2 != 0 { d = mul(d, t.DroneArmour_Jacket); }
            if a & 1 != 0 { d = mul(d, t.DroneArmour_Vest); }
        }
    }
    let k = match difficulty { 1 => t.DroneDamage_Easy, 3 | 4 => t.DroneDamage_Hard, _ => t.DroneDamage_Normal };
    let d = mul(d, k);
    Damage { loss: if target.damageable { d } else { 0.0 }, recorded: d, head }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_box(c: V3, half: [f32; 3], part: i16) -> HitBox {
        HitBox { m: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [c[0], c[1], c[2], 1.0]], half, part }
    }

    #[test]
    fn rays_hit_boxes_in_list_order() {
        let a = unit_box([0.0, 0.0, 5.0], [0.5; 3], 4);
        let b = unit_box([0.0, 0.0, 2.0], [0.5; 3], 5);
        // The far box comes first in the list and wins although the near one is hit earlier along the ray.
        assert_eq!(first_box_hit([0.0, 0.0, 0.0], [0.0, 0.0, 10.0], &[a, b]), Some((4, 0.45)));
        assert_eq!(first_box_hit([0.0, 0.0, 0.0], [0.0, 0.0, 10.0], &[b, a]), Some((5, 0.15)));
        // Missing to the side, and stopping short.
        assert_eq!(first_box_hit([1.0, 0.0, 0.0], [0.0, 0.0, 10.0], &[a, b]), None);
        assert_eq!(first_box_hit([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], &[a, b]), None);
        // Starting inside: t = 0.
        assert_eq!(ray_box([0.0, 0.0, 2.1], [0.0, 0.0, 1.0], &b), Some(0.0));
    }

    #[test]
    fn hit_damage_per_part_and_difficulty() {
        let mut t = Tuning::default();
        (t.DroneDamage_Easy, t.DroneDamage_Normal, t.DroneDamage_Hard) = (1.5, 1.0, 0.8);
        (t.DroneDamage_Head, t.DroneDamage_Legs, t.DroneDamage_Arms, t.DroneDamage_Torso) = (10.0, 0.75, 1.0, 1.0);
        let g = Target { damageable: true, armour: 0, mode: 0, skin: 0, kind: 2 };
        // P2K 3.5 on 07000005: torso Easy 5.25, legs Normal 2.625, head 35 and a silent kill; fists on the head = torso.
        let h = |part, weapon, diff| { let r = hit_damage(&t, g, 3.5, part, weapon, false, diff, 0x0700_0005); (r.loss, r.head) };
        assert_eq!(h(4, 2, 1), (5.25, false));
        assert_eq!(h(0x35, 2, 2), (2.625, false));
        assert_eq!(h(5, 2, 2), (35.0, true));
        assert_eq!(h(5, 1, 2), (3.5, false));
        assert_eq!(h(4, 2, 3), (2.8, false));
        let off = hit_damage(&t, Target { damageable: false, ..g }, 3.5, 4, 2, false, 2, 0x0700_0005);
        assert_eq!((off.loss, off.recorded), (0.0, 3.5));
    }
}
