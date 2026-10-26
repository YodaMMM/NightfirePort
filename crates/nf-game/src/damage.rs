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

/// Bond's 16 body boxes (`AnimGetCollData` 0x1c4ac8 over `HumanCollisionBones` 0x2c6be8 [D]) at the
/// [L] standing idle pose, riding Bond's yaw: the game's guard-round test is `Collide_Intersect`
/// 0x1ebaa8:101-105's `Collide_Jointy` gate (the player obj carries glist and obj+0xFA bit 3 —
/// both live [L ee_start_05.bin obj 0x16cefe0: glist 0x17381d0, +0xFA = 0x8]) with kind 0x201, so
/// every round tests these boxes — the old upright cylinder was the documented deviation.
/// Unchanged from the game: the box order (list order wins, `Collide_Jointy` 0x1ec180 returns on
/// the first box hit), the part ids (bone a), the half extents × the skin scale, the box matrix =
/// the other bone's rotation, entry 0's centre rule (the bone plus (0, bone.y − other.y, 0)).
/// Not modelled: the palette's walk/crouch animation (the boxes hold the idle stance at any yaw).
pub fn bond_boxes(d: &Data<'_>, yaw: f32, pos: V3) -> Vec<HitBox> {
    let sk = player_skin_hash(d.level);
    let Some(skin) = d.skins.get(&sk) else { return Vec::new() };
    let table = d.hitboxes.for_skin(sk);
    let om = nf_formats::placement::object_matrix([0.0, yaw, 0.0], pos);
    // The yaw matrix's 3×3 (row-vector rows; object_matrix's rows 0..2, the translation is row 3).
    let ry = [[om[0][0], om[0][1], om[0][2]], [om[1][0], om[1][1], om[1][2]], [om[2][0], om[2][1], om[2][2]]];
    let mut palette = vec![[[0.0, 0.0, 0.0, 0.0]; 4]; 60];
    for (bone, t, r) in BOND_IDLE_POSE {
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
