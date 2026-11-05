# level-12-engage-range — do alarmed guards stop too far out?

Branch `s/level-12-engage-range`, cut from origin/master f68a72216, 2026-11-05 ~11:08-12:15 UTC.
The predecessor `s/level-12-invisible-wall-spots` found that on 07000012 after the real alarm
drone #6 holds at the corridor mouth with clear sight at 9-10 u and #7 stays in cover, so nobody
reaches the cafeteria. This task: compare `engage_range` for skins 0x5000008 and 0x5000006 with
the game's Drone+0x1A0 [M] and the combat_state advance rule; fix the port if it holds where the
game advances; test the no-sight advance against the corner at (22.47,36.64) against the game's
arrival radius; run the 12 walks.

## The comparison: the port does NOT hold where the game advances

**engage_range values match.** The game initializes the combat ranges in `NDrone2_DefaultInit`
(0x14b300) from the placement's sight class (`switch(piVar19[0x1b])`, the LAB_0014b8a4 region):

| class | Drone+0xE8 sight | Drone+0xF0 engage | Drone+0xFC combat |
| --- | --- | --- | --- |
| default | 24.0 | 12.0 + FRand(4.0) | engage + 1.0 + FRand(4.0) |
| 1 | 12.0 | 12.0 + FRand(3.0) | engage + 1.0 + FRand(3.0) |
| 2 | 16.0 | 12.0 + FRand(3.0) | engage + 1.0 + FRand(3.0) |
| 3 | 40.0 | 25.0 + FRand(6.0) | 100.0 + FRand(6.0) |
| 4 | 1000.0 | 75.0 + FRand(8.0) | 1000.0 |
| 5 | 0.0 | 12.0 + FRand(4.0) | engage + 1.0 + FRand(4.0) |

[M NDrone2_DefaultInit 0x14b300 — `switch(piVar19[0x1b])` cases default/1/2/3/4/5; the floats
0x40800000=4.0, 0x40400000=3.0, 0x40c00000=6.0, 0x41000000=8.0, 0x41c00000=24.0, 0x41800000=16.0,
0x42200000=40.0, 0x447a0000=1000.0]. The only per-level overrides in that function are for
GameState 0x7000011 and 0x7000014 — **none for 0x7000012**. Both skins on this level carry sight
class 2: placement #5 (skin 0x5000006) and #6 (skin 0x5000008) both have param 14 = 0x2
[D docs/levels/07000012-audit.md:21-22], so both drones roll engage_range 12.0-15.0. The port's
table (drone.rs:1985-1988, stored at :2017) is value-for-value identical, including case 1's
sight=12.0 and case 5's zeroed +0xE8/+0xE4.

**The advance rule matches.** `DroneFunc_CombatState` (0x148338):

- COMBAT (case 0): `+0xF0 <= +0x1A0` → COMBAT_OUT_OF_RANGE 0x5a [M 0x148338 case 0].
- COMBAT_OUT_OF_RANGE (case 2): cover first (0x8c), then `+0x1A0 < +0xF0` → back to COMBAT 0x58,
  else move on the `NDrone2_MoveToObject` result [M case 2].
- AIM_STAND_FIRE (case 0x10): empty magazine (+0xBBC < 1) → 0x69 first; then advance iff
  `routed && +0xF0 <= +0x1A0` where `routed` = the `MoveToObject` result computed at the head of
  the function is < 4 or == 10 [M case 0x10].
- AIM_CROUCH_FIRE (case 0x18): the same `routed && +0xF0 <= +0x1A0` exit [M case 0x18].
- The router calls `NDrone2_MoveToObject` every tick with radius `min(engage, 6.0)` — the C shows
  `(int)fVar10 * (uint)(fVar10 < 6.0) | (uint)(fVar10 >= 6.0) * 0x40c00000` (6.0 = 0x40c00000)
  [M 0x148338:35-37].
- Drone+0x1A0 is the opponent distance: `Drone_GetOpponentInfo` (0x138f68) copies the opponent
  object's +0x30 position into Drone+0x180 and stores `Vec_Magnitude(opp_pos - own_obj+0x30)` —
  a full 3-D distance between object positions — into +0x1A0 [M 0x138f68].

The port implements the same arms at drone.rs:10723-10742 (COMBAT/OUT_OF_RANGE), :10759
(AIM_STAND_FIRE, with the clip→reload gate), :10787 (AIM_CROUCH_FIRE), with `out_of_range =
self.engage_range <= self.dist` at :10725 and `self.dist` the 3-D obj-to-obj distance at
:11361. **So at dist 9-10 u with clear sight and engage 12-15, both the game and the port hold
in the fire states; both advance only when dist >= engage.** The port does not hold where the
game advances — the corridor-mouth hold is what the game's own rule produces. The predecessor's
"nobody reaches the cafeteria" cause must lie elsewhere (the sight/LOS gain point, or the goal
machinery below), not in engage_range.

## The found difference: goal-move arrival radius — fix attempted, REVERTED

The no-sight advance is goal machinery, not combat machinery. The game's `MoveToGoalPosition`
(0x1517d8) arrives when the route distance drops under **route+0x8C = Drone+0x8EC**
[M 0x1517d8:49-52], and that field:

- is seeded 2.0 by `DefaultInit` [M 0x14b300:579 — both +0x8EC and +0x8F0 = 0x40000000];
- is rewritten by `SetupRouteToPosition` from the goal's radius argument on every position route
  build (`*(float *)(puVar9 + 0x46) = param_1`, 0 → 2.0) [M 0x158b10];
- is NOT rewritten by the combat route build `SetupRouteToObject` [M 0x158948 — no route+0x8C
  store] — `MoveToObject` (0x151618:65-67) reads whatever the last position route left.

