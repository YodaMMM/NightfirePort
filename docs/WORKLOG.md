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
