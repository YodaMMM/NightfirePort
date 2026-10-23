//! The breakable scenery: placement 0x20 (`Break_Create` 0x12c230, `Break_Update` 0x12bfd8,
//! `Break_ApplyDamage` 0x12c128, `Break_Kill` 0x12b970).
//!
//! Semantics [M]:
//! - `Break_Create` 0x12c230: class-0xC object, `obj+0xFF = 0x20`; tag+0x00 = placement param 0
//!   (the break kind; 2 = glass), tag+0x04 = param 1 as f32 (the health; 0 dies on the first
//!   bullet), tag+0x08 = param 2 == 0 ? 0 : ~param2 as u16 (the bullet filter word).
//! - `Break_Update` 0x12bfd8: per tick (obj+0xFE bit 0 clear) the contact list is filtered with
//!   `Collide_FilterBullets(obj+0xD0, filter | 0x106)` and walked: a class-0x05 (bullet) contact
//!   subtracts the node's damage float from tag+0x04; a class-0x02 (drone) contact damages ONLY
//!   when `GameState+0x12 == 0x7000001` AND kind == 2 — the damage is
//!   `Vec_SqDist3D(contact+0x30, contact+0x40) · 20.0` (the drone's squared step this tick):
//!   guards walking through the castle's glass smash it. Health < 0 → `Break_Kill`.
//! - `Break_Kill` 0x12b970: `obj+0xFE |= 1` (dead); kind == 2 → sound 0x2E and
//!   `ShatterPolyRecurse` per triangle of `psiGetTriList(obj)`; otherwise the sound is 0x5F2 when
//!   `Rand_Rand(100) < 0x29` (41%) else 0x5F3, then `Drone_BrokenObject` + `Debris_Create_Loop`.
//!   NO channel is written anywhere [M 0x12b970 — no switch_channels store].
//!
//! The kill itself spawns the game's pieces through `crate::shatter` (`Shatter::spawn_glass` /
//! `spawn_debris`, docs/systems/break-debris.md): kind 2 shatters the draw model's triangles,
//! every other kind runs the debris loop over the collision block's leaf records — the sources
//! are built here at create (`Source`).
//!
//! Port deviations (documented, none tested as game behaviour):
//! - the pieces' draw presentation (matrices, models, look) is the renderer's work; the sim
//!   exposes the piece lists (`Shatter`) the draw layer reads (break-debris.md).
//! - the 41% sound variant needs the game's `Rand_Rand` draw-order context; the port keeps the
//!   deterministic 0x5F3 arm and does not consume the shared RNG (`// [S] unverified`).
//! - the filter word's weapon-bit layout is not evidenced (all 55 placements on 07000001 carry
//!   param 2 = 0 [D]); the port applies the mask as an exclude filter only when non-zero.

use crate::drone::Rng;
use crate::shatter::{self, DebrisRec, GlassTri, ShardVert, Source};
use crate::world::V3;
use nf_formats::level::Level;
use nf_formats::placement::{object_matrix, transform};

/// One kill effect for the caller to present (sound + shatter/debris) — `Break_Kill` 0x12b970's
/// observable outputs. The game plays the sound at obj+0x80 (the placed object's position).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BreakEffect {
    pub placement: usize,
    /// The kill position (the bounds centre — the port's stand-in for obj+0x80).
    pub pos: V3,
    /// tag+0x00: 2 = glass (shatter + sound 0x2E), other = debris (sound 0x5F3; the 41% 0x5F2
    /// variant is `// [S] unverified` — see the module header).
    pub kind: u8,
    pub sound: u16,
}

#[derive(Clone, Debug)]
pub struct Break {
    pub placement: usize,
    pub lo: V3,
    pub hi: V3,
    /// tag+0x00 — placement param 0's low byte [M 0x12c230:21 `*puVar1 = *(param_4 + 0x2c)`].
    pub kind: u8,
    /// tag+0x04 — (f32)(i32) placement param 1 [M 0x12c230 disasm 0x12c2a8-0x12c2e0 cvt.s.w].
    pub health: f32,
    /// tag+0x08 — param 2 == 0 ? 0 : !(param 2) as u16 [M 0x12c230:31-39].
    pub filter: u16,
    /// obj+0xFE bit 0 — set by Break_Kill [M 0x12b970], gates Break_Update [M 0x12bfd8:14].
    pub dead: bool,
    /// The placement transform (`object_matrix`) — the draw matrix while alive; the renderer
    /// stops drawing a dead one (the movement handler's delete pass removes the object
    /// [M 0x12b970:41 — obj+0xFE bit 0]).
    pub matrix: [[f32; 4]; 4],
    /// The kill-piece sources (shatter::Source): the collision leaf records (every kind) and
    /// the draw model's triangles (kind 2). Built at create; empty records = the silent kill
    /// [M 0x12b970:61-63].
    pub src: Source,
}

#[derive(Clone, Debug, Default)]
pub struct Breaks {
    pub list: Vec<Break>,
    /// The kill pieces (crate::shatter) — `Break_Kill`'s spawns, updated per tick; the draw
    /// layer reads them (docs/systems/break-debris.md).
    pub shatter: shatter::Shatter,
}

