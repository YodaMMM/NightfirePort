# s/touch-ref-point

**The random stream does not move.** The shipped trigger contact is unchanged. This branch ports
the game's full on-foot contact law and measures it, but it does not switch to it. Switching broke
24 walk and plate pins (listed below).

Based on `s/touch-mesh-contact`.

## What is ported

`triggers::cyl_geom_min` runs `Intersect_CylGeom` 0x109da0 over one glist, as the walk HITTEST
pick drives it. It applies these rules per triangle, in glist order [M, asm cited in the doc]:

- **Query point.** With |n·(B−A)| > 0.0002 (0x3951B717) the query point is the clamped plane
  crossing. The head-on skip uses the 0.999 constant (0x3f7fbe77). Otherwise the query point is
  the triangle centroid projected onto the segment.
- **Dist2Tri.** `ASH_vecutil_Dist2Tri` 0x109770 (`world::game_dist2tri`) runs with the record
  plane. Code 99 is behind the plane. The closest point goes to HITTEST+0x50.
- **Re-anchor.** An edge or vertex code moves the query point to the segment point nearest
  HITTEST+0x50. The distance stays the first query's.
- **Raw branch.** This is HITDATA+0x51 & 6, set from obj+0xF0 bit 0x40 by `Collide_Intersect`
  0x1ebaa8:93-99. The Dist2Tri distance is tracked as is.
- **Reference-point law.** This covers every other glist. v = P − HITTEST+0x50 must satisfy
  n·v > 0. The distance becomes |v| and must be < r. The segment is then pushed out by r − |v|
  before the next triangle.
- **Tracking.** The tracked distance is the minimum, seeded at r (strict).

Other pieces:

