//! The kill pieces of a broken 0x20 breakable: glass shards and debris objects
//! (docs/systems/break-debris.md). `Break_Kill` 0x12b970 spawns them at the kill — kind 2
//! (glass) shatters `psiGetTriList` 0x11c9d8's triangles through `ShatterPolyRecurse`
//! 0x12b270, every other kind runs `Debris_Create_Loop` 0x1ed548 over the collision block's
//! leaf boxes. The piece lists here are the sim side the draw layer renders each tick
//! (`UpdateAllShards` 0x12b808 / `DrawAllShards` 0x12bef8 / `Debris_Update` 0x1aac00).

use crate::drone::Rng;
use crate::grenade::REC_FRAME_RATE;
use crate::world::V3;

/// `WldGravity` (VECTOR at 0x2d8890): (0, −9.8, 0) [L grenade.rs:145, docs/systems/pickups.md].
const WLD_GRAVITY: V3 = [0.0, -9.8, 0.0];

/// One draw-model vertex the shard carries (the 64-byte runtime vertex's used parts):
/// pos at +0x10, ST/UV at +0x20, RGBA as floats at +0x28..+0x37 [M psiCreateBreakableData
/// 0x117418:90-156 — the 0x68/0x64/0x6e kicks fill exactly these slots].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShardVert {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
    pub rgba: [f32; 4],
}

/// One source triangle of the draw model (`psiCreateBreakableData`'s 12-byte face record +
/// its three vertices): strips emit `(i−2, i−1, i)` per kicking vertex ≥ 2 [M 0x117418:121-130],
/// `material` = the stream's last 0x3-tag word (the port's `Batch::texture`) [M 0x117418:175-177,
/// read at the leaf as `ShatterPolyRecurse`'s `param_5`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlassTri {
    pub v: [ShardVert; 3],
    pub material: u32,
}

/// One collision leaf record the debris loop consumes (8 bytes: 3 vertex halfwords + the plane
/// halfword — `crate::collision::CollTri`): the leaf-decoded positions [M 0x1ed548:114-125],
/// the stored plane normal (`s16 · 2⁻¹⁴`) [M 0x1ed548:132-139], the surface byte [D materials].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DebrisRec {
    pub v: [[f32; 3]; 3],
    pub n: [f32; 3],
    pub mat: u8,
}

/// What a breakable feeds its kill: the collision leaf records (the hit ray-test + the debris
/// source) and, for kind 2, the draw model's triangles (the shard source).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Source {
    pub records: Vec<DebrisRec>,
    pub glass: Vec<GlassTri>,
}

/// One `EffectInfo` row [D ACTION.ELF 0x3037c0 + k·0x88]: the debris dressing per hit material.
/// `+0x74` model hash, `+0x78` spread, `+0x7c` spin, `+0x81` lifetime base (the life is
/// `(base >> 1) + Rand_Rand(base >> 1)` ticks [M Debris_Create 0x1aaa00:34-37]). The +0x80 byte
/// is not read by `Debris_Create`. 23 entries — ASCII section headers follow entry 22 [D
/// 0x3043f8: "rtstr…"]. Entries with model 0 spawn nothing [M 0x1aaa00:19].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EffectInfo {
    pub model: u32,
    pub spread: f32,
    pub spin: f32,
    pub life_base: u8,
}