/// One placement's target object, resolved like `bounds` does.
fn resolve<'a>(level: &'a Level, m: usize, p: &nf_formats::placement::Placement) -> Option<&'a nf_formats::level::MapObject> {
    let obj = if p.by_hash() { level.by_hash.get(&p.hashcode).map(|&(map, object)| &level.maps[map].objects[object]) }
        else { level.maps[m].objects.get(p.index as usize) };
    obj
}

/// One placement's target object's bounds placed by the placement transform (same rule as
/// crate::props::bounds — the glist bounds the collision pass would register).
fn bounds(level: &Level, m: usize, p: &nf_formats::placement::Placement) -> (V3, V3) {
    let Some(o) = resolve(level, m, p) else { return ([0.0; 3], [0.0; 3]) };
    let mx = object_matrix(p.rotation, p.translation);
    let mut corners = Vec::new();
    let sane = [o.min, o.max].iter().all(|b| b.iter().all(|v| v.abs() < 1.0e6));
    if sane {
        for x in [o.min[0], o.max[0]] { for y in [o.min[1], o.max[1]] { for z in [o.min[2], o.max[2]] {
            corners.push([x, y, z]);
        } } }
    } else if let Some(c) = o.collision.as_ref() {
        for t in c.triangles() { for v in t { corners.push(v); } }
    }
    if corners.is_empty() { return ([0.0; 3], [0.0; 3]) }
    let (mut lo, mut hi) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
    for v in corners {
        let v = transform(&mx, v);
        for k in 0..3 { lo[k] = lo[k].min(v[k]); hi[k] = hi[k].max(v[k]); }
    }
    (lo, hi)
}

/// The kill-piece sources of one placed object (shatter::Source):
/// - the collision block's leaf records, stride 3 [M Debris_Create_Loop 0x1ed548 + disasm
///   0x1ed638 — `for (i = start; i < end; i += 3)`], local verts = `origin + s16·scale`
///   [M 0x1ed548:114-125], the stored plane normal (`s16·2⁻¹⁴`) [M 0x1ed548:132-139], the
///   surface byte [D materials];
/// - for kind 2, the draw model's triangles [M psiCreateBreakableData 0x117418 — the strip
///   faces `(i−2, i−1, i)` per kicking vertex ≥ 2, verts pos/uv/rgba, the batch's texture as
///   the face's draw word].
fn source(o: &nf_formats::level::MapObject, kind: u8) -> Source {
    let mut records = Vec::new();
    if let Some(c) = o.collision.as_ref() {
        for bx in c.boxes.iter().filter(|b| b.children.is_none()) {
            let base = bx.vblock as usize * 0x20;
            let mut i = bx.tris.start as usize;
            while i < bx.tris.end as usize {
                let t = &c.tris[i];
                let v = t.v.map(|vi| {
                    let w = |k: usize| c.words.get(base + vi as usize + k).copied().unwrap_or(0) as f32;
                    [w(0) * bx.scale + bx.origin[0], w(1) * bx.scale + bx.origin[1], w(2) * bx.scale + bx.origin[2]]
                });
                let p = t.plane as usize;
                let pw = |k: usize| c.words.get(base + p + k).copied().unwrap_or(0) as f32 * (1.0 / 16384.0);
                let n = [pw(0), pw(1), pw(2)];
                let mat = c.materials.get(i).copied().unwrap_or(0);
                records.push(DebrisRec { v, n, mat });
                i += 3;
            }
        }
    }
    let mut glass = Vec::new();
    if kind == 2 {
        for leaf in o.gfx.leaves.iter() {
            for batch in leaf.batches.iter() {
                let material = batch.texture.unwrap_or(0);
                let rgba = |i: usize| batch.verts[i].rgba.map(f32::from);
                for idx in (2..batch.verts.len()).filter(|&i| batch.kicks(i)) {
                    let v = |k: usize| ShardVert {
                        pos: batch.verts[idx - k].pos,
                        uv: batch.verts[idx - k].uv,
                        rgba: rgba(idx - k),
                    };
                    glass.push(GlassTri { v: [v(2), v(1), v(0)], material });
                }
            }
        }
    }
    Source { records, glass }
}

impl Breaks {
    /// `Break_Create` over the main map's 0x20 placements (placement order, like the game's
    /// `parsemap_create_dynamic_objects` case 0x20 [M 0x1d04f0]).
    pub fn create(level: &Level) -> Breaks {
        let Some(m) = level.main_map() else { return Breaks::default() };
        let mut list = Vec::new();
        for (i, p) in level.maps[m].placements.iter().enumerate() {
            if p.flags & 0xff != 0x20 { continue; }
            let param = |k: u32| p.params.iter().find(|(j, _)| *j == k).map_or(0, |&(_, v)| v);
            let (lo, hi) = bounds(level, m, p);
            let p2 = param(2);
            let kind = (param(0) & 0xff) as u8;
            let src = resolve(level, m, p).map_or(Source::default(), |o| source(o, kind));
            list.push(Break {
                placement: i,
                lo,
                hi,
                kind,
                // `Break_Create` 0x12c230 disasm 0x12c2a8-0x12c2e0: `lw 0x30($s2)` → `mtc1` →
                // `cvt.s.w` → `swc1 4($s1)` (the negative branch is the same cvt through the
                // unsigned-double trick) — the health is (f32)(i32) param 1, NOT the bit
                // pattern: 07000009's pillarglass #123 authored 6 [D placement params] is 6.0.
                health: (param(1) as i32) as f32,
                // `Break_Create` 0x12c230 disasm 0x12c2e4-0x12c2f8: param 2 == 0 → 0, else
                // `lhu` + `nor` (~param2) → `sh 8($s1)` — the datum word `Break_Update` ORs
                // 0x106 into [M 0x12c028-0x12c030].
                filter: if p2 == 0 { 0 } else { !(p2 as u16) },
                dead: false,
                matrix: object_matrix(p.rotation, p.translation),
                src,
            });
        }
        Breaks { list, shatter: shatter::Shatter::default() }
    }

