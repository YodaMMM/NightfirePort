# WORKLOG

### level-09-use-pick-head (started 22:57 UTC, 2026-10-24)

Task: `interact_07000009_live::the_load_point_e_route_exits_with_objective_2_incomplete` is
ignored: the port's use pick runs from the RENDER CAMERA (scene.rs passes `camera` as the head),
while the game's `Player_Activate` 0x1a5170 builds it from `Player_GetHeadPos` [M :101-106 — the
use point = head + view]. At the 07000009 lift doors the camera hangs ~2.5 behind Bond, so the
1.0 use sphere never spans the exit box (z 10.33..12.49 [D LOAD_LEVEL #63]) and the E press never
fires. Make the use pick originate at the game's head position for every use (doors, switches,
pickups, ladders, wall-hug, lifts); search for the other callers that pass the camera as the head
and fix those too. Un-ignore the test; every interact_* and walkreal_* test green on master must
stay green.

Plan: fast suite in background; read Player_Activate 0x1a5170 / Player_GetHeadPos decompile; find
the scene.rs use-pick call and every camera-as-head caller; switch to head pos; un-ignore the
07000009 test; full suite; report.

- (22:57 UTC) Sandbox kept repo/work/ISO from a previous session: skipped downloads, branched
  s/level-09-use-pick-head from origin/master at 7dbc9cb1, pushing this checkpoint. Suite start
  next.
