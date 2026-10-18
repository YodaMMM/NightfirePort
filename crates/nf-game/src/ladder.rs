//! The ladder (`Ladder_Create` 0x1966c0, placement type 0x22, class 0x2D).
//!
//! `Ladder_Create` [M disasm 0x1966c0-0x19675c] calls `control_create_object(4, pos, rot, 0)` — instance size
//! 4 bytes: `inst+0x00 = 0` (`sh zero,0(v0)`), `inst+0x02 = *(u16*)(level_tag+0x30)` = placement param 1
//! (`lhu v1,48(s1)` / `sh v1,2(v0)`); `obj+0xFF = 0x2D` (`addiu a2,zero,45` / `sb a2,255(s0)`);
//! `obj+0xD8 = celglist`; `build_LinkToRoom(obj,0,glb_world)`; `Controls_StraddleTest(obj)`.
//! The rotation quaternion is passed through **unmodified** — the wire atlas's "rot.z is negated … as for
//! ladders" side note is wrong for ladders (no negation in `Ladder_Create` or its dispatch case [M
//! 0x1d04f0 case 0x22]). No param is ever read back by any ladder code (only the wire's channel pattern
//! matches this instance layout), so the port keeps `channel` for completeness.
//!
//! The runtime geometry comes from the object's world sphere (`Control_BuildWorldSph` 0x133070 pattern, the
//! same computation `Bond::sphere_centre` ports): `world = local_centre` (the glist bounds centre,
//! `MapObject.centre`) through the object matrix + translation, `radius = MapObject.radius * |scale|`
//! [M centre/radius = glist+0x20/+0x30 per docs/formats/level.rs; the ladder sphere's +0x84 (centre y) and
//! +0x8C (radius) are what `Player_CollLadder` 0x196118 and `Player_Climb` 0x196310 clamp against].
//!
//! Spec: docs/atlas/ladder.md (this session), docs/systems/player.md.

use crate::world::V3;
use nf_formats::level::{Level, MapObject};
use nf_formats::placement::Placement;

/// A ladder object (class 0x2D, `Ladder_Create` 0x1966c0).
#[derive(Clone, Debug)]
pub struct Ladder {
    /// `obj+0x80..0x8C`: the world-sphere centre (`Control_BuildWorldSph` 0x133070).
    pub centre: V3,
    /// `obj+0x8C`: the world-sphere radius.
    pub radius: f32,
    /// `obj+0x54`: the yaw from the object matrix's forward row (row 2 = (sin y, ·, cos y)).
    pub yaw: f32,
    /// `inst+0x02` = placement param 1 (`level_tag+0x30`) [M 0x196710-0x196714]; no reader found.
    pub channel: u16,
    /// The glist bounds box half-extents in the ladder's own frame, `[lateral, frontal]` (object-space
    /// x and z of `MapObject.min/max` × scale, about the bounds centre) — the pickable mesh's footprint
    /// [M 07000002's hut ladder `Ladder_medium`: x ±0.354, z ±0.060, and its 0x2E collision triangles
    /// span the same box]. The pick's stage 2 measures the capsule against this box.
    pub half: [f32; 2],
    /// The same box's world y span `[min, max]` (translation.y + min.y / max.y × scale).
    pub y_span: [f32; 2],
}

impl Ladder {
    /// `Ladder_Create` (0x1966c0) from a placement (type 0x22) plus the placement's object bounds.
    /// The yaw convention matches `Bond::look` (0 faces +Z, forward = (sin, ·, cos)); the object matrix's
    /// yaw-only rotation carries the glist bounds centre into the world.
    pub fn create(p: &Placement, obj: Option<&MapObject>) -> Option<Ladder> {
        let o = obj?;
        let yaw = p.rotation[1];
        // G-046: the object-matrix yaw rows are PS2Sinf-built [M the placement matrix; Control_BuildWorldSph
        // 0x133070 consumes it] — the port's rebuild follows (nf-formats::placement::object_matrix already
        // runs anim::sinf).
        let (s, c) = (crate::trig::sinf(yaw), crate::trig::cosf(yaw));
        // row-vector matrix rows: row0 = (cos,0,−sin), row2 = (sin,0,cos) [M placement::object_matrix].
        let centre = [
            p.translation[0] + o.centre[0] * c + o.centre[2] * s,
            p.translation[1] + o.centre[1],
            p.translation[2] - o.centre[0] * s + o.centre[2] * c,
        ];
        let scale = p.scale();
        let s_max = scale.iter().fold(1.0f32, |a, &v| a * v.max(1e-6).abs());
        let half = [
            0.5 * (o.max[0] - o.min[0]) * scale[0].abs(),
            0.5 * (o.max[2] - o.min[2]) * scale[2].abs(),
        ];
        let y_span = [p.translation[1] + o.min[1] * scale[1].abs(), p.translation[1] + o.max[1] * scale[1].abs()];
        Some(Ladder { centre, radius: o.radius * s_max, yaw, channel: param1(p), half, y_span })
    }