    fn seg_box(a: V3, b: V3, lo: V3, hi: V3) -> bool {
        // Same rule as crate::props::seg_box: the bullet segment's AABB against the bounds grown
        // by 0.1 (the contact registers on the sphere touching the glist bounds).
        Self::seg_box_grow(a, b, lo, hi, 0.1)
    }

    fn seg_box_grow(a: V3, b: V3, lo: V3, hi: V3, g: f32) -> bool {
        let (_, mut mn, mut mx) = (g, [0.0; 3], [0.0; 3]);
        for k in 0..3 {
            mn[k] = a[k].min(b[k]);
            mx[k] = a[k].max(b[k]);
            if mx[k] < lo[k] - g || mn[k] > hi[k] + g { return false; }
        }
        true
    }

    /// The bullet flight this tick against the breakables ([M Break_Update 0x12bfd8:44-48 — the
    /// class-5 contact subtracts its damage]). `weapon_mask` is the bullet's weapon-DEFINITION
    /// word at `weapon_definition+0x10` (`lhu 0x10($a0)` in `Collide_FilterBullets` [M
    /// 0x1ece94]; the caller reads `weapons.records[weapon][0x10..0x12]`), not the weapon id.
    ///
    /// The filter [M `Collide_FilterBullets` 0x1ece30, called from `Break_Update` 0x12bfd8:15
    /// with mask = tag+0x08 | 0x106 (disasm 0x12c028-0x12c030)]: a class-5 contact whose word
    /// ANDs the mask non-zero is marked t = 1e8 (0x4cbebc20, disasm 0x1ecea4) and the rebuild
    /// hands every t == 1e8 node to `LList_Add` — OUT of the contact list (disasm 0x1ecee8
    /// `c.2.s f0,f20` / 0x1ecef0 `bc1f` keep / 0x1ecef8 `jal LList_Add` drop); only the
    /// zero-AND contacts remain for `Break_Update`'s damage walk. So a word with bits 0x02 /
    /// 0x04 / 0x100 (the laser 0x0002, the watch 0x0100 [L ee_start_05.bin weapon_data
    /// +0x10: 0x4e/0x4f = 0x0002, 0x54 = 0x0100]) never damages even a no-filter pane, while
    /// the guns' 0x0008 (0x2/0x43/… [L same dump]) does; a pane's own param 2 shrinks the
    /// passing set further (07000009's securityroom_glass param 2 = 0x30 → mask 0xffef:
    /// nothing with ammo-word bits passes [D #344]).
    /// Returns the kills for presentation; each kill spawns its pieces (crate::shatter).
    pub fn bullet_hit(&mut self, a: V3, b: V3, weapon_mask: u16, damage: f32, rng: &mut Rng) -> Vec<BreakEffect> {
        let mut hits = Vec::new();
        for (i, br) in self.list.iter_mut().enumerate().filter(|(_, b)| !b.dead) {
            // The filter's drop polarity [M 0x1ece30:24-33 + 0x1ecee4-0x1ecef8]: non-zero AND
            // = the contact never reaches the damage walk.
            if weapon_mask & (br.filter | 0x106) != 0 { continue; }
            if Self::seg_box(a, b, br.lo, br.hi) {
                br.health -= damage;
                if br.health < 0.0 {
                    hits.push((i, a, b));
                }
            }
        }
        hits.into_iter().filter_map(|(i, a, b)| self.kill(i, a, b, rng)).collect()
    }

