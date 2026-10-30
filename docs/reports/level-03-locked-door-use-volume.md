# level-03-locked-door-use-volume — report

Branch `s/level-03-locked-door-use-volume` (master 8ce724ad). Task: the mask-4 trigger beside
07000003's locked door — the port's use-pick against it did not match the game's reach;
`interactables_07000003_live::locked_door_refuses_the_e` carried the named-blocker ignore from
level-03-interactables. Find the game's use-pick volume for that trigger, fix the port's stand-in,
un-ignore the test.

## Done

The port's use-pick stand-in (`Drones::player_use_triggers`, `crates/nf-game/src/drone.rs`) tested
the use point ±1.0 against the trigger's collision AABB — a documented superset that admitted the E
from the locked-door doorway, 2.9 units from plate #504's centre, firing ch 0x31 whose rising edge
cleared the lock. Replaced with the game's two-stage pick:

1. **The bounds-sphere pre-gate** [M `Collide_PickObj` 0x1eae80:219-228]:
   `dist²(use_point, obj+0x80) < (1.0 + obj+0x8C)²`. The port derives the sphere from the trigger's
   collision AABB (centre + half diagonal) — it encloses the mesh, so it never refuses a point the
   exact test accepts. (`Collide_Pick` 0x1eb320:97-105 copies the pick point to hittest+0x60 with
   the radius at +0x6c for the point modes — that is the sphere the gate tests.)
