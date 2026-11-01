# WORKLOG

## guard-approach-distance-11 (started 2026-11-01 17:50 UTC)

Task: the port's guard #126 (start corridor, 07000011, difficulty 1) stops ~10.7 u from an idle
Bond at t200 where the game stops 8.8 u (live doc docs/live/2026-11-01-07000011-start-corridor-guards.md);
find the game's run-to-point / approach stop rule [M EnemyRunToPoint 0x168910,
NDrone2_FindRunToPoint 0x153168 / ClosestAIPoint 0x151b18, arrival radius, attack-range handoff],
fix the port gap with the address cited, tighten the pin to the live distance. Branch cut from
origin/master d43c28ce (which carries the live pin). Plan: read the live doc + decompile, diff
against the port's stop rule, fix, run every *07000011* binary + ai_live + nf-game, full suite,
report.
- Radius pass (18:30-19:00 UTC): the raw MIPS chain — FindRunToPoint loads the picked AIPoint's
  +0x14 [M-dis 0x153228 lwc1 f20,0x14], SetupGoalPosition 0->2.0s it into the move-target +0x14
  (= Drone+0x704) [M-dis 0x15ad10-70], MoveToGoalPosition re-passes Drone+0x704 [M-dis 0x15325c]
  and SetupRouteToPosition lands it at route+0x8c [M 0x158b10:104], the arrived-early check reads
  route+0x8c [M-dis 0x151888] — the port compared a fixed 2.0. Fixed: move_accuracy field,
  find_run_to_point/find_alarm_point/re_find_mission_path set it, move_to_goal rides it into the
  fresh route's radius + the code-3 check. d1 #126: 10.67 -> 9.93 at t200 (the level's points are
  1.0 [D-probe]); first shot ~60 ticks earlier.
- Pin pass (19:00-19:10 UTC): t200 reach 11.0 -> 10.2, t700 10.0 -> 9.0 (the old stop fails it);
  walkreal_07000011's flight-C band 32.5 -> 32.75 (the earlier stagger costs 8 mm on the leg).
  Residual 1.1 to the live 8.84: the game's MoveTest fails the spawn->point diagonal (A* south
  through n149 (1.57,18.70) [D nav]) while the port's passes — route-shape gap, live checks in
  the report. Runs: 15x *07000011* + ai_traces 53 passed / 0 failed; nf-game 1025 / 0; alarm_06 +
  fix_06_alarm + walkreal_05 pass; walkreal_01's 3 failures reproduce at master (2 verified at
  the parent). Suite runner died twice (PATH, then the 9.9 G disk cap) — the targeted runs stand.