    /// A hand-built ladder for unit tests: a rung box ±0.354 wide, ±0.06 deep (the shipped ladder
    /// meshes' footprint [M 07000002]) spanning the sphere's height.
    pub fn simple(centre: V3, radius: f32, yaw: f32) -> Ladder {
        Ladder { centre, radius, yaw, channel: 0, half: [0.354, 0.06], y_span: [centre[1] - radius, centre[1] + radius] }
    }

    /// `BLData+0x8A0` = `centre.y + radius + 0.85` — the top dismount threshold [M Player_CollLadder
    /// 0x196118 (`+0x84 + +0x8C + 0.85`), recomputed per tick in `Player_Climb` 0x196310].
    pub fn top(&self) -> f32 { self.centre[1] + self.radius + 0.85 }
    /// `BLData+0x8A4` = `centre.y − radius + 1.0` — the bottom dismount threshold [M same functions].
    pub fn bottom(&self) -> f32 { self.centre[1] - self.radius + 1.0 }

    /// The CLIMB/CREEP form of the pick (`Player_Collision` 0x194df0's capsule switch case 0/6 [M
    /// — the switch indexes on substate−1 and its jump table @0x2fc0d0 maps case 0 → substate 1
    /// (climb) and case 6 → substate 7 (creep); NOT the walk 0]). Two stages:
    ///
    /// Stage 1 — the game's pick sphere. `Collide_PickObj` 0x1eae80:152-168 [M] tests the player's
    /// HITTEST capsule segment against the object's world sphere inflated ×1.5 plus the capsule
    /// radius (mode word BLData+0x7D4 = 0x800). The climb arm's HITTEST [M 0x1950a0]:
    /// radius `+0x7CC = 0x3e8ccccd` (0.275 — `player::WALK_PICK_RADIUS`), segment
    /// A = pos + up·0.55 (0x3f0ccccd) and B = pos + up·(+0x7CC − cc) — the game reads the B offset's
    /// base FROM the same radius slot it just wrote, so `b = pos + (capsule_radius − cc)` here.
    /// With the stand cc 1.0347 the segment spans 0.55..−0.76. The port's climb tick runs no grab
    /// scan (`Player_CollLadder` 0x196118:16-17 excludes substates 1/2 [M]), so this shape's live
    /// role is the unit pins of the switch's own geometry; the WALK grab rides
    /// `capsule_hits_default` (ladder-0c).
    ///
    /// Stage 2 — the rung-plane bound. The game's pick reach is narrowed past the sphere by the
    /// straddled-cel scoping (`Collide_Pick` 0x1eb320:125-134/318-328 tests only objects linked to the
    /// cels the player straddles) and the glist mesh refine (`Collide_Intersect` 0x1ebaa8, run per
    /// candidate in `Collide_Update` 0x1ea398:84-93, not ported): a tall ladder's bounds sphere is its
    /// half-HEIGHT (r 5.75 on the 0700000c yard ladder, y span 0.8..12.1 [D probes]), so the bare ×1.5
    /// sphere reaches ~9 units sideways, and a ladder's pickable body is its thin rung plane through
    /// the axis plus the rails' lateral span. The port models the refine as the capsule against the
    /// mesh's bounds box in the ladder's frame ([S] the stand-in for cel scoping + mesh refine; the
    /// box itself is [M], `in_mesh_box`): the capsule reaches the rung box (±0.06 deep, ±0.354 wide
    /// on the shipped ladders) and no farther. The yaw window in
    /// `Player_CollLadder` 0x196118:20 gates the sides (a walker brushing past faces across the
    /// ladder, |diff| = π/2, no grab).
    pub fn capsule_hits(&self, pos: V3, capsule_radius: f32, cc: f32) -> bool {
        self.pick_hit(pos, 0.55, capsule_radius - cc, capsule_radius)
    }

