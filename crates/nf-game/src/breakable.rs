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
//! The bullet reject, evidenced form [M 0x12bfd8:15 + 0x1ece30:24-33]: the contacts filter through
//! `Collide_FilterBullets(obj+0xD0, filter | 0x106)` — a class-5 hitter whose
//! `weapon_definition+0x10` word ANDs the mask non-zero is dropped. The disc's words: guns 0x0008,
//! Laser 0x0002, watch 0x0100 [L dump] — the base 0x106 mask rejects the laser/watch classes even
//! on a no-filter pane, guns pass. The call site reads the word at `weapons.records[weapon][0x10..0x12]`
//! like the game (drone.rs's bullet pass).
//!
//! The kill sound roll [M 0x12b970:65-76]: kind 2 → 0x2E with NO RNG draw; every other kind rolls
//! `Rand_Rand(100) < 0x29` → 0x5F2 else 0x5F3, drawn only when a non-glass pane with collision
//! records dies (the roll sits inside Break_Kill after the records gate, before Debris_Create_Loop).
//!
//! The pane bodies (glass-breach lineage): the 0x20 placements do NOT bake into the static
//! collision set (world.rs) — `Break_Kill`'s obj+0xFE |= 1 takes the pane out of the world and a
//! static bake cannot undo it [M 0x12b970:60]. The Drones inject each INTACT pane every tick
//! through `set_dynamic` (id 0x300_0000_0000 | placement, shape = the `shape_key` below); a broken
//! pane injects nothing and bodies cross the opening.
//!
//! Port deviations (documented, none tested as game behaviour):
//! - the pieces' draw presentation (matrices, models, look) is the renderer's work; the sim
//!   exposes the piece lists (`Shatter`) the draw layer reads (break-debris.md).
//! - `shape_key` — the collision-shape key the injection needs: the target object's hashcode, or
//!   world.rs's anonymous (map, object) synthetic `0x8000_0000 | (map << 16) | object` [S — the
//!   shapes index's own rule, world.rs build()].

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
    /// tag+0x04 — placement param 1's bits as f32 [M 0x12c230:23-29].
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
    /// The collision-shape key the intact-pane injection needs: the target object's hashcode, or
    /// world.rs's anonymous (map, object) synthetic `0x8000_0000 | (map << 16) | object` when the
    /// hashcode is u32::MAX [S — the shapes index's own rule, world.rs build()]. 0 = no shape (the
    /// pane injects nothing).
    pub shape_key: u32,
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
            // The shapes-index key for the injection (see the field's doc): the object's hashcode,
            // or the (map, object) synthetic for an anonymous object — the same pair resolve()
            // looks up, world.rs build()'s rule [S].
            let (om, oo) = if p.by_hash() { level.by_hash.get(&p.hashcode).copied().unwrap_or((m, p.index as usize)) } else { (m, p.index as usize) };
            let shape_key = level.maps.get(om).and_then(|mp| mp.objects.get(oo))
                .map_or(0, |o| if o.hashcode == u32::MAX { 0x8000_0000 | ((om as u32) << 16) | (oo as u32 & 0xffff) } else { o.hashcode });
            list.push(Break {
                placement: i,
                lo,
                hi,
                kind,
                health: f32::from_bits(param(1)),
                filter: if p2 == 0 { 0 } else { !(p2 as u16) },
                dead: false,
                matrix: object_matrix(p.rotation, p.translation),
                src,
                shape_key,
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
    /// class-5 contact subtracts its damage]). `weapon_mask` is the bullet's weapon-definition
    /// word at `weapon_definition+0x10` (the caller reads `weapons.records[weapon][0x10..0x12]`,
    /// like the game): `Collide_FilterBullets(list, filter | 0x106)` drops a hitter whose word
    /// ANDs the mask non-zero [M 0x12bfd8:15 + 0x1ece30:24-33] — the base 0x106 rejects the
    /// laser (0x0002) and watch (0x0100) classes even on a no-filter pane, guns (0x0008) pass
    /// [L dump]. Returns the kills for presentation; each kill spawns its pieces (crate::shatter).
    pub fn bullet_hit(&mut self, a: V3, b: V3, weapon_mask: u16, damage: f32, rng: &mut Rng) -> Vec<BreakEffect> {
        let mut hits = Vec::new();
        for (i, br) in self.list.iter_mut().enumerate().filter(|(_, b)| !b.dead) {
            // `Collide_FilterBullets` [M 0x1ece30:24-33]: the weapon word against the pane's
            // mask | 0x106 — non-zero AND = the hit is dropped.
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

    /// The placement's death flag: `obj+0xFE & 1` [M 0x12b970:60] — the draw/collision lookups
    /// go through it (a dead pane draws nothing and blocks nothing).
    pub fn broken_at(&self, placement: usize) -> bool {
        self.list.iter().find(|b| b.placement == placement).is_some_and(|b| b.dead)
    }

    /// `Break_DoBreak(prev_obj+0xC0)` 0x12c168 for the PLACED panes: the first intact breakable
    /// whose bounds (grown 0.2 [S — the game tests the world bounding sphere obj+0x80/+0x8C; the
    /// bounds are the port's sheet]) contain the point dies through `kill` (the dead bit, the
    /// sound roll, the pieces) [M 0x12c168:39-61, 0x12b970]. The game walks the CONTROL list in
    /// creation order — placed 0x20 objects precede script entities — so the NIS arm runs this
    /// BEFORE its script-entity scan.
    pub fn do_break(&mut self, pos: V3, rng: &mut Rng) -> Vec<BreakEffect> {
        let Some(i) = self.list.iter().position(|b| !b.dead && Self::seg_box_grow(pos, pos, b.lo, b.hi, 0.2)) else { return Vec::new() };
        self.kill(i, pos, pos, rng).into_iter().collect()
    }

    /// The drone-contact arm ([M Break_Update 0x12bfd8:23-34 — only on GameState+0x12 ==
    /// 0x7000001, only kind == 2, damage = the drone's squared step × 20]). `level_01` is the
    /// caller's `GameState+0x12 == 0x07000001` check. `scripted_walk`: this tick's position
    /// change was carried by the anim root motion (the port's `Drone::scripted_move`) —
    /// glass-breach-guards-4's gate, the last paragraph.
    ///
    /// The contact stands in for the game's collision pair ([M 0x12bfd8:15 — the pane's obj+0xD0
    /// list the collision pass fills): the pair exists when the body's movement CARRIES IT INTO
    /// the pane's collision sheet — this tick's step segment crosses the pane's bounds, grown 0.7
    /// [S — the game's contact is the body volume against the pane's glist triangles; the growth
    /// covers the port's blocked standoff: the injected pane stops the body just short of the
    /// sheet, and the pressed body's next step must still reach it]. A body whose approach merely
    /// closes distance (a guard walking up to the glass, or standing near it) has no pair until
    /// the collision engages; the injected pane stops bodies at the sheet, so the crossing IS the
    /// engagement. The previous 0.55-grown hull zone fired ~0.6 before any physical contact — a
    /// proximity kill the game's list cannot produce (a pressed guard's position is
    /// collision-owned: zero net step, zero damage [M the SqDist rule]).
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
            // The contact: this tick's step reaches the pane's bounds (the collision pair; the
            // 0.7 growth covers the injected pane's blocked standoff [S]).
            if Self::seg_box_grow(drone_from, drone_to, br.lo, br.hi, 0.7) {
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
        // The kill sound BEFORE the pieces, in the game's order: kind 2 plays 0x2E with no draw;
        // every other kind rolls Rand_Rand(100) < 0x29 → 0x5F2 else 0x5F3 [M 0x12b970:65-76 —
        // the roll sits after the records gate, before Debris_Create_Loop :78+].
        let sound = if br.kind == 2 {
            0x2E
        } else if rng.rand(100) < 0x29 {
            0x5F2
        } else {
            0x5F3
        };
        let (hit, normal, mat) = Self::contact(&br.src.records, a, b, trans, centre);
        // Break_Kill defaults the hit material's low 6 bits to 0x10 when zero — the debris
        // table's breakable row [M 0x12b970:137-138 `if ((hitdata+0x50 & 0x3f) == 0) = 0x10`].
        let mat = if mat & 0x3f == 0 { 0x10 } else { mat };
        if br.kind == 2 {
            self.shatter.spawn_glass(&br.src.glass, trans, hit, normal, rng);
        } else {
            self.shatter.spawn_debris(&br.src.records, trans, mat, rng);
        }
        let br = &self.list[i];
        Some(BreakEffect { placement: br.placement, pos: centre, kind: br.kind, sound })
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

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shatter::EFFECT_INFO;

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
                shape_key: 0,
            }],
            shatter: Default::default(),
        }
    }

    #[test]
    fn the_first_bullet_kills_a_zero_health_vase_with_the_debris_roll() {
        // The 07000001 vases carry params [(2,0)] only — kind 0, health 0.0 [D placements dump].
        // Break_Kill rolls Rand_Rand(100) < 0x29 → 0x5f2 else 0x5f3 [M 0x12b970:65-76] — the old
        // unconditional-0x5F3 pin was the [S] stopgap and contradicted the decompile.
        let mut b = one(0, 0.0);
        let mut rng = Rng::new(1);
        // A pistol round's weapon word (guns 0x0008 [L dump] — 0x0008 & 0x106 == 0 passes the
        // base mask [M 0x12bfd8:15]).
        let kills = b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], 0x0008, 25.0, &mut rng);
        assert!(b.list[0].dead, "health 0 - damage < 0 → Break_Kill [M 0x12bfd8:44-50]");
        assert_eq!(kills.len(), 1);
        assert_eq!(kills[0].kind, 0);
        // The roll against the same seed's stream: the kill's FIRST draw is the sound roll.
        let mut probe = Rng::new(1);
        let want = if probe.rand(100) < 0x29 { 0x5F2 } else { 0x5F3 };
        assert_eq!(kills[0].sound, want, "the kill rolled Rand_Rand(100) < 0x29 → 0x5f2 else 0x5f3 [M 0x12b970:65-76]");
        // The kill spawned the game's debris: one record → one piece, EffectInfo[0x10].
        assert_eq!(b.shatter.debris.len(), 1, "Debris_Create_Loop: one object per stride-3 record [M 0x1ed548]");
        assert_eq!(b.shatter.debris[0].model, EFFECT_INFO[0x10].model, "EffectInfo[hitdata+0x50 & 0x3f = 0x10] [M 0x12b970:137-141]");
    }

    #[test]
    fn glass_shatters_with_sound_0x2e_and_the_sister_vase_survives_a_miss() {
        // radioroom_glass/comroom_glass carry params [(0,2),(1,0),(2,0)] — kind 2 [D placements].
        let mut b = one(2, 0.0);
        let mut rng = Rng::new(1);
        let kills = b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], 0x0008, 25.0, &mut rng);
        assert_eq!(kills[0].sound, 0x2E, "glass → sound 0x2E [M 0x12b970: uVar5 = 0x2e]");
        // Sister: a shot wide of the box never touches the health and spawns nothing.
        let mut b = one(2, 0.0);
        let mut rng = Rng::new(1);
        let kills = b.bullet_hit([50.0, 8.0, -4.0], [50.0, 8.0, -2.0], 0x0008, 25.0, &mut rng);
        assert!(kills.is_empty() && !b.list[0].dead, "a miss never damages the breakable");
        assert!(b.shatter.debris.is_empty() && b.shatter.shards.is_empty(), "an unshot breakable spawns nothing");
    }

    #[test]
    fn a_health_pool_takes_repeated_hits_and_only_dies_below_zero() {
        // Destroy_test's rule: exactly 0.0 is NOT a kill [M 0x12bfd8:50 `fVar5 < 0.0`].
        let mut b = one(0, 50.0);
        let mut rng = Rng::new(1);
        for i in 0..5 {
            let kills = b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], 0x0008, 10.0, &mut rng);
            assert!(kills.is_empty() && !b.list[0].dead, "hit {}: {} left", i, 50 - 10 * (i + 1));
        }
        let kills = b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], 0x0008, 10.0, &mut rng);
        assert_eq!(kills.len(), 1, "the 6th crosses below zero");
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
        let kills = b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], 0x0008, 25.0, &mut rng);
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
        b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], 0x0008, 25.0, &mut rng);
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
        assert_eq!(rng.draws - draws_before, 9, "the sound roll [M 0x12b970:65-76] + eight shared draws per debris object [M 0x1aaa00:30-50]");
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
        let kills = b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], 0x0008, 25.0, &mut rng);
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
        b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], 0x0008, 25.0, &mut rng);
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
        b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], 0x0008, 25.0, &mut rng);
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

    #[test]
    fn the_laser_and_watch_words_are_rejected_by_the_base_mask_and_the_gun_passes() {
        // Collide_FilterBullets is called with `filter | 0x106` even on a no-filter pane [M
        // 0x12bfd8:15]; a hitter whose weapon word ANDs it is dropped [M 0x1ece30:24-33]. The
        // disc's words: guns 0x0008 (passes — 0x0008 & 0x106 == 0), Laser 0x0002 and watch 0x0100
        // (both rejected) [L dump].
        for (word, breaks) in [(0x0002u16, false), (0x0100, false), (0x0008, true)] {
            let mut b = one(2, 0.0);
            let mut rng = Rng::new(1);
            let kills = b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], word, 25.0, &mut rng);
            assert_eq!(kills.is_empty(), !breaks, "word {word:#x}: the base 0x106 mask filters it [M 0x1ece30:24-33]");
            assert_eq!(b.list[0].dead, breaks);
        }
    }

    #[test]
    fn the_panes_filter_param_rejects_matching_weapon_words() {
        // A pane with filter param 2 = 0xf8 stores ~0xf8 = 0xff07 [M 0x12c230:31-39]; a weapon
        // word ANDing filter | 0x106 is dropped [M 0x1ece30:24-33]. Word 0x0008: 0x0008 &
        // (0xff07 | 0x106) == 0 → passes; word 0x0007 (bits 0-2 sit in the mask) is dropped.
        let pane = |filter: u16| Break {
            placement: 0, lo: [0.0, 0.0, 0.0], hi: [2.0, 2.0, 2.0], kind: 2, health: 0.0,
            filter, dead: false, matrix: [[0.0; 4]; 4],
            src: Source { records: vec![DebrisRec { v: [[0.5, 0.5, -0.5], [1.5, 0.5, -0.5], [1.0, 1.5, -0.5]], n: [0.0, 0.0, -1.0], mat: 0x10 }], glass: Vec::new() },
            shape_key: 0,
        };
        let mut b = Breaks { list: vec![pane(0xff07)], shatter: Default::default() };
        let mut rng = Rng::new(1);
        let kills = b.bullet_hit([1.0, 1.0, -1.0], [1.0, 1.0, 3.0], 0x0008, 10.0, &mut rng);
        assert_eq!(kills.len(), 1, "the gun's word passes the 0xf8-derived mask");
        let mut b2 = Breaks { list: vec![pane(0xff07)], shatter: Default::default() };
        let kills2 = b2.bullet_hit([1.0, 1.0, -1.0], [1.0, 1.0, 3.0], 0x0007, 10.0, &mut rng);
        assert!(kills2.is_empty() && !b2.list[0].dead, "a word inside the mask never damages the pane [M 0x1ece30:24-33]");
    }

    #[test]
    fn a_broken_pane_registers_nothing_but_the_next_one_still_breaks() {
        // 07000008's control-room panes: kind 2, health 0, no filter [D placements] — two flat
        // planes (123 the x = 105.3 pane, 128 the z = −122 pane).
        let pane = |pl: usize, lo: V3, hi: V3| Break {
            placement: pl, lo, hi, kind: 2, health: 0.0, filter: 0, dead: false, matrix: [[0.0; 4]; 4],
            src: Source { records: vec![DebrisRec { v: [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]], n: [0.0, 0.0, -1.0], mat: 0x10 }], glass: Vec::new() },
            shape_key: 0,
        };
        let mut b = Breaks { list: vec![
            pane(123, [105.3, 34.2, -122.0], [105.3, 36.4, -120.1]),
            pane(128, [105.3, 34.2, -122.0], [109.2, 36.4, -122.0]),
        ], shatter: Default::default() };
        let mut rng = Rng::new(1);
        // The round runs along z at x = 105.3, crossing only pane 123's plane.
        let kills = b.bullet_hit([105.3, 35.0, -121.5], [105.3, 35.0, -120.0], 0x0008, 10.0, &mut rng);
        assert_eq!(kills.len(), 1, "the first round shatters it [M 0x12bfd8:49-51]");
        assert!(b.broken_at(123), "obj+0xFE & 1 is set [M 0x12b970:60]");
        // The second round through the SAME spot: the dead pane registers no hit [M 0x12b970:60].
        let kills = b.bullet_hit([105.3, 35.0, -121.5], [105.3, 35.0, -120.0], 0x0008, 10.0, &mut rng);
        assert!(kills.is_empty(), "a broken pane is gone from the world");
        // The sister: the OTHER intact pane still breaks — the system is alive.
        let kills = b.bullet_hit([107.0, 35.0, -123.0], [107.0, 35.0, -121.0], 0x0008, 10.0, &mut rng);
        assert_eq!(kills.len(), 1, "the neighbouring pane still shatters [D 07000008 has 3 panes]");
        assert!(b.broken_at(128));
    }

    #[test]
    fn a_drone_step_drains_sqdist_times_twenty_and_the_glass_kill_draws_nothing() {
        // 07000001's pane 136 bounds [D nf_view bounds dump].
        let pane = |pl: usize, kind: u8, health: f32| Break {
            placement: pl, lo: [-1.91, 5.34, -1.167], hi: [-0.09, 7.95, -1.167], kind, health,
            filter: 0, dead: false, matrix: [[0.0; 4]; 4],
            src: Source { records: vec![DebrisRec { v: [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]], n: [0.0, 0.0, -1.0], mat: 0x10 }], glass: Vec::new() },
            shape_key: 0,
        };
        let mut b = Breaks { list: vec![pane(136, 2, 0.0)], shatter: Default::default() };
        let mut rng = Rng::new(1);
        // The level gate: the class-2 arm exists on 0x0700001 ALONE [M 0x12bfd8:24].
        let ev0 = b.drone_step([-1.0, 6.3, -0.5], [-1.0, 6.3, -1.6], false, false, &mut rng);
        assert!(ev0.is_empty() && !b.broken_at(136), "another level never smashes by contact [M 0x12bfd8:24]");
        // A motionless brush drains nothing [M :27 — SqDist(pos, prev) = 0].
        let ev1 = b.drone_step([-1.0, 6.3, -1.0], [-1.0, 6.3, -1.0], true, false, &mut rng);
        assert!(ev1.is_empty() && !b.broken_at(136), "a stationary contact survives the health-0 pane");
        // The tick's movement through the pane plane kills it [M :26-27, :49-51].
        let ev2 = b.drone_step([-1.0, 6.3, -0.5], [-1.0, 6.3, -1.6], true, false, &mut rng);
        assert_eq!(ev2.len(), 1, "the crossing shatters the health-0 pane");
        assert_eq!(ev2[0].sound, 0x2e, "kind 2 plays 0x2e, no roll [M 0x12b970:65-70]");
        assert_eq!(ev2[0].kind, 2);
        assert!(b.broken_at(136), "obj+0xFE |= 1 [M 0x12b970:60]");
        // Sister: a NON-glass kind is immune to the class-2 arm (the kind gate [M :25]).
        let mut b2 = Breaks { list: vec![pane(5, 0, 0.0)], shatter: Default::default() };
        let ev3 = b2.drone_step([-1.0, 6.3, -0.5], [-1.0, 6.3, -1.6], true, false, &mut rng);
        assert!(ev3.is_empty() && !b2.broken_at(5), "the smash arm is kind-2 only [M 0x12bfd8:25]");
        // Sister: the drain is exactly SqDist·20 — a 0.3 step (d2 0.09 → 1.8) dents a 100 pool.
        let mut b3 = Breaks { list: vec![pane(6, 2, 100.0)], shatter: Default::default() };
        let ev4 = b3.drone_step([-1.0, 6.3, -0.5], [-1.0, 6.3, -0.8], true, false, &mut rng);
        assert!(ev4.is_empty() && !b3.broken_at(6));
        assert!((b3.list[0].health - 98.2).abs() < 1e-4, "health -= SqDist·20 [M 0x12bfd8:27], got {}", b3.list[0].health);
        // The RNG stream is untouched by a glass kill (0x2e is unconditional [M 0x12b970:65-70]):
        // the same-seed streams read the same next draw whether or not the kill sat between.
        let mut rng_a = Rng::new(0x5eed);
        let _ = rng_a.rand(100);
        let mut b4 = Breaks { list: vec![pane(7, 2, 0.0)], shatter: Default::default() };
        let _ = b4.drone_step([-1.0, 6.3, -0.5], [-1.0, 6.3, -1.6], true, false, &mut rng_a);
        let after_kill = rng_a.rand(100);
        let mut rng_b = Rng::new(0x5eed);
        let _ = rng_b.rand(100);
        let without_kill = rng_b.rand(100);
        assert_eq!(after_kill, without_kill, "the glass kill drew nothing between the two rolls");
    }

    #[test]
    fn do_break_kills_the_first_intact_pane_containing_the_point() {
        // The game walks the object list and kills the FIRST class-0x20 sphere containing the
        // point [M Break_DoBreak 0x12c168:39-51,61]; the port's order is placement order.
        let pane = |pl: usize, lo: V3, hi: V3| Break {
            placement: pl, lo, hi, kind: 2, health: 0.0, filter: 0, dead: false, matrix: [[0.0; 4]; 4],
            src: Source { records: vec![DebrisRec { v: [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]], n: [0.0, 0.0, -1.0], mat: 0x10 }], glass: Vec::new() },
            shape_key: 0,
        };
        let mut b = Breaks { list: vec![
            pane(10, [-1.91, 5.34, -1.167], [-0.09, 7.95, -1.167]),
            pane(11, [5.0, 5.34, -1.167], [6.8, 7.95, -1.167]),
        ], shatter: Default::default() };
        let mut rng = Rng::new(7);
        // The point sits in pane 11 only (pane 10's grow 0.2 reaches x −0.09+0.2 = 0.11 < 5).
        let ev = b.do_break([5.9, 6.3, -1.167], &mut rng);
        assert_eq!(ev.len(), 1, "the scripted kill fires [M 0x12c168:39-61]");
        assert!(b.broken_at(11) && !b.broken_at(10), "the containing pane died, the far one lives");
        assert_eq!(ev[0].sound, 0x2e, "kind 2 → 0x2e, no roll [M 0x12b970:65-70]");
        // Sister: no pane at the point → nothing dies.
        let mut b2 = Breaks { list: vec![pane(10, [-1.91, 5.34, -1.167], [-0.09, 7.95, -1.167])], shatter: Default::default() };
        let ev2 = b2.do_break([100.0, 6.3, -1.167], &mut rng);
        assert!(ev2.is_empty() && !b2.broken_at(10), "a point outside every sphere kills nothing");
        // Sister: a BROKEN pane is skipped (the first INTACT one [M :45 — the FE&1 gate]).
        let mut broken = pane(12, [-1.91, 5.34, -1.167], [-0.09, 7.95, -1.167]);
        broken.dead = true;
        let mut b3 = Breaks { list: vec![broken, pane(13, [-1.91, 5.34, -1.167], [-0.09, 7.95, -1.167])], shatter: Default::default() };
        let ev3 = b3.do_break([-1.0, 6.3, -1.167], &mut rng);
        assert!(b3.broken_at(13) && ev3.len() == 1, "the kill skips the already-broken pane");
        // Sister: a recordless pane dies silently — Break_Kill's records gate [M 0x12b970:59-63]:
        // the dead bit sets, but no sound effect returns.
        let mut bare = pane(14, [-1.91, 5.34, -1.167], [-0.09, 7.95, -1.167]);
        bare.src.records.clear();
        let mut b4 = Breaks { list: vec![bare], shatter: Default::default() };
        let ev4 = b4.do_break([-1.0, 6.3, -1.167], &mut rng);
        assert!(ev4.is_empty() && b4.broken_at(14), "the silent kill still takes the pane out of the world");
    }
}
