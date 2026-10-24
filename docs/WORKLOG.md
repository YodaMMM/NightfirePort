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

- (23:10 UTC) Evidence traced end to end: run()'s `camera` param is step_bond's
  `self.bond.head()` binding (scene.rs 3228→3256, no shadowing), so the doors/trigger/monitor
  arms were ALREADY head-based at runtime — the "render camera" story (garage-lift-calls
  comment, the test's ignore note) misread the arg name. The ignored test's premise dies on
  three gates: [M 0x1b2318:38-40] type-1 Trigger_Activate needs mask&4 and #67's disc mask is
  8 [D params (0,50),(1,8)]; [M 0x1a9ea8] SS_Activate needs mask&4 and the lift doors' SS mask
  is param 2 = 1 [D]; the Load Point's own pick plate is z 10.232..10.283 [D #63 collision],
  ~1.6 beyond the pressed-in sphere z 11.85..13.85. Code: head passed explicitly for every arm
  (scene.rs 4840/4853), false camera comments corrected (scene.rs, drone.rs). Test rewritten
  to the proven negative with the full citation block; 15/15 green on interact_07000009.
  Checkpoint pushed. Next: the wider interact_*/walkreal_* set, full suite, report.
