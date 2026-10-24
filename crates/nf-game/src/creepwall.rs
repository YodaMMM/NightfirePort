//! The creepwall — the crawlable vent duct (ACTION.ELF). There is no grate class: the crawlable vent
//! is a **creepwall** (placement 0x3F → `CreepWall_Create` 0x196fc8, class 0x2B = '+'), and the grate
//! panel itself is an ordinary door (class 0x1C, `Door_Activate` 0x135be0) [M, fix-ladder2's census].
//! The use path: `Player_Activate` 0x1a5170:66-97 — with the third-icon context 6, the use key scans
//! every object for class '+' and calls `Player_CollCreepWall` 0x196758 on the NEAREST within 1000
//! units (no capsule test). The crawl: substate 7, `Player_Creep` 0x1969d0 — `Music_Event(0xD,1)`
//! and `switch_channels[params+6] = 1` every tick, the exit at the wall's ends (the input direction
//! decides), the channel cleared on the way out.

use crate::world::V3;

#[derive(Clone, Debug)]
pub struct CreepWall {
    /// The placement index (the port's identity for the object).
    pub placement: u32,
    /// The wall's pos vec (`obj+0x80`, what `Player_CollCreepWall` 0x196758:78-79 reads as the
    /// crawl line's centre). That word is the WORLD-SPHERE CENTRE `Control_BuildWorldSph` 0x133070
    /// wrote: the object's LOCAL bound centre transformed through the object's yaw matrix — the
    /// bound-centre offset rides along (07000005 #299 "Wall Hug Short" centre x +1.95 [D the parsed
    /// object bounds] puts the world centre 1.95 up the crawl axis from the placement translation).
    pub pos: V3,
    /// The placement's yaw (`obj+0x54`); the wall's axis runs along `yaw + π/2`
    /// [M 0x196758:80-86 + 0x1969d0:99-100: `Vec_Spherical_2_Cartesian(len, yaw + 1.5707964, 0)`].
    pub yaw: f32,
    /// The axis half-length (`obj+0x8c`). `CreepWall_Create` 0x196fc8 sets no +0x8c and neither does
    /// `control_create_object` 0x1344e0 (its only stores are +0x30/+0x40/+0x50/+0x60/+0x90/+0xe0 —
    /// the disasm's full store scan) — the write is `Control_BuildWorldSph` 0x133070 (called from
    /// `build_LinkToRoom` 0x1c9680:40, which every Create ends with):
    /// `*(float*)(obj+0x8c) = *(float*)(record+0x1c) * ABS(*(float*)(obj+0xe8))` — the object's
    /// bound-sphere RADIUS (the same field the icons read as obj+0x8c [M wire.rs's Icon.radius,
    /// 0x1c9680:42]) times the placement scale. The port reads the parsed object's radius × the
    /// scale product (07000005: "Wall Hug Short" r 5.60, "Wall Hug Window" r 5.36 [D the parsed
    /// objects] — the old half-span read (5.57/5.32) sat 0.03/0.04 under). The 07000005 crawl
    /// band's floor dip (z 67-70 down to y 7-8 [D the 1-unit floor map]) is only crossable when the
    /// crawl spans the prompts' z 60-70.6 [D placements #256 (24.20, 31.93, 60.01) / #257 (24.14,
    /// 31.87, 70.56)] — the radius model's band z 60.21..71.41 covers exactly that (the old
    /// translation-centred band 62.19..73.33 left the z-60 prompt 2.2 outside).
    pub half_len: f32,
    /// The switch channel held while Bond crawls: the 4th param word's low half (level+0x38 →
    /// the object's BLData u16[3], read as `*(short *)(params + 6)` [M 0x196fc8:19-22 +
    /// 0x1969d0:86-88]).
    pub ch: u16,
}