2. **The glist refine** [M `Collide_Intersect` 0x1ebaa8:186-190 dispatches mode 0x101 to
   `Intersect_PointGeom` 0x1e92d8]: the use point vs the trigger's own collision glist — the disc's
   block-0x2E triangles [D], whose winding carries the game's stored plane normal
   (`nf_formats::collision::triangles_with_material`'s alignment). A triangle registers when the
   signed plane distance sits in the two-sided window [−1.0, +1.0] [M 0x1e92d8:200 skips `X >
   radius`; 0x1e92d8:227 skips `X < fVar25` with fVar25 = −radius when the hittest flags carry bit
   8, 0x1e92d8:116-121 — the flags byte itself is the dropped fourth constant at the call site,
   [S] below] AND the point's projection onto the tri's plane lands on the triangle (see the [S]
   below). A mesh-less trigger is never hit [M 0x1ebaa8:89-92 — `obj+0xD8 == 0` returns no hit].

The pick radius 1.0 and the point = head + 1×look stay as they were [M `Player_Activate`
0x1a5170:100-108; `Player_GetHeadPos` 0x195aa0 = obj+0x34 + BL+0x908 − BL+0x90c — the port's
`bond.head()` = pos + EYE 0.7].

## The evidence chain

- The game's use pick builds ONE point (`Player_GetHeadPos` + `Mat_GetDir` of the slot viewer) and
  calls `Collide_SphereIntersect(point, 1.0, player_cel, player, bl+0x7e8, hits, 0,0,0)`
  [M 0x1a5170:106-108]; the hitlist dispatches per class byte, class 0x1d → `Trigger_Activate`
  [M 0x1a5170:126-137], and `Trigger_Activate` 0x1b2318's type-1/2 arms gate on `mask & 4` before
  any channel write [M 0x1b2318:23-49 — the port's `activate_request` already had these].
- The disc data: #504 (0xEA, p0=49, p1=4, t (11.80, 6.04, −35.63), rot +90°) shares object 308
  `oDroneTrigger` with the walk-over plates #414/#415 [D placements-all]. Object 308's collision is
  8 triangles: three horizontal sheets (y 5.64 / 6.04 / 6.44, all facing up) plus one vertical fin
  at z −35.63 (y 5.34-6.73, facing north), footprint x 10.09-13.51, z −36.33..−34.94
  [D `nf-extract coll-scan 07000003.bin 11.80 6.04 -35.63`; the glist's local sphere centre
  (0,0,0), radius 1.89]. Object 325 `oDroneTrigger2` (the 0xEB strip) carries a separate flat box.
- The sister's doorway stance (the test's teleport `p4 − (1.8, 0.1, 1.8)` settles to pos
  (12.881, 6.480, −37.202), head (12.881, 7.180, −37.202), yaw 2.2151 → the use-door press loop
  turns to 0.7854): the settled use point is (13.58, 7.18, −36.49) — 0.74 above the top sheet and
  0.07 / 0.16 outside its east/south edges. The projection misses every sheet, and the lower
  sheets' plane distances exceed 1.0 — the pick refuses, so ch 0x31 never rises and the locked door
  never moves [D audit: the game plays the locked-door hit sound until ch 0x31]. The old AABB
  stand-in fired the plate here, and so would a pure clamped-surface-distance reading of the refine
  (the top sheet's corner distance 0.76 is inside the 1.0 reach) — the measured refusal pins the
  projection-inside reading.
- The two-sided window is forced by the level exits: 07000015's #445 LOAD_LEVEL planes all face
  +z (z −24.60..−24.96, x −0.64..0.68, y 40.18..42.49 [D coll-scan]) and the walking player's use
  point sits up to 0.45 BEHIND them when the E loads 07000016 [D walk_07000015_live — green on the
  final tree]; a front-side-only window refuses the game's own exit. The first cut of the fix
  (front-side only) failed exactly that test and 0d's crane plate; the two-sided window + the
  projection-inside requirement reconciles both.
- The plate's own control: the game fires #504 from ~1.5 south aimed at it [D
  docs/reports/level-03-interactables.md's control probe] — the use point (11.80, ~6.8, −35.1),
  projection inside the footprint, plane distance ~0.4. The fixed port fires it there. The test's
  stance moved to the documented wedge spot (10.36, −34.12 — the level-03-interactables probe's own
  head_bump spot, where the direct last leg to the plate is blocked) with the horizontal look
  across the volume: the use point (10.36, ~7.0, −35.1), plane distance ~0.5.

## [S] unverified — two readings the decompile cannot settle

1. **The registration arm**: `vecutil_Dist2Tri_PointGeom` 0x1dfac8 computes the clamped closest
   point for the edge/vertex cases too (0x1dfac8:86-106), and `Intersect_PointGeom`'s caller
   compares that distance against the radius (0x1e92d8:229) — read alone, edge-clamped hits would
   register. The game's measured refusal on 03 (the locked-door audit) requires that only the
   plane-projection arm (0x1dfac8:77-85) registers a use-pick hit from outside the footprint. The
   port implements the projection-inside reading and marks it `// [S] unverified` at
   `point_reaches_tris`.
2. **The flags byte**: the use pick's fourth trailing argument is dropped by the decompile
   (`0,0,0` for the three shown); the two-sided window follows behaviourally from the 0x15 exit
   planes [D above].

No pass/fail assertion rests on either [S] alone: the sister test's assertions are the lock gate
[M door.rs, D p3=49] and ch 0x31's absence [D audit], both pinned independently of the readings.

## Tests

- `interactables_07000003_live`: **9 passed, 0 failed, 0 ignored** (`test result: ok. 9 passed;
  0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 96.31s` — release,
  --test-threads=1, NF_ISO on the slim ISO). `locked_door_refuses_the_e` is un-ignored; the
  E-plate test's stance change is documented at the site (the old stance's use point landed north
  of the volume — the game would refuse it too under the fixed pick).
- Two neighbours the fix touched: `walk_07000015_live` **2 passed, 0 failed** (the exit's
  behind-the-plane E is what forced the two-sided window) and `interactables_0700000d_live`
  **11 passed, 0 failed** (the crane touchplate's sliver sheets need the projection inside their
  footprint — the test's stand moved onto the plate, the hold-E is the real input).
- Every `interactables_*` / `interact_*` binary on the final tree: see the suite section.

## Suite

The fast runner (`tools/test/suite.py`) on the final tree did not complete inside the session
window: the first start (18:04) was killed for running the pre-fix binaries, and the restart
(18:17) had its cargo build invalidated mid-build by the branch switch to the next task (the
runner died with a transient `probe1b_nis211` compile error that does not reproduce on the final
tree — the file builds clean alone). What ran, quoted individually (release, --test-threads=1,
NF_ISO on the slim ISO):

- `interactables_07000003_live` 9 passed / 0 failed / 0 ignored (96.40 s, the final two-sided law);
- `walk_07000015_live` 2 / 0 / 0 (4.43 s); `interactables_0700000d_live` 11 / 0 / 0 (21.81 s);
- the interact files: 01 10/0/0, 04 10/0/0, 05 11/0/0, 06 9/0/0, 07 7/0/0, 09 16/0/0,
  0700000c 9/0/0; `interact_0700004a_live` timed out at 500 s (a pre-existing hang to
  investigate — not touched by this fix's binaries).

The next session should start the runner FIRST on this tree, verify `suite.out` grows, and report
the total line.

## Live checks for the main agent

1. The locked door #4 (14.68, 6.65, −35.40): stand at ~(12.88, floor, −37.20), aim at the door,
   press E. Expected (game): the door stays shut, the locked hit sound plays, ch 0x31 stays off.
   Watch `switch_channels[0x31]` with `tools/trace/fieldprobe.py`. Expected (fixed port): the
   same — `locked_door_refuses_the_e` now asserts it.
2. The refine-reading check that settles the [S]: with `tools/trace/gdbwatch.py` on
   `Intersect_PointGeom` 0x1e92d8 during check 1's press — does the game reach the per-triangle
   registration compare for plate #504's top sheet (the edge-clamp arm registering would fire ch
   0x31) or refuse at the projection stage (the port's reading)? The port predicts: no
   registration for #504 from the doorway stance.
3. Control for both: stand ~(11.80, floor, −34.1) aimed at plate #504, press E — ch 0x31 rises
   (the game fires the plate from ~1.5 south [D level-03-interactables' control probe]; the fixed
   port agrees).

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures for
the session (no usage display is exposed to the agent; nothing to copy).
