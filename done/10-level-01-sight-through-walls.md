# Guards see Bond through walls

Seen on 07000001 near (11.43, 6.47, -3.51) and reported as general. Read the sight test (the line of sight call, its flags, which cels and objects block it) and compare it with the port's guard sight on real geometry: a guard with a solid wall between it and Bond must not see him.

Reproduce on the real level first through real input, not a teleport; a report that repeats means earlier tests passed without reaching the real path. Compare with the decompile, fix, and test red on the old code. Branch from origin/master, push to `s/level-01-sight-through-walls`, report in docs/reports/level-01-sight-through-walls.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox5 at 2026-11-07T21:55Z

---
## Report (done, 2026-11-07T22:50Z)

# level-01-sight-through-walls — report

Branch `s/level-01-sight-through-walls` from `origin/master` (d09f8b2b3). Task: guards see Bond
through walls (seen on 07000001 near (11.43, 6.47, −3.51), reported as general). Read the game's
sight test (the LOS call, its flags, which cels and objects block it), compare with the port's
guard sight on real geometry, and make a solid wall between a guard and Bond refuse the sight.

## What the game's sight test is

The guard-vision ray is TWO tests, not one — the decompile is unambiguous at all three call
shapes [M]:

1. **The cel-reachability gate.** `DroneVision_LineOfSightToObject` 0x178390:16-18 and both
   branches of `DroneVision_CanSeeObjectFrom` 0x176f18 (SP :65-74, MP :91-100) run the ray only
   when `celFrom == celTo || AINetwork_TestRayCels(from, celFrom, to, celTo)`. `TestRayCels`
   0x15ab78 is a `Collide_StraddleCels` 0x1eac98 walk (the 0x201 ray path, the same candidate
   walk `Collide_Pick` 0x1eb320:313-329 rides) from the shooter's cel along the ray that must
   REACH the target's cel — the docs' earlier "segment against every boundary of the crossed AI
   cels" reading (docs/systems/ai-network-route.md) is corrected here: the decompile shows a
   straddle + membership test (`CelList_219[..] != param_4` scan, 0 if the walk exhausts), i.e.
   the walk is stopped by boundaries the portal graph cannot cross and the sight dies there.
   The shooter's cel is the runtime cel link `obj+0x20` (`DroneVision_CanSeeObjectFrom` :52/:55;
   `Drone_PositionVisible` 0x13a7a0:47 reads the same chain for the player viewer).
2. **The mask ray.** `Collide_LineOfSight` 0x1edf50 → `Collide_RayIntersect` 0x1ec690 with the
   caller's mask `0xa27` on HITTEST+0x98 (the drops in `Collide_Intersect` 0x1ebaa8:300-319:
   0x800 → surface byte 0x40, 0x1 → classes 0xd/0xe, 0x2 → glass 0x10, 0x20 → the 0x1000 nodes).
   The per-tick flow around it: `DroneVision_ProcessDroneSight` 0x176a28 (pass 1 visibility ≥
   0.7 → +0x230, pass 2 the round-robin `SeekOpponentLOS` 0x175cb8 whose per-drone entry clears
   +0x228 bit 1 and shifts it to 0x40 :20-26, the bone-cursor walk :49-86, `| 2` on success
   :104; pass 3 accumulates +0x270 on `(+0x170 != 0) && (+0x228 & 2)` :216-231 with the
   reaction gate in `HaveOpponentSight` 0x175ef0).

## What the port did and the fix

The port had only test (2)'s raw ray: `World::line_of_sight` → `segment_clear` (with the
`LOS_MASKS_ON=false` gate, world.rs:2622) — strict in the mask dimension (more blocking than
the game, never less) but with NO cel-reachability gate at all: any boundary whose triangles
the raw ray misses was sight-through. `Drone::seek_opponent_los` (drone.rs) documented the
TestRayCels pre-pass as "the port's world ray, as before [S]" — i.e. not implemented.

Fix (this branch):
- `World::sight_cell(p)` — the smallest scope-graph box uniquely containing the point, the
  port's stand-in for the game's `obj+0x20` cel link. `None` when ambiguous (the port has not
  ported `build_FindCel` 0x1c8cb0's raycast tie-break among overlapping cel boxes — the
  `build_scoped` header documents that gap), and every caller FAILS OPEN on `None`.