/// [D ACTION.ELF 0x3037c0..0x3043f8] (dumped 2026-10-20, scripts/dump_heinfo.py).
pub const EFFECT_INFO: [EffectInfo; 23] = [
    EffectInfo { model: 0, spread: 0.0, spin: 0.0, life_base: 0 },
    EffectInfo { model: 0, spread: 0.0, spin: 0.0, life_base: 0 },
    EffectInfo { model: 0, spread: 0.0, spin: 0.0, life_base: 0 },
    EffectInfo { model: 0x0200_012b, spread: 6.2, spin: 0.392_699_092_626_571_66, life_base: 0x37 },
    EffectInfo { model: 0x0200_012b, spread: 6.2, spin: 0.392_699_092_626_571_66, life_base: 0x37 },
    EffectInfo { model: 0x0200_01ff, spread: 6.2, spin: 0.392_699_092_626_571_66, life_base: 0x37 },
    EffectInfo { model: 0x0200_01ff, spread: 6.2, spin: 0.392_699_092_626_571_66, life_base: 0x37 },
    EffectInfo { model: 0x0200_012a, spread: 2.2, spin: 0.785_398_185_253_143_3, life_base: 0x37 },
    EffectInfo { model: 0, spread: 0.0, spin: 0.0, life_base: 0 },
    EffectInfo { model: 0x0200_0200, spread: 2.2, spin: 0.785_398_185_253_143_3, life_base: 0x37 },
    EffectInfo { model: 0x0200_012c, spread: 6.2, spin: 0.785_398_185_253_143_3, life_base: 0x37 },
    EffectInfo { model: 0x0200_01fe, spread: 2.2, spin: 0.785_398_185_253_143_3, life_base: 0x37 },
    EffectInfo { model: 0x0200_0220, spread: 6.2, spin: 0.392_699_092_626_571_66, life_base: 0x23 },
    EffectInfo { model: 0x0200_0220, spread: 6.2, spin: 0.392_699_092_626_571_66, life_base: 0x23 },
    EffectInfo { model: 0x0200_0220, spread: 6.2, spin: 0.392_699_092_626_571_66, life_base: 0x23 },
    EffectInfo { model: 0x0200_012d, spread: 2.2, spin: 0.785_398_185_253_143_3, life_base: 0x37 },
    EffectInfo { model: 0x0200_012d, spread: 6.2, spin: 3.141_592_741_012_573_2, life_base: 0x37 },
    EffectInfo { model: 0x0200_012c, spread: 6.2, spin: 0.785_398_185_253_143_3, life_base: 0x37 },
    EffectInfo { model: 0x0200_012c, spread: 6.2, spin: 3.141_592_741_012_573_2, life_base: 0x37 },
    EffectInfo { model: 0x0200_012a, spread: 2.2, spin: 0.785_398_185_253_143_3, life_base: 0x37 },
    EffectInfo { model: 0, spread: 0.0, spin: 0.0, life_base: 0 },
    EffectInfo { model: 0, spread: 0.0, spin: 0.0, life_base: 0 },
    EffectInfo { model: 0, spread: 0.0, spin: 0.0, life_base: 0 },
];

/// One glass shard — the game's 0xa0 record on `ShardList` 0x316dd8 [M ShatterPolyRecurse
/// 0x12b270 leaf, UpdateAllShards 0x12b808]. The verts are LOCAL (relative to `pos`, the
/// leaf's re-centring [M 0x12b270:164-174]); `pos` = `avg + obj+0x30` (the LOCAL mesh plus the
/// object's world position, no rotation — the same rule the debris uses [M 0x1ed548:114-131]).
#[derive(Clone, Debug, PartialEq)]
/// `InitShards` 0x12bed0 (40 B, from `ResetMap_GameInit` 0x1d5080:4-11 at 0x1d50e8, right after
/// `GS_PauseGame(0)`): `LList_Init(ShardList 0x316dd8, 0xa0, 0)` — the per-map empty of the
/// glass-shard pool (0xa0-byte records, count 0 = header zero only [M 0x12bed0 + LList_Init
/// 0x1cc958]). The port's `Shatter::default()` per level attach (breakable.rs, called from
/// `attach_level_id` drone.rs) is this reset — a fresh pool per map, no preallocation.
pub struct Shard {
    pub verts: [ShardVert; 3],
    /// +0x20, world.
    pub pos: V3,
    /// +0x30, accumulated `spin` (the draw builds the matrix from this + pos [M 0x12bef8:57]).
    pub rot: V3,
    /// +0x40, per-tick displacement.
    pub vel: V3,
    /// +0x50, per-tick rotation increment.
    pub spin: V3,
    /// +0x10 — `REC_FRAME_RATE · 25.0` at spawn, counted down by REC per active tick
    /// [M 0x12b270:205-209].
    pub lifetime: f32,
    /// +0x14 — `dist · REC_FRAME_RATE · 15.0` at spawn (the peel delay from the bullet hole)
    /// [M 0x12b270:190-192].
    pub delay: f32,
    /// +0x0c — the face's draw word (the draw's texture/material selector [M 0x12bef8:60]).
    pub material: u32,
}

