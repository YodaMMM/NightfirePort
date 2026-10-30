# WORKLOG

### level-4a-every-objective-4 (start, 05:37 UTC, 2026-10-30)

Task: level 4A (0700004A) — with the game's cel-scoped drone bullet flight [M 0x12e7b0:343 +
0x1eb320:125], `walkreal_07000008_live::walk_the_whole_mission_on_real_input` stops at the
fusebox shot (ch 137 never rises); the flat flight keeps it green. The -3 report (origin/
s/level-4a-every-objective-3, unmerged) narrowed it to the station's guards' rounds reaching the
player on the stair climbs under the scoped flight. Find exactly what changes for the walk
(which guard, which round, what it hits) and what the game does there [M/D; L if needed]; fix
the wrong port side (AI, walk route, or the flight) so the scoped flight lands AND the 08 walk
is green. FULL suite total line to quote. Continuing level-4a-every-objective-3.