    /// The DEFAULT arm of the pick — THE WALK GRAB'S ARM (`Player_Collision` 0x194df0's capsule
    /// switch DEFAULT case [M]: the switch indexes on substate−1 (`lhu 0xf6`, `addiu −1`,
    /// `sltiu $v1, $a0, 0xf` unsigned bounds) and its jump table @0x2fc0d0 routes the walk 0 (index
    /// −1 → the bounds check fails on 0xffffffff), the grapple 2, the swim 3 and the substates
    /// 5/8-11 to 0x1953f4): radius `+0x7CC = 0x3f0ccccd` (0.55), segment A = pos + up·0.275
    /// (0x3e8ccccd), B = pos + up·(0.55 − cc). ladder-0c: the game's walk substate 0 rides THIS
    /// arm, so Bond's body stopped against the rung plane (the walk capsule's own 0.55 vs the
    /// ladder's collision triangles — the wall behind the 0700000c yard ladder's plane sits ~0.05
    /// recessed [D probes], stopping the walker at 0.4985) is still INSIDE the grab reach (0.4985
    /// < 0.55) and the walk-in grabs — the fix-ladder-water2 walk mapping (0.275 for substate 0)
    /// read the switch on the raw substate and left that ladder unclimbable (playtest 2026-10-18
    /// "the ladder doesnt work"). The dismount escape still clears the fatter arm: the dismount
    /// hop −0.05 [M Player_Climb 0x196310:88-96, consumed the same tick by the integration
    /// Player_Update 0x193600:284-295] leaves the stander 0.55 off a FLAT rung plane (the yard
    /// ladder's mesh is z-degenerate [D bounds]) and the refine's strict `dist < radius` [M
    /// Intersect_CylGeom 0x109da0:447] rejects the exact touch; the swim escape's drift (0.5 +
    /// 0.05 + 0.2 > 0.55) still clears. Stage 1/stage 2 as in `capsule_hits`.
    pub fn capsule_hits_default(&self, pos: V3, capsule_radius: f32, cc: f32) -> bool {
        self.pick_hit(pos, 0.275, 0.55 - cc, capsule_radius)
    }