/// One debris object — class-0x10 (`Debris_Create` 0x1aaa00), updated by `Debris_Update`
/// 0x1aac00. Fields carry the game's offsets.
#[derive(Clone, Debug, PartialEq)]
pub struct Debris {
    /// `EffectInfo+0x74` — the debris model hash [D table] set as the gfx entity [M 0x1aaa00:51].
    pub model: u32,
    /// obj+0x30, world (`avg(local verts) + obj+0x30` [M 0x1ed548:126-131]).
    pub pos: V3,
    /// obj+0x50, accumulated `spin` [M 0x1aac00:49-52].
    pub rot: V3,
    /// INFO+0x10 — `(spread·0.5 · record normal + hitvel·Rand_FRandHalf(spread)) · REC`; the
    /// hit-velocity term is zero (`Break_Kill` zeroes hitdata+0x20 [M 0x12b970:140]) but its
    /// three draws are consumed [M 0x1aaa00:38-49].
    pub vel: V3,
    /// INFO+0x20 — `Rand_FRandHalf_Vec(EffectInfo+0x7c)`, per axis `±spin/2` [M 0x1aaa00:50].
    pub spin: V3,
    /// INFO+0x30 — `(base>>1) + Rand_Rand(base>>1)` ticks; < 1 at a tick's entry → dead
    /// [M 0x1aac00:14].
    pub life: i32,
    /// obj+0x106 — 0xff at create, decremented by INFO+0x32 (= 1) per tick; exhausted → dead
    /// [M 0x1aaa00:31, 0x1aac00:21-24].
    pub fade: u8,
    /// obj+0xE8 — `Rand_FRand(0.1) + 0.4` [M 0x1aaa00:30-33]; not read by the sim update, kept
    /// for the draw layer.
    pub e8: f32,
}

/// The kill pieces of every breakable this session, oldest first. The draw layer reads
/// `shards`/`debris` directly.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Shatter {
    pub shards: Vec<Shard>,
    pub debris: Vec<Debris>,
}

fn avg3(v: [[f32; 3]; 3]) -> [f32; 3] {
    [(v[0][0] + v[1][0] + v[2][0]) / 3.0, (v[0][1] + v[1][1] + v[2][1]) / 3.0, (v[0][2] + v[1][2] + v[2][2]) / 3.0]
}

fn sq_dist(a: V3, b: V3) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
}

fn lerp_vert(a: &ShardVert, b: &ShardVert, t: f32) -> ShardVert {
    // `M = (v0 − v1)·t + v1` per component — pos, UV and RGBA all lerp [M 0x12b270:354-372].
    let l = |x: f32, y: f32| (x - y) * t + y;
    ShardVert {
        pos: [l(a.pos[0], b.pos[0]), l(a.pos[1], b.pos[1]), l(a.pos[2], b.pos[2])],
        uv: [l(a.uv[0], b.uv[0]), l(a.uv[1], b.uv[1])],
        rgba: [l(a.rgba[0], b.rgba[0]), l(a.rgba[1], b.rgba[1]), l(a.rgba[2], b.rgba[2]), l(a.rgba[3], b.rgba[3])],
    }
}

impl Shatter {
    /// `Break_Kill`'s kind-2 arm: `ShatterPolyRecurse` per source triangle. `trans` is the
    /// object's world position (obj+0x30), `hit` the contact position (hitdata+0x40), `normal`
    /// the contact's normalised normal (hitdata+0x30 [M 0x1ebaa8:391]).
    pub fn spawn_glass(&mut self, src: &[GlassTri], trans: V3, hit: V3, normal: V3, rng: &mut Rng) {
        for t in src {
            self.shatter_poly(*t, trans, hit, normal, rng);
        }
    }

