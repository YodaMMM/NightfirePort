# 07000001: a guard pushed off the upper west floor falls through

Continue blocked/10-guard-fell-through-floor.md (read its report and live-check list). With solid guard bodies a 01 guard was pushed off the upper west floor and fell through (y about -1e6). An earlier attempt (branch s/guard-floor-keep, if present) was stopped mid-way while proving its test red on the old code. Find why a pushed guard loses the floor (the body push moving him past the AI bounds or a cel edge the port lacks), compare with the game's drone bounds push and floor handling (AINetwork bounds, Drone_FeetOnPoint), and fix the port with a test red on the old code through real input. Run every walkreal_* binary one per call if guard movement changes. The two PCSX2 checks in the blocked report are the main agent's; name what they would settle.

Branch from origin/master, push to `s/guard-floor-keep-2`, report in docs/reports/guard-floor-keep-2.md. No draw changes.