    /// The two pick stages over one arm's segment: stage 1 — the object's world sphere inflated
    /// ×1.5 plus the capsule radius against the closest point on the segment A..B
    /// [`Collide_PickObj` 0x1eae80:152-168, mode word BLData+0x7D4 = 0x800]; stage 2 — the
    /// mesh box [`in_mesh_box`, the stand-in for the glist refine
    /// `Collide_Update` 0x1ea398:84-93 → `Collide_Intersect` 0x1ebaa8].
    fn pick_hit(&self, pos: V3, a_off: f32, b_off: f32, capsule_radius: f32) -> bool {
        let a = [pos[0], pos[1] + a_off, pos[2]];
        let b = [pos[0], pos[1] + b_off, pos[2]];
        let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let ap = [self.centre[0] - a[0], self.centre[1] - a[1], self.centre[2] - a[2]];
        let ab2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
        let t = if ab2 < 1e-9 { 0.0 } else { (ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2]) / ab2 };
        let t = t.clamp(0.0, 1.0);
        let cx = a[0] + ab[0] * t - self.centre[0];
        let cy = a[1] + ab[1] * t - self.centre[1];
        let cz = a[2] + ab[2] * t - self.centre[2];
        // Strictly inside: the game's pick rejects when `(r·1.5 + cap)² <= dist²` [M Collide_PickObj
        // 0x1eae80:166 `if (fVar12 * fVar12 <= fVar11) return;`].
        if (cx * cx + cy * cy + cz * cz).sqrt() >= self.radius * 1.5 + capsule_radius { return false; }
        self.in_mesh_box(pos, a_off.max(b_off), a_off.min(b_off), capsule_radius)
    }

    /// The CROUCH form of the pick (`Player_Collision` 0x194df0's capsule switch case 3 [M] — the
    /// switch indexes on substate−1 [M disasm 0x195070], so case 3 IS the crouch substate 4): radius
    /// `+0x7CC = 0x55` (0.55) and the segment A = pos + up·(0.1 − (cc − 0.55)), B = pos + up·(0.45 − cc)
    /// — a short LOW capsule ~0.48 under the root at the stand height. (Named `capsule_hits_swim`
    /// before 750a mis-read case 3 as the swim substate; the swim substate 3 rides the switch's
    /// DEFAULT arm [M 0x195070 default].) Stage 1 is the same ×1.5 sphere test; stage 2 is the same
    /// mesh box (`in_mesh_box`). The low segment bounds the grab by DEPTH: a bed-hugging
    /// root too far under the bounds centre misses the ×1.5 sphere.
    pub fn capsule_hits_crouch(&self, pos: V3, capsule_radius: f32, cc: f32) -> bool {
        self.pick_hit(pos, 0.1 - (cc - 0.55), 0.45 - cc, capsule_radius)
    }

    /// Stage 2, shared by every pick form: the capsule (the vertical segment pos.y + lo ..
    /// pos.y + hi, radius `capsule_radius`) against the ladder mesh's bounds box in the ladder's frame
    /// (`fwd(yaw) = (sin, ·, cos)`, the grab snap's convention [M 0x196118:36-44]; lateral = the rungs'
    /// width, frontal = their depth) — the stand-in for the glist refine (`Collide_Update`
    /// 0x1ea398:84-93 → `Collide_Intersect` 0x1ebaa8 → `Intersect_CylGeom` 0x109da0, not ported) and
    /// the straddled-cel scoping. The box is the mesh's own [M `half`/`y_span`]; the boundary is
    /// STRICT: the game's cylinder refine tracks a triangle only when its distance is strictly below
    /// the capsule radius [M Intersect_CylGeom 0x109da0:447 `if (fVar30 < *(float *)(param_1[8] +
    /// 0xc))`] — the touch at exactly the radius is no hit, which is what keeps the just-dismounted
    /// stander (snap 0.5 + the one-tick hop −0.05 [M Player_Climb 0x196310:88-96 + Player_Update
    /// 0x193600:284-295]) 0.55 off a FLAT rung plane (the 0700000c yard ladder's mesh is z-degenerate
    /// [D bounds]) from re-grabbing. The pre-1016 port used a guessed ±0.4 rail span with
    /// zero depth, which reached 0.95 sideways for the swim's 0.55 capsule where the mesh gives 0.904 —
    /// the 02 hut-pocket swimmer at 0.955 slid along that edge and grabbed where the PS2 swims past
    /// (PCSX2 2026-10-16, yaw 0.4/0.7 from (-76.27, 3.17, 22.14)). See `capsule_hits_default`'s doc
    /// for the walk-arm mapping.
    fn in_mesh_box(&self, pos: V3, hi: f32, lo: f32, capsule_radius: f32) -> bool {
        let dx = pos[0] - self.centre[0];
        let dz = pos[2] - self.centre[2];
        let (s, c) = self.yaw.sin_cos();
        let frontal = dx * s + dz * c;
        let lateral = dx * c - dz * s;
        let ox = (lateral.abs() - self.half[0]).max(0.0);
        let oz = (frontal.abs() - self.half[1]).max(0.0);
        let oy = (self.y_span[0] - (pos[1] + hi)).max((pos[1] + lo) - self.y_span[1]).max(0.0);
        ox * ox + oz * oz + oy * oy < capsule_radius * capsule_radius
    }
}

/// Placement param 1: `params.iter().find(|(i, _)| *i == 1)` [M the params are (index, value) pairs,
/// `level_tag+0x2C+4k` addressed by INDEX (docs/systems/levels.md §2 pickup note); `Ladder_Create` reads
/// the word at `level_tag+0x30` = param 1].
fn param1(p: &Placement) -> u16 { p.params.iter().find(|(i, _)| *i == 1).map_or(0, |&(_, v)| v as u16) }