Per-caller radii census [M]: the alarm-point goal = **the AIPoint's own +0x14 accuracy**
(`FindAlarmPoint` 0x153008:20-23 fresh / :45-48 reuse → `SetupGoalPosition(point+0x14, ...)`),
`FindRunToPoint` the same [M 0x153168:39-43], sound/drone alert routes 2.0 [M 0x154aa8:11-13 /
0x154b20:10-12], `FindCover` 0.5/1.0 by cover corner class [M 0x152548:174-181], ambush cover
0.5 [M 0x152958:207-208], safety-from-explosive 1.0 [M 0x152250:126-127, 163-164], mission fresh
path 1.0 / retry 2.0 [M 0x153a30:141-142, 168-169], the closer-position probe 3.0
[M 0x155e10:30], interrogate assist 10.0 [M 0x15add0 + its callers].

The port arrives at a hardcoded 2.0 in `move_to_goal` (drone.rs:5852) and `move_to_object`
(drone.rs:10858). The AIPoint accuracy is already parsed (`AiPoint.accuracy`, aipoint.rs:26-30)
but unused by the goal paths. For goals whose game radius is 2.0 (the sound/drone alert goals on
this level's alarm response) the port matches; for AI-point goals with accuracy ≠ 2.0, mission
1.0 and interrogate 10.0 it does not.

**The fix regressed and is not in the tree.** I implemented two forms:

1. goal-path radii only (`goal_arrival` per goal setter, `move_to_goal` arrives under it);
2. form 1 + the persistent route+0x8EC carry into `MoveToObject`'s arrival check.

Both turned `walkreal_07000001_live`'s two real-damage walks red — Bond dies (the
`[real-damage]` panic at walker.rs:279; 6/8 passed vs 8/8 on baseline). Baseline proof: with the
patch stashed the same test passes in 4.6 s; with it applied it fails. Mechanism: the true radii
let guards arrive closer to their goals (mission 1.0, alarm-point accuracies < 2.0) than the
walks' real-damage balance, which was tuned on the 2.0 stand-in — exactly the cross-level
blast radius the merge rules warn about. The patch is reverted; the next session needs the
per-level walk review (all 26 walkreal binaries) alongside the activation, plus the accuracy
values census per level (`AiPoint.accuracy` per 0xE9 placement) to predict which levels move.

## The corner question (22.47,36.64)

The game's no-sight arrival radius is the goal's radius, not a global constant. For the
07000012 alarm response the sound/drone-alert goals carry 2.0 [M 0x154aa8/0x154b20], so the
port's 2.0 does **not** stop short of the game's arrival radius for those goals — both stop when
the route distance drops under 2.0. Divergence exists only for goals with radius ≠ 2.0 (see the
census above). Whether the real game's guards, advancing no-sight past that corner, stop at the
2.0 rule or at a non-2.0 goal radius depends on which goal the game's alarm chain set for them —
that is what the live checks below pin.

## Verification on the final tree (master behavior, patch reverted)

- The 12 level walks `walkreal_0700000{01,02,03,04,05,06,07,08,09,0a,0b,0c}_live`:
  8+3+6+10+14+6+5+6+8+13+7+3 = **89 passed, 0 failed, 5 ignored**.
- This level: `walkreal_07000012_live` 3/0, `corridor_guards_07000012_live` 1/0,
  `enemies_07000012_live` 3/0.
- `cargo test -p nf-game` full suite: **1077 passed, 0 failed, 8 ignored**.
- `cargo test -p nf-formats -p nf-audio`: green except the 3 expected `disc_golden` slim-ISO
  failures (bigf_header_fields, bigf_tables, directory_tree).
- nf-view's remaining serial sweep did not fit the window (test-less change on the final tree;
  the walk binaries above were run individually).

## What's left for the next session

1. Decide the arrival-radius activation with the cross-level walk review (the patch recipe is
   above; the regression evidence is the 001 pair).
2. Census `AiPoint.accuracy` (+0x14) per level from the placements — it decides which levels'
   alarm/run-to goals move when activated.
3. The corridor-mouth hold itself (drone #6 at 9-10 u with clear sight) is game-ruled behavior
   per the comparison; if the user's playtest disagrees, the next suspects are the sight/LOS
   gain point at the mouth (BLINK walls #6/#905) and the goal the game's alarm chain sets —
   both observable with the live checks below.

## Live checks for the main agent

1. After the real alarm on 07000012, watch drone #6 (plc 12): log its `Drone+0x1A0` (distance to
   Bond), `Drone+0xF0` (engage range), and state (+0x10C) every 30 frames for 10 s. Expected if
   the comparison holds: +0xF0 ∈ [12,15]; while +0x1A0 < +0xF0 with sight, states stay in the
   0x68/0x70/0x74 band; 0x5a only when +0x1A0 >= +0xF0.
2. The same run: when a guard is in GoToGoal (0x63) advancing no-sight toward the corridor/
   cafeteria, log `Drone+0x8EC` (the route arrival radius) and the guard's stop position.
   Expected: the stop satisfies route-distance < +0x8EC; +0x8EC reveals the goal radius the
   game actually set (2.0 for the alert routes, the point's accuracy for FindAlarmPoint).
3. The corner at (22.47,36.64): a guard advancing no-sight past it — log where it stops relative
   to the corner and to its goal, plus `Drone+0x8EC`. Expected: the stop distance from the goal
   is under the game's arrival radius (i.e., it does NOT stop short of the radius).
4. `tools/trace/fieldprobe.py`: poke drone #6's `+0xF0` to 6.0 during the corridor hold.
   Expected if the rule is as ported: the guard starts advancing (0x5a) into the cafeteria
   (6.0 <= dist). This pins engage_range as the operative threshold in the real game.

## Usage

not visible to me
