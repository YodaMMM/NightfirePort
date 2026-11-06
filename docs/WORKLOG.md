# WORKLOG

## level-14-landing-float
Started 2026-11-06 09:05 UTC. Branch s/level-14-landing-float (from origin/master 62c8281d6). Task:
`walkreal_07000014_live walk_the_infiltration_to_the_delta_exit_real_damage_00agent` wedges at tick
26340 — Bond floats 0.41 over the floor at (33.84, 0.82, 30.95) on the 1.72 landing resupply leg.
Treat as a possible game-side bug first: put Bond at that spot on master, settle, print grounded /
floor ray / capsule contacts. If he hovers without the walk, fix the collision and cite the
decompile; if only the walk causes it, say which leg and fix the route. Remove the ignore when green
and quote the whole binary. Earlier task (10-level-14-landing-hover) closed with no code; its report
lives on s/level-14-landing-hover.
  Results (2026-11-06 10:50 UTC): the float is a GAME-SIDE bug and the pocket fix landed.
  probe14_pocket_settle (new, assertion-free): teleport to the recorded spot (33.84, 0.82,
  30.95), settle quiet — Bond parks at (33.9712, 0.6500, 30.9501) grounded=false for 240
  ticks, the band failing at dy 0.65 > cc 0.55 + 0.1, held by the port's [S] spine-fallback
  push (0, +0.0027, 0) every tick off the flat y≈0 floor (n=(0,1,0), cent (34.320,0.000,
  32.067), d 0.5473 = the r-equilibrium) — the game's tail is DEAD there [M Intersect_CylGeom
  0x109da0:431-447: `0 < dot(q−HITTEST+0x50, n)` AND `|q−ref| < radius`, ref = the lifetime
  DeltaP accumulator [M 0x1ebaa8:353-373; L ee_start boots (0,0,0)] — q below ref fails the
  dot gate; there is no fallback in the decompile]. Fix: the failed-ref UP-FACING hold skip
  (world.rs capsule_tri_contact walk_flat_hold, threaded capsule_push_scoped_hold/capsule_
  push_hold on the walk pass only) — skip the fallback displacement when the radial arm's
  gates fail, n[1] > 0.7, q below the spine bottom, mag < 0.02 (the quiet-dither regime; the
  unscopted version broke the stairwell pit-band jump at tick 1380 — the big-mag lift is
  load-bearing). Pocket after: grounds, hist 0xf, rest (33.9712, 0.6335, 30.9501) vel 0.
  Cherry-picked the two prior lever branches (s/level-14-ignored-walks ae4f15cca: the d3
  trench stand + the drop loot pass + the wheel draw; s/level-14-landing-hover d96f7863b:
  descent_stationary_fights + escalating_cooldowns + the d2 head-hunt; conflict resolved to
  carry both sets). The d3 walk now clears the stairwell pit band and dies at tick 6437 —
  the infiltration-grind BUDGET death the landing-hover window measured (its tick 6786):
  31 hits for 100.0, 16 fights, the far guards #9/#10/#3/#11 (hp 10 each, d 1.1-4.0) take
  65.0+15.0+10.0+10.0 through stands that land ZERO rounds (killed false — the rounds die
  on the corridor corners; eye_ray_stands did not change it). The ignore stands; the
  un-ignore waits on the grind's fight answer (the 07000013 repositioning law or the
  answer-only-after-first-damage gate), then the whole-binary quote + the walkreal battery.
  Report: docs/reports/level-14-landing-float.md.
