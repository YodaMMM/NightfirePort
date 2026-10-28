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