- `Trigger::raw` carries the glist's bit 0x40.
- `Trigger::game_contact` runs the law for one volume.
- `triggers::GAME_TOUCHER` marks the walk HITTEST under this law.
- `drone::game_touchers` is the whole on-foot set. It has one contact (the pick arm's segment)
  plus the ride reach. It has no box and no containment, because `Trigger_Hit` 0x1b1fc8 reads only
  the pick's contact list.
- Script-made triggers now carry their glist's plane records (posed with `world::plane_world`)
  and the bit-0x40 flag (`CollisionWorld::shape_planes` / `shape_flags`). Before this they had no
  planes. The shipped touchers do not read either.
- `drone::human_touchers` still returns the stand-in set, now named `box_touchers`.

[S] One part is not exact. A crossing point strictly inside the segment is taken as ON the plane
(s = 0). In f32 its recomputed s comes out ±1e-7, which would decide the one-sided test by
rounding, and the VU0's own rounding cannot be reproduced here.

## Measurement

`touch_mesh_census` counts the stand-in against the ported law over all 468 player-touchable
volumes:

- 338 volumes differ.
- Stand-in-only: 70109 samples.
  - containment: 5697 samples, 21 volumes
  - one-sided: 34294 samples, 132 volumes
  - the box's extra height: 30117 samples, 139 volumes
- Law-only: 18538 samples, 238 volumes.
- `game_touchers` matches the law at every sample.
- 449 of the 468 glists are raw, so the reference-point law decides only 19 volumes.

## The two plates the box was kept for

- **05 gatehouse, ch 0x5C (script quad, y 27.05).** The reference trace
  `tests/traces/05-truck.trace` [L] settles it. Bond stood at (100.09, 25.88, 108.01), inside the
  quad's footprint, from frame 33244 to 33456, and ch 92 stayed 0. He jumped at 33456 (pad 0x10)
  and ch 92 rose at frame 33464.
  - At frame 33463 his pos was 26.264. That puts A at 26.539, 0.511 under the quad, inside r 0.55.
  - At frame 33462 A was 0.558 under the quad, outside r.
  - So the game's contact is the segment top (pos + 0.275), seen one frame late in the trace.
  - The box fires this plate while Bond stands. The game does not.
- **011 #235 (ch 0x66).** This plate is two-sided: four triangles, two facing up and two facing
  down [D]. The vent fall through it fires under the law (the vent-route test passes either way).
  No standing samples reach it.

## Why it is not switched

With `human_touchers` returning `game_touchers` (the same build, before the s = 0 rule), these
binaries failed. There were 24 failing tests in all:

| Binary | Failed tests |
|---|---|
| fix_objects2_live | 3 |
| fix_truck_live | 2 |
| interactables_07000007_live | 1 |
| interactables_0700000a_live | 2 |
| interactables_0700000b_live | 2 |
| interactables_0700000d_live | 1 |
| interactables_07000011_live | 1 |
| interactables_0700004a_live | 1 |
| mayhew_handover_live | 2 |
| walk_0700000b_live | 1 |
| walkreal_07000001_live | 1 |
| walkreal_07000002_live | 3 |
| walkreal_07000005_live | 1 |
| walkreal_07000006_live | 3 |

walkreal 08 onward did not finish. All the other interactables binaries passed, as did both
interactables census binaries, tripwire_shot, walk_07000014, and walkreal 03, 04 and 07.

The causes read so far:

- **Teleport-and-stand pins that rely on the box's reach.**
  - The 05 gatehouse: the trace above shows the game needs the jump.
  - 07's roof music plate #443: an up-facing plate 0.2 over the segment top at the step stand.
  - 08's cable-car floor #236, which is a closed box with outward faces.
  - 4a's reveal plate.
- **Containment volumes.**
  - 06's LoadPoint #746 has three outward walls. The walk-in test reads the Touch channel after
    Bond is inside, when no wall is in reach.
  - 08 #236 is the closed box above.
- **Real walks that took other branches** on 01, 02 and 06. These may come from changed channel
  timing. They are not traced.

Each of these needs a real-path re-pin, or proof that the game's route differs, before the switch.

## Tests run

These pass on the final tree:

- nf-game lib `triggers::`, `touch` and `dist2tri` (35), with the new
  `game_touchers_take_the_game_refine_on_every_volume` and an extended
  `overhead_beam_misses_standing_bond`
- walkreal_07000005_live (17), mayhew_handover_live (3), fix_truck_live (2) and
  fix_objects2_live (6): unchanged behaviour on the script plates whose planes are now filled
- interactables_07000011_live (17) and interactables_census_live (21)
- touch_mesh_census (prints the numbers above)

The new test fails when `game_touchers` is the box set: the reference-point and containment
asserts. The existing pins stay green because the shipped set is unchanged.

## Not ported, and why

- **The nearest-solid cut-off** (`Collide_Update` 0x1ea398:216-229). A trigger node is reflected
  only while every nearer node in the distance sort is pass-through: raw, surface bit 0x40, the
  surface-0x80 `Rand_Rand(0x3f)` roll, or glass under +0x9A & 0x20.
  - The world node's distance is measured on the pre-push pose (the cel geometry node of each
    straddled cel is intersected before that cel's objects). The port's touchers see only the
    resolved pose, where every solid face sits at r or beyond.
  - Porting it needs the pre-push contact distance out of the player's collide pass.
  - The 0x80 roll would spend random numbers the port does not spend today.
- **Cel scoping of the object pick** (`Collide_Pick` 0x1eb320 walks the straddled cels' object
  lists plus the ForcedList). The port has no trigger-to-cel linkage to reproduce it.
- **The stage-1 sphere.** It never changed an answer in the census. `game_touchers` omits it.
- **The hanging arms** keep the stand-in set.
- **Physical push from non-raw trigger glists.** `Collide_Intersect` adds DeltaP to the real
  HITTEST, so the 19 non-raw volumes may push Bond in the game. This is not checked live.

**Live check that would settle the containment case:** on 0700000b, stand fully inside lift-stop
volume #245 after the crossing and read ch 0x75 for a few frames.