    /// `ShatterPolyRecurse` 0x12b270 (the depth parameter is never read by the binary [M disasm
    /// 0x12b270-0x12b310] — the port omits it; the leaf rule terminates every split).
    fn shatter_poly(&mut self, tri: GlassTri, trans: V3, hit: V3, normal: V3, rng: &mut Rng) {
        let p = [tri.v[0].pos, tri.v[1].pos, tri.v[2].pos];
        let avg = avg3(p);
        let centre = [avg[0] + trans[0], avg[1] + trans[1], avg[2] + trans[2]];
        let dist_sq = sq_dist(hit, centre);
        let e01 = sq_dist(tri.v[0].pos, tri.v[1].pos);
        let e12 = sq_dist(tri.v[1].pos, tri.v[2].pos);
        let e20 = sq_dist(tri.v[2].pos, tri.v[0].pos);
        // Leaf when the longest edge² ≤ clamp(half·distSq, 0.5, 100) [M 0x12b270:156-162].
        let thresh = (dist_sq * 0.5).max(0.5).min(100.0);
        let longest = e01.max(e12).max(e20);
        if longest <= thresh {
            let dist = dist_sq.sqrt();
            let verts = tri.v.map(|v| ShardVert {
                pos: [v.pos[0] - avg[0], v.pos[1] - avg[1], v.pos[2] - avg[2]],
                uv: v.uv,
                rgba: v.rgba,
            });
            // spin = (Rand_FRand(1.0) − 0.5) / (distSq·10) per axis — three draws
            // [M 0x12b270:193-201].
            let d10 = dist_sq * 10.0;
            let spin = [(rng.frand(1.0) - 0.5) / d10, (rng.frand(1.0) - 0.5) / d10, (rng.frand(1.0) - 0.5) / d10];
            // vel = hitdata+0x30 · min(0.1, 0.125/dist) [M 0x12b270:199-211].
            let s = (0.125 / dist).min(0.1);
            let vel = [normal[0] * s, normal[1] * s, normal[2] * s];
            self.shards.push(Shard {
                verts,
                pos: centre,
                rot: [0.0; 3],
                vel,
                spin,
                lifetime: REC_FRAME_RATE * 25.0,
                delay: dist * REC_FRAME_RATE * 15.0,
                material: tri.material,
            });
            return;
        }
        // Split: rotate the longest edge onto v0–v1 [M 0x12b270:214-347 — branch 1 rebuilds the
        // list as [v1, v2, v0] (saved v0 lands in the v2 slot), branch 2 as [v2, v0, v1] (the
        // saved v0 lands in the v1 slot, 0x12b270:331-346)]. Splitting any other edge would
        // keep the longest one whole and never terminate.
        let (v0, v1, v2) = if e01.max(e20) <= e12 {
            (tri.v[1], tri.v[2], tri.v[0])
        } else if e01.max(e12) <= e20 {
            (tri.v[2], tri.v[0], tri.v[1])
        } else {
            (tri.v[0], tri.v[1], tri.v[2])
        };
        // M = lerp(v1 → v0, t), t = Rand_FRand(1.0)·0.5 + 0.25 — one draw [M 0x12b270:348-352].
        let t = rng.frand(1.0) * 0.5 + 0.25;
        let m = lerp_vert(&v0, &v1, t);
        self.shatter_poly(GlassTri { v: [v0, m, v2], material: tri.material }, trans, hit, normal, rng);
        self.shatter_poly(GlassTri { v: [m, v1, v2], material: tri.material }, trans, hit, normal, rng);
    }

