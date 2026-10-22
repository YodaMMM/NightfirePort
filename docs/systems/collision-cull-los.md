# The head-on capsule cull and the LOS query masks (standins-casing-936, 2026-10-22)

Two `world.rs` stand-ins replaced with the game's rules. Both were guesses where the game's
exact test is in the decompile.

## The capsule's head-on cull — `Intersect_CylGeom` 0x109da0:301-322

The port one-sided the capsule-vs-triangle contact for near-horizontal faces keyed on
`|n·y| > 0.7` with the capsule's top end as the side reference — an uncited approximation. The
game's rule, in the VU0 per-triangle block (inside the `|n·d| > 0.0002` branch, `d` the RAW
spine vector, `n` the face's plane normal, `f` the clamped crossing fraction of the spine with
the face's plane):

- skip the triangle iff `(n·d > 0.999 ∧ f > 0.5) ∨ (n·d < −0.999 ∧ f < 0.5)` [M 0x109da0:321-322];
- the horizontality key is the RAW dot, so the cone follows the spine length: the walk spine is
  1.31 long [M Player_Collision 0x194df0 case 0/6], making the game's cone `|n·ŷ| > 0.999/1.31 ≈
  0.763` there — the port's old 0.7 key over-culled a 0.70–0.763 band the game treats
  double-sided (the standard closest-point path);
- the side test is the CROSSING FRACTION, not a position: an up-facing floor is solid when the
  spine crosses near its bottom end (landing, f > 0.5) and passes a jump-through from below
  (crossing near the top, f < 0.5); down-facing ceilings mirror it.

Ported verbatim in `capsule_tri_contact` (the crossing fraction it already computed IS the
game's `f`). Tests: `head_on_cull_uses_the_raw_spine_dot_and_the_crossing_half` (unit) — fails
on the pre-task tree (the f=0.4/solid and the 0.73-band pushes), green on the port; the
walk/swim/escort/stairwell live suites are the real-path guard.

## `line_of_sight` — the `Collide_LineOfSight` 0x1edf50 mask family

`Collide_LineOfSight` is a thin negated forward to `Collide_RayIntersect` with the CALLER's
query mask [M 0x1edf50:4-8]. The port's `line_of_sight(a, b)` ran the RAW segment (no mask
drops) — but every geometry-probe/sight caller's mask carries bit 0x8, the pass-flag drop
(`Collide_Intersect` 0x1ebaa8 drops hits whose node/surface carries the glist flags 0x40 for
them):

- Player_HandleJump 0x1916b0 mask 9, Player_SSCrouch 0x1924c0 mask 9, Player_SSGrapple
  0x192860 mask 8, Sensor_InCone 0x1af4f0 and Check_AutoAim 0x1a73f8 mask 0xa, Bullet_homing
  0x12f358 mask 8 — all carry bit 0x8;
- the sight-class masks add more drops (drone vision 0xa27 [M 0x176f18], copter 0x2e
  [M 0x17dbb8]); none of these masks carries bit 0x20, so the 0x1000 cage and the dynamic set
  stay tested for the geometry-probe family.

The port now runs `segment_bullet_clear` — the same sets (static + missile cage + dynamic)
with the bit-0x8 drop. Behaviour: pass-flagged foliage (07000005's trees carry glist flags
0x41 [D]) no longer blocks LOS for the port's sight/probe callers, as in the game. Tests:
`line_of_sight_drops_the_pass_flagged_foliage` (unit, fails on the pre-task tree) and
`casing936_world_live::los_sees_through_the_discs_pass_flagged_foliage` (real path on
07000005: a raw-blocked, bit-8-clear ray from the spawn grid is LOS-clear; the sister wall
ray stays blocked).

## The re-tagged markers (no behaviour change)

- The synthetic shape key (0x80000000 | map<<16 | object) is the port's internal addressing
  for the game's BY-INDEX glist objects: a placement hashcode of −1 is the game's by-index
  sentinel — `parsemap_block_map_data_dynamic` 0x1d1b68 resolves `pCelList + ref·0x50` and
  never calls `hashtable_getitem` for one; the resolved glist pointer rides the object
  (obj+0xd8). The key packing is port bookkeeping, the (map, object) addressing mirrors the
  game's own.
- `segment_hit_cell`'s static-only query IS the game's rule: the FindCel tie-break ray runs
  with mask 0x90 [M build_FindCel 0x1c8cb0:181-196], bit 0x10 set skips the object walk
  [M Collide_Pick 0x1eb320:314] — no dynamic objects tested there.
- `player_cel_relink` passes POS where the game passes obj+0x80..0x8c — an evidence-cited
  deliberate divergence (kept green for fix-08-collision2's traced 01 pin), now labelled as
  such, not [S].
- `segment_blocked`'s early-out is bit-identical to the nearest-hit search by construction
  (the existential closure of the same per-triangle predicate).
- casing.rs: the material byte IS the game's read (`*(byte *)(*(int *)(shooter+0xdc) + 0x50) &
  0x3f` [M 0x12f8f8]); the `eject()` yaw/pitch path is the port-only fallback (headless
  callers, MP remote slots) — the game always ejects from `AnimGetBoneWorldTrans`'s matrix at
  the delay's expiry [M 0x12f678:83-114], which the port's real loop (reserve →
  `eject_bone_now` / guard `eject_matrix`) already runs; the keep-option "rest" is a port-only
  extension, unreachable in the shipped game (no keep option; class 0x0d has no collide
  handler); the landing sound fires once [M 0x12fb88-0x12fb94]; the pool test's message now
  cites the game's fixed 32 [M 0x12fd48] as the contrast.