- `World::line_of_sight_cel(from, from_cell, to, mask)` — the game's gate: refuse unless the
  target sits in the from-cell (the `celFrom == celTo` arm) or the `debug_ray_cels` portal walk
  from `from_cell` reaches ANY cell whose box contains the target (the TestRayCels reach arm,
  shaped for the port's overlapping boxes). Then the raw ray as before.
- `Drone::seek_opponent_los` computes both bodies' cells per call and routes every ray
  (bone-cursor rays included, the team-3/MP bone-5 ray included) through the gate.

Behaviour on 07000001, measured: for EVERY nav-stand pair whose raw guard ray is clear
(2,347 pairs over 103 stands × two heights), the portal walk reaches a containing cell —
0 refusals (test `sight_cel_gate_01_live`). The gate therefore changes no sighting on 01's
walked geometry; it closes the mechanism the game uses to refuse sight across portal-less
boundaries (windows, one-way portals, graph seams), which the port's raw ray could only miss
when no collision triangle crosses it (skipped panes, mask-dropped material, scope gaps).

## Verification

- New unit test `world::tests::guard_sight_needs_the_cel_walk_to_reach_the_target` (world.rs):
  a synthetic two-cell graph — no portal → the gate refuses a raw-clear ray (RED on the old
  code, where the raw ray was the whole sight test); the doorway portal → pass; ambiguous eye
  cell → fail open. GREEN.
- New `crates/nf-view/tests/sight_cel_gate_01_live.rs`: on the real disc level, every
  raw-clear nav-stand pair keeps passing the gated call (no sighting changes on 01); the
  census numbers are asserted (pairs > 500).
- Full walk of the real level, real input: `walk_07000001_live` 3/3, `walkreal_07000001_live`
  8/8 (the escort walk passes end-to-end with the gate live).
- Guard/door binaries that exercise the touched code: `guard_reaction_live` 3/3,
  `find_opponent_live` 1/1, `npc_doors_live` 6/6, `fix_guard_misc_live` 6/6 (+1 ignored),
  `sight_cel_gate_01_live` 1/1, `ai_live` (nf-view), `vision_bit_census`.
- `nf-game --lib` 693/693 (+5 ignored, includes the new unit test); `nf-game --tests`
  1108/1108; `nf-formats` + `nf-audio` 138 passed / 3 failed — the failures are exactly the
  3 `disc_golden` tests allowed on the slim ISO (bigf_header_fields, bigf_tables,
  directory_tree_matches_the_golden_manifest). Not run in this window: the remaining nf-view
  walkreal set for other levels (disk-full cleanup deleted their cached binaries and the
  rebuild did not fit the window; no shared helper was touched — the walker is untouched).

## What is left (open)

1. **The reported symptom itself did not reproduce in the port.** Three probes (this session,
   scratch): a teleport of Bond to (11.43, 6.47, −3.51) with the three ch-0xB guards live —
   all three lost sight within 120 ticks; a real-input walk of the spawn → reported-spot →
   meet-plate route with per-tick ray cross-checks — every sighting had clear geometry (open
   plaza) or was dropped; a geometry sweep of all nav-stand pairs — no raw-clear pair the
   portal walk refuses. Either the reporter's spot needs a specific guard facing/motion state,
   or the through-wall sight they saw was the missing cel gate firing on a boundary my probes
   did not sample (guards move; my sweeps used spawn positions and nav nodes). **Live checks
   for the main agent** (PCSX2, `tools/trace/gdbwatch.py` on `drone+0x228`/`+0x270` of the
   guard nearest the reported spot, plus the port-side render at the same pose):
   - Watch bit 1 (0x2) of the nearest guard's `+0x228` while Bond stands at (11.43, 6.47,
     −3.51) behind the terrace wall at each of a guard's patrol posts: does the GAME's flag
     ever rise? Expected: no, from any post with the wall between.
   - The same pose in the port's render (`--bond 11.43,6.47,-3.51,<guard yaw,pitch>`): does
     the port's guard alert/track? Expected after this branch: no; on master it may have.
   - If the game's flag DOES rise somewhere: record the guard's cel (obj+0x20) and Bond's —
     that pair is the real portal-gap repro this branch's gate models.
2. **The drop-mask half of the game's ray is still parked**: `LOS_MASKS_ON=false` keeps
   `line_of_sight` strict (the port refuses sight through 0x40-material/glass that the game's
   0xa27 mask drops — guards blind where the game sees, the opposite of this task's symptom).
   The masked body exists (`line_of_sight_masked`); the per-caller flip stays the parked item
   its header documents.
3. The same cel gate applies to the other guard-vision ray callers
   (`NDrone2_CheckUnsurrender`'s LOS at drone.rs:10512, the armed-peer head-to-head rays,
   `CanSeePosition`/`LineOfSightToObject` users); they still call the raw ray. Follow-up with
   the same `sight_cell`+`line_of_sight_cel` shape.
4. `build_FindCel` 0x1c8cb0's raycast tie-break (the authoritative per-body cel among
   overlapping boxes) remains unported — with it the fail-open ambiguity above disappears.

## Hand-off notes

- No draw work was done; nothing visual changed (the gate only refuses sight states).
- The scratch diagnostics (`zdiag_sight01_*`) were run from this session's working tree and
  are NOT committed (probe/scratch outputs — rule 3).
- The 3 `disc_golden` failures are the allowed slim-ISO set (pre-existing on master).

## Usage

`not visible to me` — this session's driver shows no token, context-window or cost figures
anywhere I can read (no environment variable, no banner, no tool output carries them).