    /// `Debris_Create_Loop` 0x1ed548 + `Debris_Create` 0x1aaa00 for one kill. `mat` is the
    /// contact's surface byte (hitdata+0x50, defaulted to 0x10 by `Break_Kill`); the records
    /// are the collision leaf records (already the game's stride-3 selection). Draws per
    /// record, in the game's order: FRand(0.1), Rand_Rand(half), FRandHalf(spread)×3,
    /// FRandHalf_Vec(spin)×3 [M 0x1aaa00:30-50] — eight shared-stream draws.
    pub fn spawn_debris(&mut self, recs: &[DebrisRec], trans: V3, mat: u8, rng: &mut Rng) {
        let Some(info) = EFFECT_INFO.get((mat & 0x3f) as usize) else { return };
        if info.model == 0 {
            return;
        }
        let half_life = (info.life_base >> 1) as u32;
        for r in recs {
            let avg = avg3(r.v);
            let centre = [avg[0] + trans[0], avg[1] + trans[1], avg[2] + trans[2]];
            let e8 = rng.frand(0.1) + 0.4;
            let life = half_life + rng.rand(half_life);
            // The zero hit-velocity term's draws: `hitdata+0x20·Rand_FRandHalf(spread)` with
            // hitdata+0x20 = 0 [M 0x12b970:140 + 0x1aaa00:38-49].
            let _ = rng.frand_half(info.spread);
            let _ = rng.frand_half(info.spread);
            let _ = rng.frand_half(info.spread);
            let s = info.spread * 0.5 * REC_FRAME_RATE;
            let vel = [r.n[0] * s, r.n[1] * s, r.n[2] * s];
            let spin = [rng.frand_half(info.spin), rng.frand_half(info.spin), rng.frand_half(info.spin)];
            self.debris.push(Debris { model: info.model, pos: centre, rot: [0.0; 3], vel, spin, life: life as i32, fade: 0xff, e8 });
        }
    }

    /// `UpdateAllShards` 0x12b808 + `Debris_Update` 0x1aac00, one fixed 60 Hz tick.
    pub fn update(&mut self) {
        for s in &mut self.shards {
            if s.lifetime > 0.0 {
                if s.delay <= 0.0 {
                    s.pos = [s.pos[0] + s.vel[0], s.pos[1] + s.vel[1], s.pos[2] + s.vel[2]];
                    s.rot = [s.rot[0] + s.spin[0], s.rot[1] + s.spin[1], s.rot[2] + s.spin[2]];
                    let g = REC_FRAME_RATE * REC_FRAME_RATE * 4.0;
                    s.vel = [s.vel[0] + WLD_GRAVITY[0] * g, s.vel[1] + WLD_GRAVITY[1] * g, s.vel[2] + WLD_GRAVITY[2] * g];
                    s.lifetime -= REC_FRAME_RATE;
                } else {
                    s.delay -= REC_FRAME_RATE;
                }
            }
        }
        self.shards.retain(|s| s.lifetime > 0.0);
        Self::update_debris_list(&mut self.debris);
    }

    /// `Debris_Update` 0x1aac00 over one debris list, one fixed 60 Hz tick — shared by the
    /// breakable path (`Shatter::update`) and the blast debris (class 0x10 objects the
    /// explosion's `Debris_CreateEx` 0x1ac808 spawns; explode-debris-spawn).
    pub fn update_debris_list(debris: &mut Vec<Debris>) {
        for d in debris.iter_mut() {
            if d.life < 1 || d.fade <= 1 {
                d.life = 0;
                continue;
            }
            d.fade -= 1;
            d.life -= 1;
            d.pos = [d.pos[0] + d.vel[0], d.pos[1] + d.vel[1], d.pos[2] + d.vel[2]];
            let g = REC_FRAME_RATE * REC_FRAME_RATE;
            d.vel = [d.vel[0] + WLD_GRAVITY[0] * g, d.vel[1] + WLD_GRAVITY[1] * g, d.vel[2] + WLD_GRAVITY[2] * g];
            d.rot = [d.rot[0] + d.spin[0], d.rot[1] + d.spin[1], d.rot[2] + d.spin[2]];
        }
        debris.retain(|d| d.life >= 1);
    }

    /// `DrawAllShards` 0x12bef8:50-56 — the shard alpha the draw layer renders:
    /// `min(lifetime · FRAME_RATE · 10, 255)`.
    pub fn shard_alpha(&self, s: &Shard) -> f32 {
        (s.lifetime * crate::sky::FRAME_RATE * 10.0).min(255.0)
    }
}