    /// The drone-contact arm ([M Break_Update 0x12bfd8:23-34 — only on GameState+0x12 ==
    /// 0x7000001, only kind == 2, damage = the drone's squared step × 20]). `level_01` is the
    /// caller's `GameState+0x12 == 0x07000001` check. `scripted_walk`: this tick's position
    /// change was carried by the anim root motion (the port's `Drone::scripted_move`) —
    /// glass-breach-guards-4's gate, the last paragraph.
    ///
    /// The contact stands in for the game's collision pair ([M 0x12bfd8:15 — the pane's obj+0xD0
    /// list the collision pass fills): the pair exists when the body's movement CARRIES IT INTO
    /// the pane's collision sheet — this tick's step segment crosses the pane's bounds. A body
    /// whose approach merely closes distance (a guard walking up to the glass, or standing near
    /// it) has no pair until the collision engages; the port's pane bake stops bodies at the
    /// sheet, so the crossing IS the engagement. The previous 0.55-grown hull zone fired ~0.6
    /// before any physical contact — a proximity kill the game's list cannot produce (a pressed
    /// guard's position is collision-owned: zero net step, zero damage [M the SqDist rule]).
    ///
    /// **The scripted-walk gate (glass-breach-guards-4).** The drain is `Vec_SqDist3D(hitter+0x30,
    /// hitter+0x40)` — the hitter's current vs previous position. The movement pass copies
    /// +0x30 → +0x40 per object AFTER that object's anim/root-motion processing in the same walk
    /// [M 0x1336f8:212 — the copy sits behind the per-object rotation/world-sphere work], so a
    /// step carried by the anim root motion lands in BOTH +0x30 and +0x40 before the copy — it
    /// reads 0 and never drains the pane. The move that shows in the pair is the AI steering
    /// step. This is the [D] behaviour on 07000001: the placed guard g0's replayed intro script
    /// (his state-3 walk-by, which the port demonstrably carries across pane 130's sheet at
    /// 0xB+23) leaves the pane whole [D the observed run — pane 130 alive until the breach at
    /// 0xB+911], while the breach squad's AI charge shatters it [D 0xB+911]. The held branch's
    /// failure was exactly this: the replayed scripted walk killed 01's breakables nobody shot
    /// (interact_07000001_live.rs:171).
    pub fn drone_step(&mut self, drone_from: V3, drone_to: V3, level_01: bool, scripted_walk: bool, rng: &mut Rng) -> Vec<BreakEffect> {
        let mut kills = Vec::new();
        if !level_01 || scripted_walk { return kills; }
        let sq = (drone_to[0] - drone_from[0]).powi(2)
            + (drone_to[1] - drone_from[1]).powi(2)
            + (drone_to[2] - drone_from[2]).powi(2);
        if sq == 0.0 { return kills; }
        let dmg = sq * 20.0;
        let mut hits = Vec::new();
        for (i, br) in self.list.iter_mut().enumerate().filter(|(_, b)| !b.dead && b.kind == 2) {
            // The contact: this tick's step crosses the pane's bounds (the collision pair).
            if Self::seg_box_grow(drone_from, drone_to, br.lo, br.hi, 0.0) {
                br.health -= dmg;
                if br.health < 0.0 {
                    hits.push((i, drone_from, drone_to));
                }
            }
        }
        kills = hits.into_iter().filter_map(|(i, a, b)| self.kill(i, a, b, rng)).collect();
        kills
    }

    /// `Break_Kill` 0x12b970 for one breakable: the dead bit, then — only when the object has
    /// collision records (the cel+8 collision block [M 0x12b970:59-63]; zero → the silent
    /// kill: no sound, no pieces) — the pieces. The contact triple comes from a ray-test of
    /// the segment against the records (the game's contact is its collision pass's [M
    /// Intersect_RayGeom 0x10a8c8 — the same plane-backface skip, the surface byte →
    /// hitdata+0x50]); no crossing falls back to the bounds centre and the first record's
    /// plane [*port choice*]. Kind 2 shatters the draw triangles, every other kind spawns the
    /// debris records with `EffectInfo[material & 0x3F]` [M 0x12b970:78-141].
    pub fn kill(&mut self, i: usize, a: V3, b: V3, rng: &mut Rng) -> Option<BreakEffect> {
        let br = &mut self.list[i];
        if br.dead { return None; }
        br.dead = true;
        if br.src.records.is_empty() { return None; }
        let trans = [br.matrix[3][0], br.matrix[3][1], br.matrix[3][2]];
        let centre = [(br.lo[0] + br.hi[0]) / 2.0, (br.lo[1] + br.hi[1]) / 2.0, (br.lo[2] + br.hi[2]) / 2.0];
        let (hit, normal, mat) = Self::contact(&br.src.records, a, b, trans, centre);
        // Break_Kill defaults the hit material's low 6 bits to 0x10 when zero — the debris
        // table's breakable row [M 0x12b970:137-138 `if ((hitdata+0x50 & 0x3f) == 0) = 0x10`].
        let mat = if mat & 0x3f == 0 { 0x10 } else { mat };
        if br.kind == 2 {
            self.shatter.spawn_glass(&br.src.glass, trans, hit, normal, rng);
        } else {
            self.shatter.spawn_debris(&br.src.records, trans, mat, rng);
        }
        Some(Self::effect(&self.list[i]))
    }

