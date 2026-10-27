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
//! The script-created panes (`Script_CreateEntity` 0x1d8968's 0x20 arm — the gondola's 16 cabin
//! windows ride the carriage in 07000008's ride script 0x0600004C, streams 7-18/23-26, params
//! [2, 1, 0] [D decode]): the op-7 params land in the shared level_tag 0x349948's +0x2C/+0x30/
//! +0x34 slots (`Script_EntityStart` 0x1d8d78:80-87 writes param k at 0x349948+0x2C+4k;
//! `Pickup_Create`'s :159 reads the same slots), so `Break_Create` reads kind/health/filter from
//! the Entity params exactly as from a placement's. The pane is created at the origin with the
//! Entity's glist [M 0x1d8968:120-123] and POSED BY THE SCRIPT: `Script_Interp` 0x1d9a08:94-112
//! rewrites obj+0x90 every tick from the parent chain × the stream keys (the windows are parented
//! to the carriage stream), and the bullet's `Collide_Intersect` 0x1ebaa8:157-177 tests the
//! segment against the pane's CURRENT local space — the ride carries the hit surface. The port's
//! pane row therefore re-derives its bounds from the live `Obj::Entity` matrix every tick
//! (`script_pane`), and `Break_Kill` on a script pane gives the dead bit + the kind sound and NO
//! pieces: the shatter tri list comes from `psiCreateBreakableData`, whose only caller is the
//! placement path (parsemap case 0x20 kind == 2 [M 0x1d04f0:70-73]) — `psiGetTriList` reads
//! count 0 and the `ShatterPolyRecurse` loop never runs [M 0x12b970:78-133]. The entity's
//! deletion (obj+0xFE bit 0's world+draw effect [M 0x12b970:60]) is the caller's.
//!
//! Port deviations (documented, none tested as game behaviour):
//! - the pieces' draw presentation (matrices, models, look) is the renderer's work; the sim
//!   exposes the piece lists (`Shatter`) the draw layer reads (break-debris.md).
//! - the kill's position in the shared RNG stream: the game draws the roll inside
//!   `Break_Update`'s control-walk slot — the frame AFTER the bullet's flight registered the
//!   contact (the game's contact comes from `Collide_Update` 0x1ea398, the last step of
//!   `control_movement_object_handler` 0x1336f8 [M]; `Break_Update` consumes it in the NEXT
//!   frame's walk). The port compresses the two frames into its bullet pass, so the roll (and
//!   the debris draws) land one AI-frame earlier in the port's stream than the game's — the
//!   same standing compression every bullet-impact consumer already carries. Within the kill
//!   the order is the game's: roll, then the pieces' draws.

use crate::drone::Rng;
use crate::shatter::{self, DebrisRec, GlassTri, ShardVert, Source};
use crate::world::V3;
use nf_formats::level::Level;
use nf_formats::placement::{transform, view_rot_trans_matrix};

/// One kill effect for the caller to present (sound + shatter/debris) — `Break_Kill` 0x12b970's
/// observable outputs. The game plays the sound at obj+0x80 (the placed object's position).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BreakEffect {
    pub placement: usize,
    /// The script-entity panes: the killed object's index in `Scripts::objects` (the caller
    /// removes the entity — `Break_Kill`'s obj+0xFE delete flag [M 0x12b970:60]); `None` = placed.
    pub script_obj: Option<usize>,
    /// The kill position (the bounds centre — the port's stand-in for obj+0x80).
    pub pos: V3,
    /// tag+0x00: 2 = glass (shatter + sound 0x2E), other = debris (0x5F2 on
    /// `Rand_Rand(100) < 0x29`, else 0x5F3 [M 0x12b970:65-76]).
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
    /// [M 0x12b970:61-63]. A script pane carries none — see the module header.
    pub src: Source,
    /// The collision world's shape key for the pane's object (`world.shapes`' indexing: the
    /// object's hashcode when `by_hash` resolves it to this very object, else the anonymous
    /// synthetic `0x8000_0000 | map << 16 | object` [M world.rs `build_scoped`'s shape index]).
    /// 0 when the placement resolves to nothing. The solid-while-alive ride uses it
    /// (`set_dynamic`'s parts carry a glist shape key, not a placement).
    pub shape_key: u32,
    /// The frame the pane died on (`u32::MAX` while alive). The solid-while-alive ride keeps a
    /// just-killed pane's part for its DEATH tick: the game's collide pass registers the killing
    /// bullet's contact BEFORE the object walk's `Break_Kill` takes the collision away
    /// [M the contact list is the collide pass's product; Break_Update walks it same tick], so
    /// the same-tick deferred HITDATA re-trace (`flush_world_impacts`) must still see the sheet;
    /// the next tick's ride drops it [M 0x12b970:60].
    pub dead_frame: u32,
    /// The script-entity panes (`Script_CreateEntity` 0x1d8968's 0x20 arm): the object index in
    /// `Scripts::objects` whose live matrix carries the pane (the gondola's windows ride the
    /// carriage); `None` = a placed pane. `placement` carries the same index for these rows
    /// (there is no placement).
    pub script_obj: Option<usize>,
}