/// Every ladder placement of a level (`flags & 0xff == 0x22`), resolved against its map object.
/// Mirrors the wire sweep in nf-view's `Scene::build` [M 0x1d04f0 case 0x22 → Ladder_Create].
pub fn all(level: &Level) -> Vec<Ladder> {
    let main = level.main_map();
    let mut out = Vec::new();
    for (mi, m) in level.maps.iter().enumerate() {
        for p in &m.placements {
            if p.flags & 0xff != 0x22 { continue; }
            let obj = if p.by_hash() { level.by_hash.get(&p.hashcode).copied() }
                else if (p.index as usize) < m.objects.len() { Some((mi, p.index as usize)) } else { None };
            let obj = obj.and_then(|(om, oi)| level.maps.get(om).and_then(|map| map.objects.get(oi)));
            if let Some(l) = Ladder::create(p, obj) { out.push(l); }
        }
    }
    let _ = main;
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ladder(yaw: f32) -> Ladder {
        Ladder::simple([10.0, 3.0, 20.0], 1.5, yaw)
    }

    #[test]
    fn thresholds_match_the_decompile_constants() {
        let l = ladder(0.0);
        // top = 3.0 + 1.5 + 0.85 [M 0x196118]; bottom = 3.0 − 1.5 + 1.0.
        assert!((l.top() - 5.35).abs() < 1e-6, "{}", l.top());
        assert!((l.bottom() - 2.5).abs() < 1e-6, "{}", l.bottom());
        // Sister: a different radius moves both thresholds.
        let l2 = Ladder::simple([10.0, 3.0, 20.0], 2.5, 0.0);
        assert!((l2.top() - l.top()).abs() > 0.5 && (l2.bottom() - l.bottom()).abs() > 0.5);
    }

    #[test]
    fn grab_zone_is_the_game_pick_shape() {
        let l = ladder(0.0);
        let cc = 1.0347f32;
        let wr = crate::player::WALK_PICK_RADIUS; // 0.275 [M the climb/creep arm 0x1950a0 via the jump table @0x2fc0d0]
        // On the rung plane at mid height: the climb capsule (r 0.275) touches it — hit (stage 1 dist
        // ≈ 0.6 ≤ 1.5·1.5 + 0.275; stage 2 frontal 0.2 < 0.275).
        assert!(l.capsule_hits([10.0, 3.0, 20.2], wr, cc));
        // Sister: 0.4 off the plane is past the climb capsule's reach (0.4 > 0.275) — the climb/creep
        // arm never touches there [M 0x1950a0]. (The WALK grab does — see the default-arm pins below.)
        assert!(!l.capsule_hits([10.0, 3.0, 20.4], wr, cc));
        // 2 units to the side: the bare stage-1 sphere reaches (2.0 < 2.775) [M 0x1eae80:152-168], but the
        // game narrows the reach past the sphere (the straddled-cel scoping Collide_Pick 0x1eb320:125-134
        // + the glist mesh refine Collide_Intersect 0x1ebaa8 — both unported); the port's stage-2
        // mesh-box bound stands in: lateral 2.0 − 0.354 > 0.275 → no hit. (The bare stage 1 here is what
        // re-grabbed Bond off the 0700000c ladder top — the 102h regression.)
        assert!(!l.capsule_hits([12.0, 3.0, 20.0], wr, cc));
        // 4 units to the side: beyond both stages: no hit.
        assert!(!l.capsule_hits([14.0, 3.0, 20.0], wr, cc));
        // DEEP below the sphere bottom (the climb-out assist): the climb segment (down to
        // pos + 0.275 − cc = pos − 0.76 [M 0x1950a0 reads the B base from the radius slot]) against
        // the ×1.5 sphere reaches — the grab y-snap then lifts him to bottom + 0.02 [M 0x196118:23-31].
        assert!(l.capsule_hits([10.0, 0.9, 20.0], wr, cc));
        // Sister: a shallow radius ladder stops reaching deep (the pick tracks the sphere).
        let thin = Ladder::simple([10.0, 3.0, 20.0], 0.6, 0.0);
        assert!(!thin.capsule_hits([10.0, 0.9, 20.0], wr, cc), "the inflated pick reached a thin ladder's bed depth");
        // Sister (the swim2 regression, the 0700000c yard ladder's real geometry — y span 0.8..12.1
        // [D probes] → cy 5.55, r 5.75): the top-exit dismount leaves Bond ~2.9 from the axis at
        // walkway height when regrab expires. The bare stage-1 sphere HITS him there (dist 6.4 < 9.03)
        // and yanked him back into the climb; the stage-2 column bound rejects.
        let yard = Ladder::simple([9.7, 5.55, -187.3], 5.75, 0.0);
        // Walking INTO the ladder at walkway height (crossing the rung plane) still grabs — the
        // climb capsule's own reach [M 0x1950a0] (and the walk arm's, which is wider).
        assert!(yard.capsule_hits([9.7, 11.736, -187.1], wr, cc), "the walk-on grab at the walkway height must still reach");
        assert!(!yard.capsule_hits([9.285, 11.736, -190.167], wr, cc),
            "the dismounted walker 2.9 from the axis re-grabbed (the 102h limit cycle)");
    }

    /// ladder-0c: the WALK substate 0 rides the switch's DEFAULT arm [M the jump table @0x2fc0d0:
    /// substate−1, walk 0 → −1 → the unsigned bounds check fails → 0x1953f4, r 0.55]. These pins
    /// replace fix-ladder-water2's walk-arm units, which mapped the switch on the raw substate and
    /// pinned the 0.275 climb arm for the walk — the 0700000c yard ladder #164 (a z-degenerate rung
    /// plane [D bounds], the wall ~0.05 recessed behind it) stopped the walker at 0.4985 and the
    /// 0.275 arm never touched: the user's "the ladder doesnt work" (playtest 2026-10-18).
    #[test]
    fn walk_grab_is_the_default_arm_the_jump_table_routes_substate_zero_to_0x1953f4() {
        let cc = 1.0347f32;
        let r = crate::player::RADIUS;
        // The 0700000c yard ladder's DISC geometry [D placement #164 (9.6697, 5.5266, −187.3427)
        // yaw 0.5934 + its map object: bounds x ±0.576195, y ±5.73384, z degenerate (−0.0..−0.0),
        // sphere r 5.7627 → the port's centre/radius/half/y_span below].
        let yard = Ladder {
            centre: [9.669741, 5.526596, -187.3427],
            radius: 5.762721,
            yaw: 0.5934119,
            channel: 0,
            half: [0.576195, 0.0],
            y_span: [-0.20724726, 11.260439],
        };
        // The walker's stop: the port's walk capsule (r 0.55) rests Bond at frontal 0.4985 (the wall
        // recess behind the plane) — the default arm's stage 2 reaches it (0.4985 < 0.55) [M
        // 0x1953f4 + Intersect_CylGeom 0x109da0:447 strict], so the walk-in grabs.
        assert!(yard.capsule_hits_default([9.92, 1.02, -186.91], r, cc),
            "the walked-in yard ladder base (0.4985 off the plane) must grab under the default arm");
        // Sister that FAILS on the pre-ladder-0c code: the same spot under the old 0.275 walk arm
        // (the climb arm) never touches — pinned here so the arm mapping can't silently regress.
        assert!(!yard.capsule_hits([9.92, 1.02, -186.91], crate::player::WALK_PICK_RADIUS, cc),
            "the 0.275 climb arm must NOT reach the walker's stop (that's the playtest's broken ladder)");
        // The grab-snap spot (0.5 in front of the plane [M 0x196118:36-44]): inside the default arm.
        assert!(yard.capsule_hits_default([9.9493, 5.5266, -186.9282], r, cc));
        // The just-dismounted stander: snap 0.5 + the one-tick hop −0.05 [M Player_Climb 0x196310:88-96
        // + Player_Update 0x193600:284-295] = 0.55 off the FLAT plane — the strict refine boundary [M
        // Intersect_CylGeom 0x109da0:447 `dist < radius`] rejects the exact touch. The 0.55 spot is
        // the [M] shape; the exact f32 landing is the port's own bit-exact trig + hop (below, the
        // game-shaped construction via the port's ee ops).
        let a = crate::trig::ee_add(yard.yaw, crate::trig::PI);
        let mul = nf_formats::ps2float::mul;
        let snap = [
            crate::trig::ee_sub(yard.centre[0], mul(crate::trig::sinf(a), 0.5)),
            11.736,
            crate::trig::ee_sub(yard.centre[2], mul(crate::trig::sinf(crate::trig::ee_add(a, crate::trig::HALF_PI)), 0.5)),
        ];
        // The snap IS inside the arm (0.5 < 0.55): the stander only escapes once the hop carries
        // him past the strict boundary.
        assert!(yard.capsule_hits_default(snap, r, cc), "the grab-snap spot (0.5 off the plane) must grab");
        // 0.56 out: past the strict boundary — no grab.
        let out = [snap[0] - 0.06 * crate::trig::sinf(a), 11.736, snap[2] - 0.06 * crate::trig::sinf(crate::trig::ee_add(a, crate::trig::HALF_PI))];
        assert!(!yard.capsule_hits_default(out, r, cc), "0.56 off the flat plane is past the strict 0.55 reach");
        // Lateral: the walkway strip 0.67 off the plane (the real walkway edge [D probes]) — no re-grab
        // after the top mantle.
        assert!(!yard.capsule_hits_default([9.3, 11.086, -187.9], r, cc),
            "the walkway edge 0.67 off the plane must not re-grab");
        // The simple-ladder walk-on shapes: 0.4 off the plane grabs under the default arm (0.4 < 0.55)
        // where the old walk arm missed — the game's stander 0.4 off the plane re-latches on approach.
        let l = ladder(0.0);
        assert!(l.capsule_hits_default([10.0, 3.0, 20.4], r, cc));
        assert!(!l.capsule_hits_default([12.0, 3.0, 20.0], r, cc), "lateral 2.0 − 0.354 stays past the stage-2 box");
        // The deep walk segment: B = pos + (0.55 − cc) = pos − 0.485 reaches the ×1.5 sphere at the
        // root 0.9 (closest point dist ≈ 1.83 < 1.5·1.5 + 0.55).
        assert!(l.capsule_hits_default([10.0, 0.9, 20.0], r, cc));
        let thin = Ladder::simple([10.0, 3.0, 20.0], 0.6, 0.0);
        assert!(!thin.capsule_hits_default([10.0, 0.9, 20.0], r, cc), "the thin ladder's sphere stops the deep reach");
    }

    #[test]
    fn crouch_pick_is_the_game_low_capsule_with_the_mesh_scale_reach() {
        let l = ladder(0.0); // centre (10, 3, 20), r 1.5 → bottom 2.5, top 5.35
        let cc = 1.0347f32; // the stand root height [M Player_Collision 0x194df0 cc = AnimObj+0xCC]
        // The LOW (crouch) segment [M 0x194df0 case 3]: A = pos + 0.1 − (cc − 0.55) = pos − 0.3847,
        // B = pos + 0.45 − cc = pos − 0.5847. On the axis at the grab-snap distance (0.5, the game's
        // own snap offset 0x196118:36-44): stage 1 (dist ≈ 1.17 ≤ 1.5·1.5 + 0.55) and stage 2
        // (0.5 ≤ 0.55) both hit.
        assert!(l.capsule_hits_crouch([10.0, 3.0, 20.5], 0.55, cc));
        // 0.62 off the plane: past the mesh-scale stage 2 (0.55) → no hit.
        assert!(!l.capsule_hits_crouch([10.0, 3.0, 20.62], 0.55, cc));
        // DEEP below the sphere bottom (a bed-hugging root ~2.0): the low segment still reaches
        // the ×1.5 sphere (dist ≈ 2.05 ≤ 2.8).
        assert!(l.capsule_hits_crouch([10.0, 2.0, 20.3], 0.55, cc));
        // Sister: a thin ladder stops the deep reach (the pick tracks the sphere): at the root
        // 0.9 the low segment (0.52..0.32) sits 2.49 from the sphere centre — past the thin ×1.5
        // reach 1.45, while the full ladder still reaches (2.49 ≤ 2.8).
        let thin = Ladder::simple([10.0, 3.0, 20.0], 0.6, 0.0);
        assert!(!thin.capsule_hits_crouch([10.0, 0.9, 20.0], 0.55, cc));
    }

    #[test]
    fn swim_pick_is_the_game_default_arm_the_switch_indexes_substate_minus_one() {
        let l = ladder(0.0); // centre (10, 3, 20), r 1.5 → bottom 2.5, top 5.35
        let cc = 1.0347f32;
        // The swim grab rides the switch's DEFAULT arm [M the jump table @0x2fc0d0: swim 3 → index 2
        // → 0x1953f4]: r 0.55, A = up·0.275, B = up·(0.55 − cc).
        // At the grab-snap offset 0.5 [M 0x196118:36-44] it reaches (stage 2: 0.5 < 0.55).
        assert!(l.capsule_hits_default([10.0, 3.0, 20.5], crate::player::RADIUS, cc));
        // The dismount-into-water escape (109i): the back-stick drift (~0.2 over the swim entry's
        // 5-tick regrab [M 0x195e50]) clears the plane cut: 0.5 + 0.05 (the hop) + 0.2 > 0.55.
        assert!(!l.capsule_hits_default([10.0, 3.0, 20.62], crate::player::RADIUS, cc));
    }
}