    /// The kill contact: the closest record the segment crosses (Möller–Trumbore, local space —
    /// the game's debris/shard math is local verts + the object position [M 0x1ed548]), with
    /// `Intersect_RayGeom`'s plane skip: a record whose plane normal faces along the ray
    /// (`dot(n, dir) ≥ 0`) is skipped [M 0x10a8c8, docs/formats/collision.md].
    fn contact(recs: &[DebrisRec], a: V3, b: V3, trans: V3, fallback: V3) -> (V3, V3, u8) {
        let la = [a[0] - trans[0], a[1] - trans[1], a[2] - trans[2]];
        let lb = [b[0] - trans[0], b[1] - trans[1], b[2] - trans[2]];
        let dir = [lb[0] - la[0], lb[1] - la[1], lb[2] - la[2]];
        let mut best: Option<(f32, usize)> = None;
        for (ri, r) in recs.iter().enumerate() {
            if r.n[0] * dir[0] + r.n[1] * dir[1] + r.n[2] * dir[2] >= 0.0 { continue; }
            let e1 = [r.v[1][0] - r.v[0][0], r.v[1][1] - r.v[0][1], r.v[1][2] - r.v[0][2]];
            let e2 = [r.v[2][0] - r.v[0][0], r.v[2][1] - r.v[0][1], r.v[2][2] - r.v[0][2]];
            let pv = [dir[1] * e2[2] - dir[2] * e2[1], dir[2] * e2[0] - dir[0] * e2[2], dir[0] * e2[1] - dir[1] * e2[0]];
            let det = e1[0] * pv[0] + e1[1] * pv[1] + e1[2] * pv[2];
            if det.abs() < 1.0e-9 { continue; }
            let inv = 1.0 / det;
            let tv = [la[0] - r.v[0][0], la[1] - r.v[0][1], la[2] - r.v[0][2]];
            let u = (tv[0] * pv[0] + tv[1] * pv[1] + tv[2] * pv[2]) * inv;
            if !(0.0..=1.0).contains(&u) { continue; }
            let qv = [tv[1] * e1[2] - tv[2] * e1[1], tv[2] * e1[0] - tv[0] * e1[2], tv[0] * e1[1] - tv[1] * e1[0]];
            let v = (dir[0] * qv[0] + dir[1] * qv[1] + dir[2] * qv[2]) * inv;
            if v < 0.0 || u + v > 1.0 { continue; }
            let t = (e2[0] * qv[0] + e2[1] * qv[1] + e2[2] * qv[2]) * inv;
            if !(0.0..=1.0).contains(&t) { continue; }
            if best.map_or(true, |(bt, _)| t < bt) {
                best = Some((t, ri));
            }
        }
        match best {
            Some((t, ri)) => {
                let r = &recs[ri];
                let hit = [la[0] + dir[0] * t + trans[0], la[1] + dir[1] * t + trans[1], la[2] + dir[2] * t + trans[2]];
                let l = (r.n[0] * r.n[0] + r.n[1] * r.n[1] + r.n[2] * r.n[2]).sqrt();
                let normal = if l > 0.0 { [r.n[0] / l, r.n[1] / l, r.n[2] / l] } else { [0.0, 1.0, 0.0] };
                (hit, normal, r.mat)
            }
            None => {
                // No crossing in the records: the bounds centre and the first record's plane.
                let r = &recs[0];
                let l = (r.n[0] * r.n[0] + r.n[1] * r.n[1] + r.n[2] * r.n[2]).sqrt();
                let normal = if l > 0.0 { [r.n[0] / l, r.n[1] / l, r.n[2] / l] } else { [0.0, 1.0, 0.0] };
                (fallback, normal, r.mat)
            }
        }
    }

