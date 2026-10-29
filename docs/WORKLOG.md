# Worklog

## level-0c-door2turbine (branch s/level-0c-door2turbine, from origin/master c0d3e92e)

Sandbox opened 23:57 UTC, expires ~01:57 UTC (stop new work 01:37). Scope: fix Chain Reaction
Part 1's blockers on the real walk, in order, each from the game's behaviour [M/D], replacing
each stand-in in walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff
with the real-input leg: (1) door #281 door2turbine (40.0, 12.6, -210.4) state 0x2 on the real E
path [M Player_Activate 0x1a5170 case 0x1C]; (2) plate #274 inside the north display case's glass
box; (3) display ground -> warehouse/yard descent (ch-84 warehouse vent #180/#184?); (4) the
ring's west/south gaps; (5) doorway-ledge -> yard descent. Real input only. Full suite + total
line before the final push.

Progress (00:30 UTC): the door2turbine census — the door's E gates are ALL open [M Door_Activate
0x135be0: placement #281 parses kind 0, ch_open 5, ch_lock 0, lock_flag 0, approach 0, state 0x2;
nothing writes ch 5 anywhere on the level (trigger/SS sweep empty [D])] — the E press is the door's
ONLY opener. The disc collision census [D]: the door's 78-tri glist is the leaf slab only (x
38.49..40.02, y 11.15..14.00, z −210.56..−210.36, sphere (40.01,12.57,−209.69) r 1.62); the SE
walkway's own floor (y 12.02) ends in a wall at z −213.51; a 10.17 ledge patch + a 9.11 step sit
east (x 40.37..41.54) with the shaft void below (6.17); the corner (11.06..11.14, x 36..40.5, z
−210..−208) and the band ((44,−209) 11.17 / (44,−208) 10.34) are in the port. The prior session's
"state holds 0x2" was the ROUTE, not the door code: the walker wedged south-west of the leaf, out
of the sphere's reach (use_point dist 3.1+ vs reach 2.62 [M Collide_PickObj 0x1eae80's 0x101 arm]).
The E stance from the 10.17 ledge patch computes to dist ≈ 2.51 < 2.62 — the real-input leg is
being routed; the walkreal keeps its green shape (2 passed) with the walkway-end wall pinned.