#[derive(Clone, Debug, Default)]
pub struct Breaks {
    pub list: Vec<Break>,
    /// The kill pieces (crate::shatter) — `Break_Kill`'s spawns, updated per tick; the draw
    /// layer reads them (docs/systems/break-debris.md).
    pub shatter: shatter::Shatter,
    /// The sim frame, stamped by the tick (the kill's `dead_frame` reader).
    pub frame: u32,
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
    // Break_Create builds obj+0x90 with control_create_object's quaternion-0 arm →
    // View_RotTransMatrix [M 0x12c230:12, 0x1344e0:26-29].
    let mx = view_rot_trans_matrix(p.rotation, p.translation);
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
            // The shape key, resolved exactly like `world.shapes`' index: by hashcode when
            // `by_hash` maps it back to this object, else the anonymous synthetic. The game's
            // pane object carries its own collision block (`cel+8` [M 0x12b970:59-63]) — the
            // same triangles `Break_Create`'d object stands on.
            let target = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < level.maps[m].objects.len() { Some((m, p.index as usize)) } else { None };
            let shape_key = match target {
                Some((om, oo)) => {
                    let o = &level.maps[om].objects[oo];
                    if o.hashcode != u32::MAX && level.by_hash.get(&o.hashcode).copied() == Some((om, oo)) { o.hashcode }
                    else { 0x8000_0000 | ((om as u32) << 16) | oo as u32 }
                }
                None => 0,
            };
            list.push(Break {
                placement: i,
                lo,
                hi,
                kind,
                // tag+0x04 = (f32)(i32) param 1 — the SIGNED INT→FLOAT conversion [M 0x12c230
                // disasm 0x12c2a8-0x12c2c4: `lw 0x30($s3)` → `mtc1` → `cvt.s.w` → `swc1 4($s1)`;
                // the old f32::from_bits read the bits raw (param 1 = 1 was 1e-45, the gondola's
                // authored health 1 is 1.0)]. 0700000a's #379 param 1 = 9 [D] → health 9.0.
                health: (param(1) as i32) as f32,
                // `Break_Create` 0x12c230 disasm 0x12c2e4-0x12c2f8: param 2 == 0 → 0, else
                // `lhu` + `nor` (~param2) → `sh 8($s1)` — the datum word `Break_Update` ORs
                // 0x106 into [M 0x12c028-0x12c030].
                filter: if p2 == 0 { 0 } else { !(p2 as u16) },
                dead: false,
                matrix: view_rot_trans_matrix(p.rotation, p.translation), // [M 0x12c230:12 CCO(0)]
                src,
                shape_key,
                dead_frame: u32::MAX,
                script_obj: None,
            });
        }
        Breaks { list, shatter: shatter::Shatter::default(), frame: 0 }
    }

    fn seg_box(a: V3, b: V3, lo: V3, hi: V3) -> bool {
        // The bullet segment must actually cross the pane's bounds (grown 0.1 — the
        // sphere-touch stand-in), not merely overlap them with its AABB.
        Self::seg_box_grow(a, b, lo, hi, 0.1)
    }

    /// The entry fraction of the segment through the same 0.1-grown bounds box (the slab test on
    /// the grown box `seg_box` overlaps), for the game's nearest-solid rule: the collide list is
    /// sorted by the hit distance [M Collide_Update 0x1ea398:120-158 QuickSort on HITDATA+0xc] and
    /// the first solid node takes the contact and kills the round [M 0x1ea398:216-229 +
    /// Bullet_CollisionHandler 0x12ce80 disasm 0x12cff0-4].
    pub fn seg_box_t(a: V3, b: V3, lo: V3, hi: V3) -> Option<f32> {
        // Degenerate bounds (the script-keyed regions parse to a point — the port has no glist
        // shape for them): the game's pick shape for such an object is its world sphere [M
        // Collide_PickObj 0x1eae80 — the closest point on the query line vs the object's
        // +0x80/+0x8c sphere], but the port cannot know the real region's extent here. The old
        // segment-AABB overlap stands for these until the shape lands — the port's own coarse
        // stand-in, documented [S] (a long step's AABB covers the point's neighbourhood even when
        // the ray misses; the real region shape is an open live check).
        if lo == hi {
            let mut over = true;
            for k in 0..3 {
                let (mn, mx) = (a[k].min(b[k]), a[k].max(b[k]));
                if mx < lo[k] - 0.1 || mn > hi[k] + 0.1 { over = false; break; }
            }
            if !over { return None; }
            let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let ab2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
            let ap = [lo[0] - a[0], lo[1] - a[1], lo[2] - a[2]];
            let t = if ab2 > 1.0e-9 { (ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2]) / ab2 } else { 0.0 }
                .clamp(0.0, 1.0);
            return Some(t);
        }
        let mut t0 = 0.0f32;
        let mut t1 = 1.0f32;
        for k in 0..3 {
            let (lo, hi) = (lo[k] - 0.1, hi[k] + 0.1);
            let d = b[k] - a[k];
            if d.abs() < 1.0e-9 {
                if a[k] < lo || a[k] > hi { return None; }
            } else {
                let (mut ta, mut tb) = ((lo - a[k]) / d, (hi - a[k]) / d);
                if ta > tb { std::mem::swap(&mut ta, &mut tb); }
                t0 = t0.max(ta);
                t1 = t1.min(tb);
                if t0 > t1 { return None; }
            }
        }
        Some(t0)
    }

    fn seg_box_grow(a: V3, b: V3, lo: V3, hi: V3, g: f32) -> bool {
        // A true segment-vs-box intersection (the slab method). The game's class-5 contact is
        // the collision pass's HITDATA — the bullet sphere touching the pane's collision
        // triangles [M 0x12bfd8:15, 36-48] — so a segment whose axis-aligned bounding BOX
        // overlaps the bounds but which flies PAST the sheet must not register (the 3b evidence:
        // 07000006's one pane died to a spread shot whose AABB spanned the slab while the
        // segment crossed the wall plane 1.9 units to its right, b9dfe235). The gondola's 16
        // windows sit side by side on the carriage — the same phantom would kill neighbours.
        // Degenerate segments (a == b: do_break's point probe, a collision-stopped body's zero
        // net step) fall out as the point-in-box check.
        let mut t0 = 0.0f32;
        let mut t1 = 1.0f32;
        for k in 0..3 {
            let d = b[k] - a[k];
            let (lo_k, hi_k) = (lo[k] - g, hi[k] + g);
            if d == 0.0 {
                if a[k] < lo_k || a[k] > hi_k { return false; }
            } else {
                let (mut tn, mut tf) = ((lo_k - a[k]) / d, (hi_k - a[k]) / d);
                if tn > tf { std::mem::swap(&mut tn, &mut tf); }
                t0 = t0.max(tn);
                t1 = t1.min(tf);
                if t0 > t1 { return false; }
            }
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
    /// glass-breach-guards-4's gate, the last paragraph. `blocked_reach`: the length of the
    /// step the body's movement probe REJECTED this tick (0 on an unblocked move) — see the
    /// contact paragraph.
    ///
    /// The contact is the collision PAIR ([M 0x12bfd8:15 — the pane's obj+0xD0 list the
    /// collision pass fills): the pair exists when the body's collision shape engages the
    /// pane's sheet. The pane stands in the collision world while alive (the dead one leaves it
    /// [M 0x12b970:60 — "the collision goes away"]), so the engagement is the body's 0.4
    /// collision cylinder [M DroneFunc_ClampPosToCollisionCylinder 0x1499d0 — the 0.4-radius
    /// body cylinder] swept over this tick's step touching the pane's bounds, grown by the
    /// rejected step: the port's movement probe (`Drone::move_by`) rejects a whole step whose
    /// ray would cross the pane and leaves +0x30 at the previous position, while the game's
    /// movement pass corrects +0x30 UP TO the shape — the rejected step's length is the port's
    /// measure of the movement that engagement absorbed [S] (the game's corrected step for the
    /// same contact is that order of magnitude; its exact value is a live check). A body whose
    /// approach merely closes distance (walking up to the glass, standing near it) has no pair
    /// until the shapes touch, and a body at rest (no rejected step, no sweep) reads none — the
    /// proximity kill the previous 0.55-grown hull zone produced cannot fire.
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
    pub fn drone_step(&mut self, drone_from: V3, drone_to: V3, level_01: bool, scripted_walk: bool, blocked_reach: f32, rng: &mut Rng) -> Vec<BreakEffect> {
        let mut kills = Vec::new();
        // The caller passes `scripted_walk` only for an AI state's anim step; a PlayScript start
        // script's move drains (the opening Yakuza's dive shatters #130 [L 2026-10-26, PCSX2 frame 2175]).
        if !level_01 || scripted_walk { return kills; }
        let sq = (drone_to[0] - drone_from[0]).powi(2)
            + (drone_to[1] - drone_from[1]).powi(2)
            + (drone_to[2] - drone_from[2]).powi(2);
        // The drain magnitude: the tick's real step when the body moved; the rejected step when
        // the collision engagement stopped it (the game's +0x30 reads the corrected position —
        // the movement the collision absorbed; the port's probe leaves the body put) [S].
        let drain_sq = if sq > 0.0 { sq } else { blocked_reach * blocked_reach };
        if drain_sq == 0.0 { return kills; }
        let dmg = drain_sq * 20.0;
        // The pair's reach: the 0.4 body cylinder swept over the step, grown by the rejected
        // step's length (the probe's own reach past the stopped body).
        let reach = 0.4 + blocked_reach;
        let mut hits = Vec::new();
        for (i, br) in self.list.iter_mut().enumerate().filter(|(_, b)| !b.dead && b.kind == 2) {
            // The contact: the body's collision shape engaged the pane's bounds this tick.
            if Self::seg_box_grow(drone_from, drone_to, br.lo, br.hi, reach) {
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
    /// debris records with `EffectInfo[material & 0x3F]` [M 0x12b970:78-141]. A script pane's
    /// kill is the dead bit + the kind sound and NO pieces (see the module header —
    /// `psiCreateBreakableData` never ran on its glist); the entity's deletion is the caller's.
    pub fn kill(&mut self, i: usize, a: V3, b: V3, rng: &mut Rng) -> Option<BreakEffect> {
        let br = &mut self.list[i];
        if br.dead { return None; }
        br.dead = true;
        br.dead_frame = self.frame;
        if br.script_obj.is_some() {
            // A script pane carries no records (see the module header); its sound follows the same
            // kind split and roll [M 0x12b970:65-76].
            let sound = if br.kind == 2 { 0x2E } else if rng.rand(100) < 0x29 { 0x5F2 } else { 0x5F3 };
            return Some(Self::effect(br, sound));
        }
        if br.src.records.is_empty() { return None; }
        let trans = [br.matrix[3][0], br.matrix[3][1], br.matrix[3][2]];
        let centre = [(br.lo[0] + br.hi[0]) / 2.0, (br.lo[1] + br.hi[1]) / 2.0, (br.lo[2] + br.hi[2]) / 2.0];
        // Break_Kill's sound, in the game's order: kind 2 plays 0x2E with NO draw; every other
        // kind draws `Rand_Rand(100)` — < 0x29 (41) → 0x5F2, else 0x5F3 — after the records
        // gate, BEFORE the shatter/debris draws [M 0x12b970:59-76: the `*piVar14 != 2` branch
        // draws at 0x12b9b4, both sound arms play at 0x12b9f0/0x12ba40, the pieces come at
        // 0x12ba48+; the debris/shatter draws are 0x1aaa00/0x12b270's, inside the loop].
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
        Some(Self::effect(&self.list[i], sound))
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

    fn effect(br: &Break, sound: u16) -> BreakEffect {
        let pos = [(br.lo[0] + br.hi[0]) / 2.0, (br.lo[1] + br.hi[1]) / 2.0, (br.lo[2] + br.hi[2]) / 2.0];
        BreakEffect { placement: br.placement, script_obj: br.script_obj, pos, kind: br.kind, sound }
    }

    /// Upsert one script-created pane (`Script_CreateEntity` 0x1d8968's 0x20 arm; the caller's
    /// per-tick sync — see the module header): kind/health/filter from the Entity params (the
    /// op-7 param block [M 0x1d8d78:80-87] is `Break_Create`'s level_tag [M 0x12c230:21-35]),
    /// bounds the glist's collision triangles transformed by the LIVE entity matrix (the game's
    /// per-tick obj+0x90 pose [M 0x1d9a08:94-112]; the port's stand-in for the current-local-space
    /// tri test [M 0x1ebaa8:157-177]). A same-object re-run only refreshes (the reuse path only
    /// re-points the gfx [M 0x1d8968:92-101]); the state (health) survives a re-point.
    /// No shape (the glist has no collision) never registers — the game's pane would be
    /// unhittable and its kill silent [M 0x12b970:61-63].
    pub fn script_pane(&mut self, obj: usize, params: &[u32], shape: &[[V3; 3]], matrix: [[f32; 4]; 4]) {
        let (lo, hi) = Self::shape_bounds(shape, &matrix);
        if let Some(br) = self.list.iter_mut().find(|b| b.script_obj == Some(obj)) {
            br.lo = lo;
            br.hi = hi;
            br.matrix = matrix;
            return;
        }
        let p = |k: usize| params.get(k).copied().unwrap_or(0);
        let p2 = p(2);
        self.list.push(Break {
            placement: obj,
            lo,
            hi,
            kind: (p(0) & 0xff) as u8,
            // (f32)(i32) param 1 [M 0x12c230 disasm 0x12c2a8-0x12c2c4 — the cvt.s.w, see create].
            health: (p(1) as i32) as f32,
            filter: if p2 == 0 { 0 } else { !(p2 as u16) },
            dead: false,
            matrix,
            src: Source::default(),
            // A script pane has no placed object: no static shape to ride (983's solid-while-
            // alive set covers placed panes only).
            shape_key: 0,
            dead_frame: u32::MAX,
            script_obj: Some(obj),
        });
    }

    /// The sync's retain: a script pane whose object is gone (the kill's deletion, a
    /// `Script_KillStream`) drops its row — the pane left the world with the object
    /// [M 0x12b970:60].
    pub fn retain_script_panes(&mut self, live_objs: &[usize]) {
        self.list.retain(|b| b.script_obj.is_none_or(|o| live_objs.contains(&o)));
    }

    /// The world-space bounds of a shape's triangles under `matrix`.
    fn shape_bounds(shape: &[[V3; 3]], matrix: &[[f32; 4]; 4]) -> (V3, V3) {
        let (mut lo, mut hi) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
        for t in shape {
            for v in t {
                let v = transform(matrix, *v);
                for k in 0..3 { lo[k] = lo[k].min(v[k]); hi[k] = hi[k].max(v[k]); }
            }
        }
        (lo, hi)
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
                shape_key: 0,
                dead_frame: u32::MAX,
                script_obj: None,
            }],
            shatter: Default::default(),
            frame: 0,
        }
    }

    #[test]
    fn the_first_bullet_kills_a_zero_health_vase_with_the_debris_roll() {
        // The 07000001 vases carry params [(2,0)] only — kind 0, health 0.0 [D placements dump].
        let mut b = one(0, 0.0);
        let mut rng = Rng::new(1);
        let draws0 = rng.draws;
        let kills = b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], GUN, 25.0, &mut rng);
        assert!(b.list[0].dead, "health 0 - damage < 0 → Break_Kill [M 0x12bfd8:44-50]");
        assert_eq!(kills.len(), 1);
        assert_eq!(kills[0].kind, 0);
        // The kill's FIRST draw is the sound roll: Rand_Rand(100) < 0x29 → 0x5F2, else 0x5F3
        // [M 0x12b970:65-76 — the roll at 0x12b9b4 sits in the `*piVar14 != 2` branch, after
        // the records gate, before both Sound_Play3D arms and Debris_Create_Loop].
        let mut probe = Rng::new(1);
        let want = if probe.rand(100) < 0x29 { 0x5F2 } else { 0x5F3 };
        assert_eq!(kills[0].sound, want, "the kill rolled Rand_Rand(100) < 0x29 → 0x5f2 else 0x5f3 [M 0x12b970:65-76]");
        // The kill spawned the game's debris: one record → one piece, EffectInfo[0x10].
        assert_eq!(b.shatter.debris.len(), 1, "Debris_Create_Loop: one object per stride-3 record [M 0x1ed548]");
        assert_eq!(b.shatter.debris[0].model, EFFECT_INFO[0x10].model, "EffectInfo[hitdata+0x50 & 0x3f = 0x10] [M 0x12b970:137-141]");
        // The kill's draw stream is the game's, bit-exact: the roll, then Debris_Create's eight
        // per-record draws — FRand(0.1), Rand_Rand(half_life), FRandHalf(spread)×3,
        // FRandHalf_Vec(spin)×3 [M 0x1aaa00:30-50].
        let info = &EFFECT_INFO[0x10];
        let half_life = (info.life_base >> 1) as u32;
        let _ = probe.frand(0.1);
        let _ = probe.rand(half_life);
        for _ in 0..3 { let _ = probe.frand_half(info.spread); }
        for _ in 0..3 { let _ = probe.frand_half(info.spin); }
        assert_eq!((rng.x, rng.y), (probe.x, probe.y),
            "the kill's stream is the roll then the debris draws, bit-exact [M 0x12b970:65-76 + 0x1aaa00:30-50]");
        assert_eq!(rng.draws - draws0, 9, "one roll draw + eight debris draws [M 0x12b970:66 + 0x1aaa00:30-50]");
    }

    #[test]
    fn glass_shatters_with_sound_0x2e_and_consumes_no_roll_draw() {
        // radioroom_glass/comroom_glass carry params [(0,2),(1,0),(2,0)] — kind 2 [D placements].
        let mut b = one(2, 0.0);
        let mut rng = Rng::new(1);
        let draws0 = rng.draws;
        let kills = b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], GUN, 25.0, &mut rng);
        assert_eq!(kills[0].sound, 0x2E, "glass → sound 0x2E [M 0x12b970: uVar5 = 0x2e]");
        // The roll lives inside the `*piVar14 != 2` branch — a glass kill consumes NO draw
        // (the shatter's draws come per real triangle; this source has none) [M 0x12b970:59-66].
        assert_eq!(rng.draws - draws0, 0, "kind 2 → no roll, no debris draws [M 0x12b970:59-66]");
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
        let kills = b.drone_step([5.0, 8.0, -1.5], [5.3, 8.0, -1.5], true, false, 0.0, &mut rng);
        assert_eq!(kills.len(), 1, "0.09 · 20 = 1.8 damage kills the 0-health glass");
        let mut v = one(0, 0.0);
        let mut rng = Rng::new(1);
        assert!(v.drone_step([5.0, 8.0, -1.5], [5.3, 8.0, -1.5], true, false, 0.0, &mut rng).is_empty(),
            "kind != 2 never takes drone damage [M 0x12bfd8:31-34]");
        let mut g = one(2, 0.0);
        let mut rng = Rng::new(1);
        assert!(g.drone_step([5.0, 8.0, -1.5], [5.3, 8.0, -1.5], false, false, 0.0, &mut rng).is_empty(),
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
        assert!(b.drone_step([5.0, 8.0, -1.5], [5.3, 8.0, -1.5], true, true, 0.0, &mut rng).is_empty(),
            "the scripted (root-motion-carried) walk-by reads SqDist(pos, prev) = 0 [M 0x1336f8:212]");
        assert!(!b.list[0].dead, "pane 130 survives g0's scripted walk-by [D the observed run]");
        let kills = b.drone_step([5.0, 8.0, -1.5], [5.3, 8.0, -1.5], true, false, 0.0, &mut rng);
        assert_eq!(kills.len(), 1, "the same crossing by an AI-steered mover drains [M 0x12bfd8:26-27]");
    }

    #[test]
    fn a_body_stopped_against_the_pane_still_drains_it_through_the_rejected_step() {
        // The collision pair (983): the pane is SOLID while alive, so the charger's step is
        // collision-stopped — the port's probe rejects it (SqDist(pos, prev) = 0) where the
        // game's movement pass corrects +0x30 up to the sheet (a nonzero corrected step
        // [M 0x12bfd8:26-27]). The rejected step's length stands in [S breakable.rs drone_step].
        let mut b = one(2, 0.0);
        let mut rng = Rng::new(1);
        // The body AT rest inside the pane's grown reach (the probe stopped him there), step 0,
        // rejected step 0.4: the pair reads and the drain fires.
        let kills = b.drone_step([5.0, 8.0, -1.2], [5.0, 8.0, -1.2], true, false, 0.4, &mut rng);
        assert_eq!(kills.len(), 1, "the pressed body drains through the rejected step [S]");
        assert!(b.list[0].dead);
        // Sister: the same stopped body with NO rejected step (an idle body at rest near the
        // glass) has no pair engagement and never drains.
        let mut c = one(2, 0.0);
        let mut rng = Rng::new(1);
        assert!(c.drone_step([5.0, 8.0, -1.2], [5.0, 8.0, -1.2], true, false, 0.0, &mut rng).is_empty(),
            "a body merely at rest near the pane drains nothing (no rejected step, no sweep)");
        assert!(!c.list[0].dead);
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
        assert_eq!(rng.draws - draws_before, 9, "the sound roll + eight shared draws per debris object [M 0x12b970:66 the roll precedes the debris loop + 0x1aaa00:30-50]");
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

    #[test]
    fn a_script_pane_dies_to_one_bullet_with_the_glass_sound_no_pieces_and_the_object_id() {
        // The gondola's windows: script 0x0600004C streams 7-18/23-26, Entity params [2, 1, 0]
        // [D decode] — kind 2, health 1.0, filter 0 (Break_Create's tag reads [M 0x12c230:21-35
        // via 0x1d8d78:80-87]). Break_Kill on a script pane: dead bit + sound 0x2E, NO pieces —
        // psiCreateBreakableData's only caller is the placement path [M 0x1d04f0:70-73], so
        // psiGetTriList reads count 0 [M 0x12b970:78-133]. No RNG draw either (the sound arm for
        // kind 2 has none [M 0x12b970:64-76]).
        let mut b = Breaks::default();
        let shape: [[[f32; 3]; 3]; 2] = [
            [[2.0, 30.0, -128.0], [2.0, 34.0, -128.0], [2.0, 30.0, -126.0]],
            [[2.0, 34.0, -128.0], [2.0, 34.0, -126.0], [2.0, 30.0, -126.0]],
        ];
        b.script_pane(7, &[2, 1, 0], &shape, [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]);
        assert_eq!(b.list.len(), 1, "the pane registered");
        assert_eq!(b.list[0].kind, 2, "param 0 → tag+0x00 [M 0x12c230:21]");
        assert_eq!(b.list[0].health, 1.0, "param 1's bits [M 0x12c230:22-28]");
        assert_eq!(b.list[0].filter, 0, "param 2 == 0 → filter 0 [M 0x12c230:29-35]");
        let mut rng = Rng::new(0x5eed);
        let draws_before = rng.draws;
        // The pane is the x=2 plane (degenerate in x): the shot runs along −x through it.
        let kills = b.bullet_hit([5.0, 32.0, -127.0], [-1.0, 32.0, -127.0], GUN, 25.0, &mut rng);
        assert!(b.list[0].dead, "health 1.0 - 25 < 0 → Break_Kill [M 0x12bfd8:36-50]");
        assert_eq!(kills.len(), 1, "the kill reports (the sound plays)");
        assert_eq!(kills[0].sound, 0x2E, "kind 2 → 0x2E [M 0x12b970:64-76]");
        assert_eq!(kills[0].kind, 2);
        assert_eq!(kills[0].script_obj, Some(7), "the caller deletes the entity [M 0x12b970:60]");
        assert!(b.shatter.shards.is_empty() && b.shatter.debris.is_empty(), "a script pane spawns NO pieces [M 0x12b970:78-133 + 0x1d04f0:70-73]");
        assert_eq!(rng.draws - draws_before, 0, "no shared-RNG draws on a script-pane kill");
        // A second shot finds a dead pane — no repeat kill.
        let more = b.bullet_hit([5.0, 32.0, -127.0], [-1.0, 32.0, -127.0], GUN, 25.0, &mut rng);
        assert!(more.is_empty(), "dead panes are skipped [M 0x12bfd8:12]");
    }

    #[test]
    fn a_script_pane_rides_its_object_the_refreshed_bounds_take_the_hit() {
        // The windows are parented to the carriage stream: Script_Interp re-poses obj+0x90 every
        // tick [M 0x1d9a08:94-112] and Collide_Intersect tests the CURRENT local space [M
        // 0x1ebaa8:157-177]. The port's refresh: a re-upsert moves the row's bounds; a shot at
        // the OLD position misses, a shot at the NEW position hits.
        let mut b = Breaks::default();
        let unit: [[[f32; 3]; 3]; 1] = [[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]];
        let id = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
        let moved = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, -10.0, 0.0, 1.0]];
        b.script_pane(3, &[2, 1, 0], &unit, id);
        let mut rng = Rng::new(1);
        // The ride moved the object 10 down: the refresh (the caller's per-tick sync).
        b.script_pane(3, &[2, 1, 0], &unit, moved);
        assert!((b.list[0].hi[1] - (-9.0)).abs() < 1.0e-5, "the bounds followed the matrix");
        // A shot at the pre-ride position: no hit (the slab test — the segment crosses nothing).
        assert!(b.bullet_hit([0.5, 0.5, 5.0], [0.5, 0.5, -5.0], GUN, 25.0, &mut rng).is_empty(),
            "the stale position is gone after the refresh");
        // The same segment at the new position kills.
        let kills = b.bullet_hit([0.5, -9.5, 5.0], [0.5, -9.5, -5.0], GUN, 25.0, &mut rng);
        assert_eq!(kills.len(), 1, "the riding pane takes the hit at its new bounds");
    }

    #[test]
    fn a_script_pane_dies_with_its_object_and_a_reregistered_pane_is_fresh() {
        // The kill deletes the object (the caller's removal); the next sync's retain drops the
        // row [M 0x12b970:60 — the object left the world]. A script re-run of the stream creates
        // a fresh entity (and pane) with fresh health.
        let mut b = Breaks::default();
        let shape: [[[f32; 3]; 3]; 1] = [[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]];
        let id = [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
        b.script_pane(5, &[2, 1, 0], &shape, id);
        b.retain_script_panes(&[]);
        assert!(b.list.is_empty(), "the killed object's pane dropped at the sync");
        b.script_pane(5, &[2, 1, 0], &shape, id);
        assert_eq!(b.list.len(), 1, "a re-run stream re-registers");
        assert_eq!(b.list[0].health, 1.0, "fresh health [M 0x1d8968:92-102 — the delete+fresh path]");
    }

    #[test]
    fn the_filter_is_the_weapon_word_against_mask_or_0x106() {
        // Collide_FilterBullets [M 0x1ece30:24-33]: a class-5 hitter whose weapon_definition+0x10
        // word ANDs (filter | 0x106) non-zero is dropped. Disc words [L dump]: guns 0x0008,
        // Laser 0x0002, watch 0x0100. On a no-filter pane the laser and the watch are rejected,
        // the gun passes. The 07000006 security-room pane carries p2 = 0xf8 → filter 0xff07
        // [D placements] — the gun still passes (0xff07's bit 3 is clear).
        let mut b = one(2, 0.0);
        let mut rng = Rng::new(1);
        assert!(b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], 0x0002, 25.0, &mut rng).is_empty(),
            "the laser (0x0002) is rejected by the base mask [M 0x1ece30:26-31]");
        let mut b = one(2, 0.0);
        let mut rng = Rng::new(1);
        assert!(b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], 0x0100, 25.0, &mut rng).is_empty(),
            "the watch (0x0100) is rejected by the base mask [M 0x1ece30:26-31]");
        let mut b = one(2, 0.0);
        let mut rng = Rng::new(1);
        assert_eq!(b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], GUN, 25.0, &mut rng).len(), 1,
            "the gun (0x0008) passes [M 0x1ece30 — 0x0008 & 0x106 == 0]");
        let mut b = one(2, 0.0);
        b.list[0].filter = 0xff07; // !0xf8 [D 07000006 #376 param 2; M 0x12c230:29-35]
        let mut rng = Rng::new(1);
        assert_eq!(b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], GUN, 25.0, &mut rng).len(), 1,
            "the 06 pane's own filter keeps the guns");
        let mut b = one(2, 0.0);
        b.list[0].filter = 0xff07;
        let mut rng = Rng::new(1);
        assert!(b.bullet_hit([5.0, 8.0, -4.0], [5.0, 8.0, -2.0], 0x0002, 25.0, &mut rng).is_empty(),
            "the pane's filter word still rejects its own classes");
    }

    #[test]
    fn a_segment_past_the_sheet_does_not_register_the_adjacent_pane() {
        // The slab rule (b9dfe235): a segment whose AABB spans the bounds but which flies past
        // the sheet is no contact — with the gondola's 16 windows side by side, the old
        // AABB-overlap would kill neighbours. Pane at x 2, segment crossing z -16.33 at x 3.9.
        let mut b = one(2, 0.0);
        b.list[0].lo = [1.9, 30.0, -17.0];
        b.list[0].hi = [2.1, 34.0, -15.0];
        let mut rng = Rng::new(1);
        let kills = b.bullet_hit([3.9, 32.0, -20.0], [3.9, 32.0, -10.0], GUN, 25.0, &mut rng);
        assert!(kills.is_empty() && !b.list[0].dead, "the segment flies past the sheet");
    }
}