    fn effect(br: &Break) -> BreakEffect {
        let pos = [(br.lo[0] + br.hi[0]) / 2.0, (br.lo[1] + br.hi[1]) / 2.0, (br.lo[2] + br.hi[2]) / 2.0];
        let sound = if br.kind == 2 { 0x2E } else { 0x5F3 };
        BreakEffect { placement: br.placement, pos, kind: br.kind, sound }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shatter::EFFECT_INFO;

    /// The bullet_hit weapon argument is the weapon-DEFINITION word at +0x10 (the game loads it
    /// `lhu 0x10($a0)` [M 0x1ece94]); the disc's gun records carry 0x0008 [L ee_start_05.bin
    /// weapon_data @0x2bf150 + w·0x10c: 0x2 (the P2K) = 0x0008], the laser 0x0002, the watch
    /// 0x0100.
    const GUN: u16 = 0x0008;

    /// One breakable with one collision record — a triangle across the test bullet's path
    /// [(5,8) in XY, at z = −2.5] facing the shooter (−Z). Kills play the sound and spawn one
    /// debris piece (EffectInfo[0x10] — model 0x0200012d [D 0x3037c0+0x10·0x88]).
    fn one(kind: u8, health: f32) -> Breaks {
        Breaks {
            list: vec![Break {
                placement: 0,
                lo: [4.0, 7.0, -3.0],
                hi: [6.0, 9.0, -1.0],
                kind,
                health,
                filter: 0,
                dead: false,
                matrix: [[0.0; 4]; 4],
                src: Source {
                    records: vec![DebrisRec { v: [[4.5, 7.5, -2.5], [5.5, 7.5, -2.5], [5.0, 8.5, -2.5]], n: [0.0, 0.0, -1.0], mat: 0x10 }],
                    glass: Vec::new(),
                },
            }],
            shatter: Default::default(),
        }
    }

    #[test]
    fn the_first_bullet_kills_a_zero_health_vase_with_the_debris_sound() {
        // The 07000001 vases carry params [(2,0)] only — kind 0, health 0.0 [D placements dump].
        let mut b = one(0, 0.0);
        let mut rng = Rng::new(1);
        let kills = b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], GUN, 25.0, &mut rng);
        assert!(b.list[0].dead, "health 0 - damage < 0 → Break_Kill [M 0x12bfd8:44-50]");
        assert_eq!(kills.len(), 1);
        assert_eq!(kills[0].sound, 0x5F3, "non-glass → 0x5F3 (the 41% 0x5F2 variant is unported, [S])");
        assert_eq!(kills[0].kind, 0);
        // The kill spawned the game's debris: one record → one piece, EffectInfo[0x10].
        assert_eq!(b.shatter.debris.len(), 1, "Debris_Create_Loop: one object per stride-3 record [M 0x1ed548]");
        assert_eq!(b.shatter.debris[0].model, EFFECT_INFO[0x10].model, "EffectInfo[hitdata+0x50 & 0x3f = 0x10] [M 0x12b970:137-141]");
    }

    #[test]
    fn glass_shatters_with_sound_0x2e_and_the_sister_vase_survives_a_miss() {
        // radioroom_glass/comroom_glass carry params [(0,2),(1,0),(2,0)] — kind 2 [D placements].
        let mut b = one(2, 0.0);
        let mut rng = Rng::new(1);
        let kills = b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], GUN, 25.0, &mut rng);
        assert_eq!(kills[0].sound, 0x2E, "glass → sound 0x2E [M 0x12b970: uVar5 = 0x2e]");
        // Sister: a shot wide of the box never touches the health and spawns nothing.
        let mut b = one(2, 0.0);
        let mut rng = Rng::new(1);
        let kills = b.bullet_hit([50.0, 8.0, -4.0], [50.0, 8.0, -2.0], GUN, 25.0, &mut rng);
        assert!(kills.is_empty() && !b.list[0].dead, "a miss never damages the breakable");
        assert!(b.shatter.debris.is_empty() && b.shatter.shards.is_empty(), "an unshot breakable spawns nothing");
    }

    #[test]
    fn a_health_pool_takes_repeated_hits_and_only_dies_below_zero() {
        // Destroy_test's rule: exactly 0.0 is NOT a kill [M 0x12bfd8:50 `fVar5 < 0.0`].
        let mut b = one(0, 50.0);
        let mut rng = Rng::new(1);
        for i in 0..5 {
            let kills = b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], GUN, 10.0, &mut rng);
            assert!(kills.is_empty() && !b.list[0].dead, "hit {}: {} left", i, 50 - 10 * (i + 1));
        }
        let kills = b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], GUN, 10.0, &mut rng);
        assert_eq!(kills.len(), 1, "the 6th crosses below zero");
    }

    #[test]
    fn the_laser_and_watch_words_never_reach_a_no_filter_pane() {
        // `Collide_FilterBullets` [M 0x1ece30:24-33 + 0x1ecee4-0x1ecef8]: a class-5 contact
        // whose weapon word ANDs `filter | 0x106` non-zero is dropped from the list — on a
        // no-filter pane the base 0x106 (bits 0x2/0x4/0x100) rejects the laser's 0x0002 and
        // the watch's 0x0100 [L ee_start_05.bin weapon_data +0x10: 0x4e/0x4f = 0x0002,
        // 0x54 = 0x0100], while the guns' 0x0008 passes.
        let seg = ([5.0, 8.0, -4.0], [5.0, 8.0, -2.0]);
        for (name, word) in [("laser", 0x0002u16), ("watch", 0x0100u16)] {
            let mut b = one(2, 50.0);
            let mut rng = Rng::new(1);
            let kills = b.bullet_hit(seg.0, seg.1, word, 25.0, &mut rng);
            assert!(kills.is_empty() && !b.list[0].dead && (b.list[0].health - 50.0).abs() < 1e-6,
                "the {name} word {word:#06x} never damages the pane");
            assert!(b.shatter.debris.is_empty() && b.shatter.shards.is_empty(), "{name}: no pieces");
        }
        let mut b = one(2, 0.0);
        let mut rng = Rng::new(1);
        let kills = b.bullet_hit(seg.0, seg.1, GUN, 25.0, &mut rng);
        assert_eq!(kills.len(), 1, "the gun word 0x0008 passes the 0x106 base mask");
    }

    #[test]
    fn the_security_room_filter_word_rejects_gunfire_but_takes_a_zero_word() {
        // 07000009's securityroom_glass #344 carries param 2 = 0x30 [D placement params] →
        // `Break_Create` stores ~0x30 = 0xffcf [M 0x12c2e4-0x12c2f8] and `Break_Update` ORs
        // 0x106 → mask 0xffef [M 0x12c028-0x12c030]: a gun's 0x0008 ANDs non-zero → dropped;
        // only a zero word (the grenades' [L ee_start_05.bin: 0x2c..0x31 = 0x0000]) passes.
        let mut b = one(2, 50.0);
        b.list[0].filter = !0x30u16;
        let mut rng = Rng::new(1);
        let kills = b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], GUN, 25.0, &mut rng);
        assert!(kills.is_empty() && !b.list[0].dead, "gunfire is filtered out of the security glass");
        let mut b = one(2, 0.0);
        b.list[0].filter = !0x30u16;
        let mut rng = Rng::new(1);
        let kills = b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], 0x0000, 25.0, &mut rng);
        assert_eq!(kills.len(), 1, "a zero word (the explosive class) passes 0xffef");
    }

    #[test]
    fn a_guard_walking_through_glass_smashes_it_only_on_level_01() {
        // [M Break_Update 0x12bfd8:23-34: GameState+0x12 == 0x7000001 and kind == 2 gate the
        // drone arm; damage = squared step × 20 — a 0.5-step guard crosses 5.0, a vase (kind 0)
        // never takes drone damage.]
        let mut b = one(2, 0.0);
        let mut rng = Rng::new(1);
        let kills = b.drone_step([5.0, 8.0, -1.5], [5.3, 8.0, -1.5], true, false, &mut rng);
        assert_eq!(kills.len(), 1, "0.09 · 20 = 1.8 damage kills the 0-health glass");
        let mut v = one(0, 0.0);
        let mut rng = Rng::new(1);
        assert!(v.drone_step([5.0, 8.0, -1.5], [5.3, 8.0, -1.5], true, false, &mut rng).is_empty(),
            "kind != 2 never takes drone damage [M 0x12bfd8:31-34]");
        let mut g = one(2, 0.0);
        let mut rng = Rng::new(1);
        assert!(g.drone_step([5.0, 8.0, -1.5], [5.3, 8.0, -1.5], false, false, &mut rng).is_empty(),
            "off 07000001 the drone arm is dead [M 0x12bfd8:24]");
    }

    #[test]
    fn a_scripted_walk_by_never_drains_the_pane_and_the_sister_ai_step_still_does() {
        // glass-breach-guards-4 [D 750p + M 0x1336f8:212]: the placed guard g0's replayed intro
        // script walks him across pane 130's sheet at 0xB+23 in the port, yet the game's pane
        // survives until the breach at 0xB+911 — the root-motion-carried step lands in BOTH
        // +0x30 and +0x40 (the movement pass copies +0x30 → +0x40 after the anim processing in
        // the same walk), so SqDist(pos, prev) = 0 and the drain never fires. The SAME crossing
        // by an AI-steered mover (the breach squad's charge) drains [M 0x12bfd8:26-27].
        let mut b = one(2, 0.0);
        let mut rng = Rng::new(1);
        assert!(b.drone_step([5.0, 8.0, -1.5], [5.3, 8.0, -1.5], true, true, &mut rng).is_empty(),
            "the scripted (root-motion-carried) walk-by reads SqDist(pos, prev) = 0 [M 0x1336f8:212]");
        assert!(!b.list[0].dead, "pane 130 survives g0's scripted walk-by [D the observed run]");
        let kills = b.drone_step([5.0, 8.0, -1.5], [5.3, 8.0, -1.5], true, false, &mut rng);
        assert_eq!(kills.len(), 1, "the same crossing by an AI-steered mover drains [M 0x12bfd8:26-27]");
    }

    #[test]
    fn a_kill_without_collision_records_is_silent_and_spawns_nothing() {
        // Break_Kill returns after the dead bit when cel+8 (the collision block) is zero — no
        // sound, no shatter, no debris [M 0x12b970:59-63].
        let mut b = one(0, 0.0);
        b.list[0].src.records.clear();
        let mut rng = Rng::new(1);
        let kills = b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], GUN, 25.0, &mut rng);
        assert!(b.list[0].dead, "the dead bit is still set [M 0x12b970:60]");
        assert!(kills.is_empty(), "no effect (no sound) [M 0x12b970:61-63]");
        assert!(b.shatter.debris.is_empty() && b.shatter.shards.is_empty(), "no pieces");
    }

    #[test]
    fn the_debris_piece_carries_the_games_launch_numbers() {
        // Debris_Create [M 0x1aaa00]: velocity = (spread·0.5 · record normal) · REC (the
        // hit-velocity term is zeroed by Break_Kill [M 0x12b970:140]); life =
        // (base>>1) + Rand_Rand(base>>1); fade 0xff; spin = ±EffectInfo+0x7c/2 via
        // Rand_FRandHalf_Vec — eight draws per piece [M 0x1aaa00:30-50].
        let mut b = one(0, 0.0);
        let mut rng = Rng::new(0x5eed);
        let draws_before = rng.draws;
        b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], GUN, 25.0, &mut rng);
        let d = &b.shatter.debris[0];
        let info = EFFECT_INFO[0x10];
        let speed = info.spread * 0.5 * crate::grenade::REC_FRAME_RATE;
        let n = [0.0f32, 0.0, -1.0]; // the record's plane normal, unit
        for k in 0..3 {
            let want = n[k] * speed;
            assert!((d.vel[k] - want).abs() < 1.0e-5, "vel[{}] = {} want {}", k, d.vel[k], want);
        }
        let half = (info.life_base >> 1) as u32;
        assert!(d.life >= half as i32 && d.life < (half * 2) as i32, "life = half + Rand_Rand(half) [M 0x1aaa00:34-37]");
        assert_eq!(d.fade, 0xff, "obj+0x106 = 0xff at create [M 0x1aaa00:31]");
        for k in 0..3 {
            assert!(d.spin[k] >= -info.spin / 2.0 && d.spin[k] < info.spin / 2.0,
                "spin[{}] in ±spin/2 (Rand_FRandHalf_Vec [M 0x1dfef0])", k);
        }
        assert_eq!(rng.draws - draws_before, 8, "eight shared draws per debris object [M 0x1aaa00:30-50]");
        // The centre: avg(local verts) + the object position [M 0x1ed548:126-131] — the ray hit
        // at (5, 8, −2.5) is not the piece centre; the piece centre is the record triangle's
        // average (5.0, 7.833…, −2.5).
        assert!((d.pos[0] - 5.0).abs() < 1.0e-5 && (d.pos[1] - 23.5 / 3.0).abs() < 1.0e-5 && (d.pos[2] + 2.5).abs() < 1.0e-5,
            "centre = avg(verts) + trans [M 0x1ed548:126-131]");
    }

    #[test]
    fn a_shot_window_spawns_one_shard_per_source_triangle_and_the_sister_unshot_spawns_none() {
        // The glass path: ShatterPolyRecurse per psiGetTriList triangle [M 0x12b970:79-131].
        // Small source triangles at the hit's distance spawn as leaves (longest edge² ≤
        // clamp(half·distSq, 0.5, 100) [M 0x12b270:156-162]) — one shard each.
        let mut b = one(2, 0.0);
        let mk = |z: f32| GlassTri {
            v: [
                ShardVert { pos: [4.5, 7.5, z], uv: [0.0, 0.0], rgba: [255.0; 4] },
                ShardVert { pos: [4.9, 7.5, z], uv: [1.0, 0.0], rgba: [255.0; 4] },
                ShardVert { pos: [4.5, 7.9, z], uv: [0.0, 1.0], rgba: [255.0; 4] },
            ],
            material: 7,
        };
        b.list[0].src.glass = vec![mk(-2.5), mk(-2.4), mk(-2.3)];
        let mut rng = Rng::new(0x5eed);
        let draws_before = rng.draws;
        let kills = b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], GUN, 25.0, &mut rng);
        assert_eq!(kills.len(), 1);
        assert_eq!(b.shatter.shards.len(), 3, "one leaf shard per small source triangle [M 0x12b270]");
        for s in &b.shatter.shards {
            assert_eq!(s.material, 7, "the face's draw word rides the shard [M 0x12b270:210]");
            assert!((s.lifetime - crate::grenade::REC_FRAME_RATE * 25.0).abs() < 1.0e-6,
                "lifetime = REC·25 [M 0x12b270:205-209]");
            assert!(s.delay > 0.0, "the peel delay = dist·REC·15 > 0 for a hit at distance [M 0x12b270:190-192]");
        }
        assert_eq!(rng.draws - draws_before, 9, "three leaf shards × three spin draws [M 0x12b270:193-201]");
        // The tick: the shards hold their delay, then fall and expire on the game's timer.
        for _ in 0..45 {
            b.shatter.update();
        }
        assert!(b.shatter.shards.is_empty(), "expired on the game's 25-tick timer [M 0x12b808:52-83]");
        // Sister: an unshot breakable spawns nothing (the same pane, no bullet).
        let b = one(2, 0.0);
        assert!(b.shatter.shards.is_empty() && b.shatter.debris.is_empty());
    }

    #[test]
    fn a_big_source_triangle_splits_with_the_games_edge_rule() {
        // A triangle whose longest edge² exceeds the threshold splits on its longest edge at a
        // random t ∈ [0.25, 0.75) — one draw — and recurses [M 0x12b270:214-352].
        let mut b = one(2, 0.0);
        b.list[0].src.glass = vec![GlassTri {
            v: [
                ShardVert { pos: [4.0, 7.0, -2.5], uv: [0.0, 0.0], rgba: [255.0; 4] },
                ShardVert { pos: [8.0, 7.0, -2.5], uv: [1.0, 0.0], rgba: [255.0; 4] },
                ShardVert { pos: [4.0, 11.0, -2.5], uv: [0.0, 1.0], rgba: [255.0; 4] },
            ],
            material: 1,
        }];
        let mut rng = Rng::new(3);
        b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], GUN, 25.0, &mut rng);
        assert!(b.shatter.shards.len() >= 2, "the split recursed until every piece was a leaf");
        // Every shard's verts stay within the source triangle's span (the bisection conserves;
        // the recomposition carries ~1e-7 of float dust).
        for s in &b.shatter.shards {
            for v in &s.verts {
                assert!((4.0 - 1.0e-3..=8.0 + 1.0e-3).contains(&(v.pos[0] + s.pos[0]))
                    && (7.0 - 1.0e-3..=11.0 + 1.0e-3).contains(&(v.pos[1] + s.pos[1])),
                    "the sub-triangles tile the source [M 0x12b270]");
            }
        }
    }

    #[test]
    fn the_debris_tick_follows_debris_update() {
        // pos += vel; vel += WldGravity·REC²; rot += spin; life −= 1; fade −= 1 per tick; dead
        // at life < 1 [M Debris_Update 0x1aac00:21-56].
        let mut b = one(0, 0.0);
        let mut rng = Rng::new(5);
        b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], GUN, 25.0, &mut rng);
        let life0 = b.shatter.debris[0].life;
        let pos0 = b.shatter.debris[0].pos;
        let vel0 = b.shatter.debris[0].vel;
        b.shatter.update();
        let d = &b.shatter.debris[0];
        assert_eq!(d.life, life0 - 1, "life −= 1 per tick [M 0x1aac00:27-29]");
        assert_eq!(d.fade, 0xfe, "obj+0x106 −= 1 per tick [M 0x1aac00:21-24]");
        for k in 0..3 {
            assert!((d.pos[k] - (pos0[k] + vel0[k])).abs() < 1.0e-6, "pos += vel [M 0x1aac00:31-35]");
            let g = [0.0, -9.8, 0.0][k] * crate::grenade::REC_FRAME_RATE * crate::grenade::REC_FRAME_RATE;
            assert!((d.vel[k] - (vel0[k] + g)).abs() < 1.0e-7, "vel += WldGravity·REC² [M 0x1aac00:36-43]");
        }
        for _ in 0..200 {
            b.shatter.update();
        }
        assert!(b.shatter.debris.is_empty(), "dead on the game's timer (life < 1 at entry) [M 0x1aac00:14]");
    }
}
