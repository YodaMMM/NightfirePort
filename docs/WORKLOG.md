## level-4a-every-objective (started 13:31 UTC 2026-10-29)

Task: make crates/nf-view/tests/walkreal_0700004a_live.rs walk the WHOLE mission of level
0700004a from a fresh load on real input (step_bond only; no teleports past blockers, no
channel or state pokes): every objective the mission data lists [D objective records /
Mission_MonitorObjectives 0x1827d8 M], in player order, each asserted complete, ending on
the level exit firing. First list what the current tests already cover and every
stand-in/poke they carry, then add legs for the rest. Every blocker is a port gap: find the
game's behaviour [M/D] and fix it, or list it in the report if drawing only. Full suite,
quote its total line.

Plan: branch pushed from origin/master (1b2309ea) now; downloads (work-slim + ISO) next,
then read the existing walkreal_0700004a_live.rs coverage, the level docs and the mission
data, and build the missing legs.

Progress (14:55 UTC): the mission walk was already green on master (walkreal ride 5.5s,
satchel 7.9s — the level's ONE objective "Gain access to lobby" reveal ch 0x4d asserted,
level exit ch 0x84 → LevelEnd asserted). The real work became GAP-1: the two interact
spawner tests were parked on "the car walls do not block the Drone_PositionVisible stand-in".
Probe (probe_4a_spawn_live) redid the diagnosis: at the parked z −12.4 the rays ARE blocked
and ch 0x17 never fires (the park leaves every plate's volume — the arm never latched); on
the ride line the spawner arms but every template sits 7.6–7.7 away in x/z forever — the
port's min-dist screen was 2-D, and the game's is `Vec_SqDist3D(player+0x30, spot) >=
min*min` [M 0x137cb0:28-31; the 10.0 default 0x1378d0:35-37]. FIXED the metric to 3-D —
the garrison now releases on the ride line (spawn at the arm tick, the far-down templates
pass the sphere at once); `the_floor_spawner_releases_its_drones_on_the_floor_plate`
un-ignored and green + the no-fire health sister moved in (the spawn health is the port's
stats-default 10.0 [M 0x1c7e58; L 07000005] — the old param-5=0x64 [D] reading pinned the
behaviour words, test fixed per the discipline). The fire-across-the-shaft leg RE-PARKED
with the new exact blocker: 0/2400 clear eye→drone rays from the ride line (the shaft core
blocks the x ≈ −4.3 spots [D census]); game-side reachability unverified [S] — live check
in the report. walkreal + satchel re-run green with the garrison live (6.6s / 10.2s).

Progress (17:05 UTC): DONE — the suite chunks ran in the foreground (background dies between
tool calls in this sandbox): all 893 binaries started, aggregate 3433 passed / 7 failed /
197 ignored / 17 binary-timeouts. Failures classified: 3 disc_golden + identity_disc (the
slim-ISO class), fix_0c_pylon_live ×2 re-run GREEN-check FAILED on the pre-fix drone.rs
(master-preexisting), probe_render_pylon_only passes direct with both trees (env). Timeouts:
walkreal_07000003 4p/387s, walkreal_07000008 3p/94.8s, weapon_matrix 6p/133s uncapped (slow,
not broken); the rest the sandbox hang class. Level docs updated (0700004a.md §7 spawner row
+ live checks; audit GAP-1 → RESOLVED). Report at docs/reports/level-4a-every-objective.md.
