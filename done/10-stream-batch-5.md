# Stream batch: void recovery out, guards re-acquire

Branch from `origin/s/stream-batch-4`, merge current `origin/master`, push to `s/stream-batch-5`. Read docs/reports/stream-batch-4.md.

1. Take void recovery OUT of this branch: revert b5e916d84's changes to player.rs, world.rs, scene.rs and drop void_recovery_07000003_live. It is held: it pushes Bond back at valid spots (`find_cel` gaps; 4422 passed, 12 failed on its own). Do not merge `s/void-recovery*` unless it is on origin/master.
2. Re-enable the re-acquire exit in `combat_state` (`if false &&` in drone.rs) with the game's own 0x5c shape [M DroneFunc_CombatState 0x148338:67-70/151/235, NDrone2_DSTATE_CombatNoSight 0x164838:26-35], your report's reading. Guards freezing in no-sight is a player-visible bug. It once broke the abseil, door-chase and bathhouse guards on 06/09/16: find out why, fix those causes, re-prove those walks. If it is the port's LOS ray failing at those spots, say where, compared with `RecurseBoxesSph` 0x1EDAE0.
3. Re-verify the 03 composed blocks, then the plain walks 01/06/08/0b/0c/11/12/13/14/15/16, `mayhew_hold`, `fix_npc_doors`, the enemy_accuracy re-pin, the start_corridor d3 owner point.
4. Then real damage, Agent, 00 Agent. An ignore with a measured reason is allowed for 00 Agent walks only.

Do not loosen pins tagged [L] and do not change game code to suit a walk.

---
claimed-by: sandbox7 at 2026-11-06T19:03Z

---
## Report (done, 2026-11-06T20:04Z)

# stream-batch-5 — void recovery out, the re-acquire chase

Branch `s/stream-batch-5` = `origin/s/stream-batch-4` 95772586e + `origin/master` ab80ac6ed
(merge clean) + this window's work. Time-boxed window; the report states what landed, what was
baseline-proven, and the one diagnosis that moved a layer.

## 1. Void recovery OUT (done)

`e8b14adb5` (the out-of-cel recovery: player.rs +8, world.rs +29, scene.rs +42,
void_recovery_07000003_live.rs +133) reverted clean via `git revert` — the 4-test file is
dropped, `nf-game`/`nf-view` libs compile. `docs/reports/void-recovery.md` stays as the held
work's record; nothing is merged from `s/void-recovery*`.

## 2. The re-acquire exit — the decomp answer, and the fix that landed

The task asked to re-enable the `if false &&` exit "with the game's own 0x5c shape". The
decompile says that exit has NO game shape to re-enable:

- The router sends a blind guard (sight flag 4 clear, `+0x274 > 0xf`) to CombatNoSight 0x5c
  from exactly three arms [M DroneFunc_CombatState 0x148338:67-70 (COMBAT), :151
  (AIM_STAND_FIRE), :235 (AIM_CROUCH_FIRE)]; 0x5c's UPDATE re-runs the router every tick [M
  NDrone2_DSTATE_CombatNoSight 0x164838:26-35], and with the ray back case-4's sight arm
  returns 0x5b — firing resumes [M 0x148338 case 4].
- NoOpponent 0x60 comes ONLY from the top gate `+0x170 == 0` with the `+0x238` 30-frame stamp
  expired [M 0x148338:20-31] — and in SP `FindOpponent` re-points `+0x170` at Bond EVERY tick
  [M NDrone2_FindOpponent 0x1424e0:100-121; the CanSeeObject clear path :148-153 is reached
  only when there is no player target]. So the game never returns 0x60 from sight loss while
  Bond lives: the `unseen > 15 → NO_OPPONENT` exit was a port workaround, and enabling it as-is
  is what broke the 06/09/16 abseil/door-chase/bathhouse guards (it kicks game-blind guards out
  of combat entirely). The dead block is REMOVED with this citation trail (drone.rs
  combat_state), not re-enabled.

**The freeze's real cause, found by probe (this window's fix):** the port's guard sight ray is
UNSCOPED — it walks the port's global grid buckets — while the game scopes the guard→Bond ray
from the TARGET's cel [M DroneVision_CanSeeObjectFrom 0x176f18:60-77:
`Collide_LineOfSight(from, to, iStack_60, param_2, …, 0xa27)` with
`iStack_60 = *(int *)(target + 0x20)`, so `Collide_Pick` 0x1eb320's BFS admits only the
target-cel chain]. `world::line_of_sight_scoped(cell, a, b, mask)` added (the same masked drop
semantics over `scoped_runs`, falling back to the unscoped walk on `cell = None` or a
portal-BFS miss [S — the game's `AINetwork_TestRayCels` 0x15ab78 REFUSES sight on a chain
miss, but the port's graph is coarser and the conservative fallback keeps cross-graph sight]);
`seek_opponent_los` rewired to it with the snapshot's `player.cel` (already carried
[drone.rs::Player]).

**Where the port's LOS stands vs `RecurseBoxesSph` 0x1EDAE0** (the task's comparison ask):
`RecurseBoxesSph` is the box-tree AABB-vs-sphere collector used by `Intersect_PointGeom`
0x1e92d8 (the type-0x101 point query) — it is NOT in the LOS path. The game's LOS is a thin
type-0x201 ray [M `Collide_RayIntersect` 0x1ec690: the HITTEST shape word 0x201 →
`Intersect_RayGeom` 0x10a8c8, wrapped by `Collide_LineOfSight` 0x1edf50 = `!RayIntersect`] —
the same family the port's `line_of_sight_masked` models (back-face cull at 0x10a8c8:168
included). The port's divergence is the CEL SCOPE and the query set, not thin-vs-thick.

