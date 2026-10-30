# WORKLOG

## level-0a-corner-walk-speed (branch s/level-0a-corner-walk-speed)

Start: 2026-10-30 10:28 UTC, branch from origin/master 06b95819.

Task: un-ignore `walkreal_0700000a_live::the_corner_crosses_like_the_game` — the 0a corner only
crosses with the below-B kill (NF_WALK_KILL=1) in 870 ticks vs the game's ~60 frames
[L 2026-10-26], riding spine-direction pushes. Find what the game does at that corner
[M Collide_Intersect 0x1ebaa8 / Intersect_CylGeom 0x109da0 / the player push-out; D the corner's
geometry], fix the port side so the corner crosses at walk speed without the kill switch, and
un-ignore the test. Every test in the file green, plus the other walkreal binaries that can run.