impl CreepWall {
    /// The wall's axis endpoints: pos ± fwd(yaw + π/2) · half_len [M 0x196758:80-90: the two vectors
    /// `p + d` and `p − d` with `d = Cartesian(len, yaw + π/2, 0)`].
    pub fn endpoints(&self) -> (V3, V3) {
        // G-046: d = Vec_Spherical_2_Cartesian(len, yaw + π/2, 0) [M 0x196758:80-90]: the game's θ add is
        // an EE add.s and the φ = 0 lane still multiplies by cos(0) = sinf(1.5707964) = 0x3f800001 [M
        // 0x1e3948], so the game's d carries that factor; the ± sums are EE add.s/sub.s.
        let a = crate::trig::ee_add(self.yaw, crate::trig::HALF_PI);
        let mul = nf_formats::ps2float::mul;
        let c0 = crate::trig::sinf(crate::trig::HALF_PI); // cos(0), 0x3f800001 [M 0x1e3948's φ lane]
        let d = [
            mul(mul(self.half_len, crate::trig::sinf(a)), c0),
            0.0,
            mul(mul(self.half_len, crate::trig::cosf(a)), c0),
        ];
        (
            [crate::trig::ee_add(self.pos[0], d[0]), self.pos[1], crate::trig::ee_add(self.pos[2], d[2])],
            [crate::trig::ee_sub(self.pos[0], d[0]), self.pos[1], crate::trig::ee_sub(self.pos[2], d[2])],
        )
    }

    /// The closest point of the axis segment to `p` (the game's
    /// `vecutil_calculate_closest_point_on_line` 0x196758:91-92 / 0x1969d0:106-107).
    pub fn closest_point(&self, p: V3) -> V3 {
        let (a, b) = self.endpoints();
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let l2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
        let t = if l2 > 0.0 {
            (((p[0] - a[0]) * ab[0] + (p[1] - a[1]) * ab[1] + (p[2] - a[2]) * ab[2]) / l2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        [a[0] + ab[0] * t, a[1] + ab[1] * t, a[2] + ab[2] * t]
    }
}

fn param(p: &nf_formats::placement::Placement, k: u32) -> u32 {
    p.params.iter().find(|(i, _)| *i == k).map_or(0, |&(_, v)| v)
}

/// Every creepwall placement of a level (`flags & 0xff == 0x3F`), mirroring the wire sweep
/// (`wire::all`) [M 0x1d04f0 case 0x3F → `CreepWall_Create` 0x196fc8]. The wall's pos is the
/// WORLD-SPHERE CENTRE (the local bound centre through the placement's yaw matrix [M
/// `Control_BuildWorldSph` 0x133070 — the same transform `wire::icons` mirrors]) and the half-length
/// the bound radius × |scale| [M 0x133070's `+0x8c` write, see the field doc]; the channel is
/// param 3 [M 0x196fc8 reads placement+0x2c/0x30/0x34/0x38 into BLData u16[0..3], and
/// `Player_Creep` reads `*(short *)(params + 6)` = u16[3] as the switch channel].
pub fn all(level: &nf_formats::level::Level) -> Vec<CreepWall> {
    let mut out = Vec::new();
    for m in level.maps.iter() {
        for p in &m.placements {
            if p.flags & 0xff != 0x3f { continue; }
            // The object's own bound centre + radius [M 0x133070 — see the field docs]. The
            // placement may reference the object by hashcode or by map index.
            let obj = if p.by_hash() {
                level.by_hash.get(&p.hashcode).and_then(|&(mp, oi)| level.maps.get(mp).and_then(|mm| mm.objects.get(oi)))
            } else {
                m.objects.get(p.index as usize)
            };
            let (centre, radius) = obj.map(|o| (o.centre, o.radius)).unwrap_or(([0.0; 3], 0.0));
            let yaw = p.rotation[1];
            // The world-sphere centre: the bound centre through the yaw-only matrix — the same
            // computation `wire::icons` mirrors [M Control_BuildWorldSph 0x133070 via 0x1c9680:40].
            let (s, c) = (crate::trig::sinf(yaw), crate::trig::cosf(yaw));
            let pos = [
                p.translation[0] + centre[0] * c + centre[2] * s,
                p.translation[1] + centre[1],
                p.translation[2] - centre[0] * s + centre[2] * c,
            ];
            let scale = p.scale();
            let s_max = scale.iter().fold(1.0f32, |a, &v| a * v.max(1e-6).abs());
            out.push(CreepWall {
                placement: u32::from(p.index),
                pos,
                yaw,
                half_len: if radius > 0.0 { radius * s_max } else { scale[0].abs().max(1.0) },
                ch: param(p, 3) as u16,
            });
        }
    }
    out
}