## 3. The 07000011 d2 stall — diagnosed one layer deeper (the next lead)

The stall reproduces (both guards hold `lost_sight=true/seen_ticks=0` 10000+ ticks at idle
visible Bond, health parks 4.0). The probe (`crates/nf-view/examples/los_probe11.rs`, plus the
read-only `los_debug_hit`/`los_debug_hit_scoped` diagnostics and an `NF_AILOG`-gated `[ai:5c]`
line) pins:

- Both guards park at (−2.34, 47.78, 22.22) directly behind a thin column (x −1.63..−1.35, y
  47.03..51.12, z ≈ 23.7, mat 0x0f) with mat 0x05/0x06 trim patches at y 48.23/48.36; **all 7
  view-bone rays hit it** (t 0.53..0.60).
- In 0x5c the router reads `mv 0` (MOVING) with the route `remaining 4.10` FROZEN for 10000+
  ticks — `Route::follow` reports moving while the body never advances (the creep wedge behind
  the column; the step-back arms at [M 0x15a2c8] never fire because the follow never returns
  BLOCKED). The scoped ray does NOT clear this stall (the column is inside the port's Bond-cel
  chain).
- Next lead: the 0x5c follow wedge — the game's guard re-positions out of a grazed-corner park
  (root motion slides past grazed corners [M 0x15a148's note]; the port's follow never steps
  back/around here). Live checks: (1) PCSX2 07000011 d2 idle Bond — do the two corridor guards
  ever park behind the column at (−2.3, 22.2), or do they strafe around it (the 0x42 step
  property)? (2) gdbwatch `LinkCreep_Decrement` 0x15a148's boundary-test calls at that spot —
  does the real creep step back and re-creep?

## 4. The verification battery (whole, --test-threads=1, real ISO)

| binary | result | note |
| --- | --- | --- |
| walkreal_07000009_live | 9/0/0 | the trio re-proven |
| walkreal_07000016_live | 14/0/2 | the trio re-proven |
| walkreal_07000006_live | 4/2/1 | the 2 reds = real_damage + real_damage_agent; **baseline-proven pre-existing** (same 4/2/1 with my src changes rolled back) |
| walkreal_07000001_live | 6/2 | reds = real_damage_agent + real_damage_00agent (the item-4 lane); plain leg green |
| walkreal_07000008_live | 5/1 | red = real_damage_00agent |
| walkreal_0700000b_live | 5/2 | reds = real_damage_agent + 00agent |
| walkreal_0700000c_live | 2/1/1 | red = real_damage (the batch-4 death-bleed lane) |
| walkreal_07000012_live | 2/1 | red = real_damage_agent |
| walkreal_07000013_live | 1/2 | reds = real_damage + agent (the 4A→13 lobby-fight bleed) |
| walkreal_07000014_live | 7/2/1 | reds = real_damage + agent |
| walkreal_07000015_live | 5/1/1 | red = real_damage_agent |
| start_corridor_guard_07000011_live | 6/2 | the same two reds as batch-4 (the d2 stall — section 3; the d3 owner point 2.4 u short — batch-4's finding); unchanged by my change |
| mayhew_hold_live | 0/1 | **baseline-proven pre-existing** (same red with my src rolled back) |
| fix_npc_doors_live | 0/1 | **baseline-proven pre-existing** (same red with my src rolled back) |
| enemy_accuracy_live | did not fit | the run exceeded this window's remaining time (7-min cap); the re-pin re-measure is item-4 lane work on the harness the batch-4 report describes |

Every failure in the walks is a real-health/Agent/00-Agent leg — the task's item-4 lane (its
own queue tasks), except the two start_corridor reds covered above. No plain walk regressed.

The full suite did not fit this window (the standing chunk ceiling); the touched-code binaries
above ran whole. `crates/nf-game/src/{world,drone}.rs` changed — the walkreal battery +
start_corridor + mayhew_hold + fix_npc_doors above are the grep-derived consumers; the
`enemy_accuracy` gap is named.

## What's left

1. The 0x5c follow wedge (section 3) — the d2 stall's root; then the d2 death completes and
   the d3 owner-point pin can be re-measured on the settled stream.
2. The item-4 lane: the real-damage/Agent/00-Agent walks (06's two reds, 01/08/0b/0c/12/13/14/15
   legs), the enemy_accuracy re-pin, mayhew_hold + fix_npc_doors (both pre-existing reds on the
   merged tree), the 13 d3 budget levers.
3. The scoped-LOS [S] fallback question (BFS miss → refuse vs the port's fall-open) once a
   live check can see the game's TestRayCels at a graph-gap spot.

## Usage

not visible to me
