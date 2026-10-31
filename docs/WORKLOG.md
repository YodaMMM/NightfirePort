# WORKLOG

## level-0b-parachute-step-up (branch s/level-0b-parachute-step-up)

- start (master 3afcdce1). Task: `walkreal_0700000b_live.rs` falls back to a stance sweep for the
  parachute pickup #280 (2.53, 58.45, 42.34 [D]) — the helicopter body's volume blocks the walk
  2.4 m short (cel 189); the pickup floor is 57.92 vs the deck 57.03 at z 42-44. Find how the
  game lets Bond reach it [D the copter's collision/glist and the pickup's touch volume; M
  Pickup_Handler 0x18e650, the player step-up in Player_Collision 0x194df0], fix any port gap
  with evidence, and replace the stance sweep with a walked take. Run walkreal_0700000b_live and
  any *pickup* binary; quote the totals.
- outcome (2026-10-31): the stance sweep is GONE — the take is WALKED. The blocker was never the
  helicopter: the copter flies its route away from the pad (state 1, (66, 57, 64) at +3600 ticks,
  probe on this tree) and the wedge at (3.61, 58.07, 41.34) ("cel 189") is the disc's own static
  geometry — the pickup's low box (top ≈57.92) + the housing's south wall east of x ≈ 2.8. The
  game's touch law [M Pickup_Update 0x18e148:202-220: dist(root) < radius+1.1 (0.69+1.1 [D model
  bounds]) AND Collide_LineOfSight(sphere centre → pickup, 0x721; one-sided rays [M 0x10a8c8:168])]
  passes from the deck slot x 2.0-2.5, z 41.0-41.5 where the housing's south face is open above the
  box; the old walk target (z +1.2 north) ground Bond east along the box into the wall. The test now
  walks the slot (arrived +77 ticks at (2.20, 58.07, 40.34), both walks) and hard-asserts ch 0x3F —
  no teleports. No nf-game source changed (the port's touch model already matched the decompile).
  walkreal_0700000b 3/3 (warm 15296 ticks, fresh 21056, both `stuck 0`).
- finish (2026-10-31): merged origin/master 20624c8c clean; the final tree's touched binaries all
  green (walkreal_0700000b 3/3, pickup_live 1/1, pickups08_live 5/5, fix_pickups_remainder 1/1,
  interactables_0700000b 9/9, walk_0700000b 4/4, models_sweep_0700000b 2/2, walkreal_07000011_route
  1/1). Suite (resumable foreground protocol — the sandbox kills every background job between tool
  calls): 279/976 binaries done, 812 passed, 6 failed (3 disc_golden allowed on the slim ISO, 2 the
  GPU-less 0c pylon pair, 1 campaign_flow_live 240 s cap), 53 ignored, 0 CRASH. Report:
  docs/reports/level-0b-parachute-step-up.md.
