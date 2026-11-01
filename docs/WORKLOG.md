# WORKLOG

## level-0d-interactables-2
Started 2026-11-01 19:52 UTC. Branch s/level-0d-interactables-2 from origin/master 6042ee53.
Task: interactables_0700000d_live.rs has 11 tests; using interactables_0700000b_live.rs as the model,
add one live test per KIND of player-touchable placement in 0700000d (doors, switches/consoles,
fuseboxes, lifts, pickups, breakables, plates, monitors, rigid bodies, player-triggered SimpleScripts),
each driven through the real path (walk/E/shoot) with a sister that fails on the wrong result; fix any
port gap the tests expose, with evidence. Run interactables_0700000d_live, walkreal_0700000d_live and
nf-game; quote the totals.
Plan:
1. Census of 0700000d placements [D every placement kind].
2. Map each kind to the port's class Update/Activate; find what 0b's tests cover that 0d lacks.
3. Add live tests per kind + sisters; fix port gaps with [M]/[D] citations.

Work log:
- (19:52) branch created, WORKLOG started; setup in progress (work-slim + ISO download).
