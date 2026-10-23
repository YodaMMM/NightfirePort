# The aim ray's door pick — the game's front-face mesh ray (standins-door-933, 2026-10-22)

`Doors::pick_target` — the per-door arm of `Check_Target` 0x1a7810's aim ray (the 0x200000b
"Press Action to open door" hint's door test, `Door_DoMsg` 0x135f18). This replaces the port's
world-sphere pick with the game's own mesh test.

## The game's chain

- `Check_Target` 0x1a7810 aims a 50-unit ray (HITTEST A/B) and calls `Collide_RayIntersect`
  [M disasm 0x1a79ec-0x1a7a20: a0 = HITDATA+0x58, the hit object; the door arm dispatches on
  class `obj+0xff` == 0x1c].
- `Collide_RayIntersect` 0x1ec690 builds a mask-0x201 HITTEST (A at +0x20, B at +0x30, mask
  0x201 [M :95-112]) and walks the straddled cels' objects, calling `Collide_Intersect`
  [M :163-166], then sorts the HITDATA nodes by the +0xc fraction, dropping 1e8 no-hits
  [M :114-160].
- `Collide_Intersect` 0x1ebaa8's 0x201 arm transforms the ray's A and B into OBJECT space with
  the inverse of the object's live matrix `obj+0x90` (`Mat_Copy`/`Mat_Inverse` +
  `RotPreTransVec`) [M :157-161, 193-229] and calls `Intersect_RayGeom` [M :229].
- `Intersect_RayGeom` 0x10a8c8 is a one-sided ray: a triangle whose plane denominator is
  non-negative is skipped (`n·dir >= 0` → skip [M :168]), the fraction `t = −num/den` is
  accepted only in [0, 1] [M :169-172], and the hit point must lie inside the triangle
  (three edge tests [M :200-245]). The stored winding IS the facing — the plane normal is
  +cross(v1−v0, v2−v0) [D, the identity `world.rs`'s static queries already rely on].

Consequences the old stand-in got wrong: the old pick tested each door's LIVE WORLD SPHERE
(obj+0x80/obj+0x8C) and clamped the entry fraction to 0 when the eye was already inside the
sphere — reporting the door for ANY ray from inside the sphere, including one pointing AWAY
from the leaf. The game's ray reports the door only when the ray actually crosses a front
face: a ray that starts inside the closed leaf hits nothing (exit faces have n·dir > 0), and
a ray pointing away from the leaf never crosses it. The two agree when the eye inside the
sphere aims at the door; they disagree aiming away — the old clamp burned the hint latch
(one attempt per door ever, `Door_DoMsg`'s +0x74) for a door the player never aimed at.

## The port now

`pick_target(&self, world, eye, dir, wall_t)` (the caller passes the collision world):
for each door with a resolved glist (the game's ray only tests objects that have one — the
same admission as the use pick [M 0x1eae80:88-90]), transform `eye` and `eye + 50·dir` into
the door's object space with the inverse of the LIVE pose matrix (`rigid_inverse` +
`transform` — the game's `RotPreTransVec` with an inverse pose [M 0x1ebaa8:157-161]), and run
the front-face Möller–Trumbore test over the glist's own collision triangles
(`world.shape(glist)` — the same data `set_dynamic` poses for the door's solidity), fraction
in [0, 1], nearest across doors [M 0x1ec690:114-160 sorts by fraction]. The fraction of the
50-unit ray is the returned hit fraction; `wall_t` still caps the pick by the caller's
static-segment fraction (the game's ray is occluded by the same world walk).

Nearest-door selection, the 50-unit ray, the wall cap and the caller wiring
(scene.rs's Check_Target block) are unchanged — only the per-door shape test changed.

## Tests

`crates/nf-view/tests/door_aim_ray_live.rs` (NF_ISO-gated, real doors through
`nf_view::load`):

- `aim_ray_inside_the_sphere_away_reports_nothing` — the eye parked inside a shape-backed
  door's live world sphere (half the radius out along the through-axis), ray pointing away
  from the leaf: the pick is None. The deleted stand-in returned the door here (entry
  clamped to 0) — this test fails on the earlier code.
- Sister `aim_ray_inside_the_sphere_at_the_leaf_still_reports_the_door` — the same eye aiming
  at the leaf: the pick returns that door with a fraction in (0, 1) — the front-face hit the
  game reports, and the case the sphere pick got right.
