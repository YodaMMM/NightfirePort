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

### level-4a-every-objective-4 (finish, 06:32 UTC, 2026-10-30)

The scoped flight carried onto master (the one-hunk cherry) and the 08 walk green WITH it:
the mechanism measured (the flat/scoped A/B — the wedges identical on both trees, the delta is
the fusebox-approach leg; the station guards' scoped rounds reach Bond — 228 impacts by the
lower-landing wedge — and each hit's pain-grunt roll [M 0x1902c8:111] draws the shared RNG the
AI draws, so the mitigation sidesteps wobble and the stair edge drops Bond into the pit; no
breakable dies, no door flips, no blast kick — the walk's fragility, not a port AI bug). The
fix: the approach's square-up waypoints on the measured green line, arrive radii the wobble
cannot outrun. walk_the_whole_mission_on_real_input 3 passed / 0 failed, 31855 ticks, falls [].
