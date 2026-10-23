# WORKLOG

## stuck-patrol-0c-0d
Started 2026-10-23 16:42 UTC. Branch s/stuck-patrol-0c-0d from origin/master a67f1a27e.
Task: two patrol movers are frozen from the level start (the stuck-guard sweep
crates/nf-view/tests/guard_stuck_sweep_live.rs):
- 0700000c drone 15, PATROL (0x07), stops at (9.30, 12.24, -220.82), 4.7 units from its home
  (6.55, 12.24, -224.56); nearest door 32.4 units away.
- 0700000d drone 33, PATROL (0x07), parked at its home (191.08, 13.66, -232.19) with a ~0.09 z
  jitter for the whole run; nearest door 44.0 units away.
Plan:
1. Trace each drone's patrol: its path record and node chain [D], the next node it is heading
   for, and the query that stops it (route follow BLOCKED / NO_PATH, the mover's collision
   response, an unreachable node) [M NDrone2_ReachedDestNode 0x153358 and the patrol functions].
2. Fix the cause in the shared code (never a per-level patch) and turn the two sweep rows into
   assertions: each guard walks its patrol. Sister: a guard whose patrol path is genuinely
   blocked in the game still stops.
3. If a live number is needed, say exactly which drone field to read in PCSX2 on a fresh boot.

Work log:
- (16:42) branch created, WORKLOG started; repo/ISO/work/queue-worktree already present from the
  earlier session; suite start deferred until tools/test/suite.py is confirmed on the tree.
- (16:52) the sandbox disk was full (9.9 G root): cleared the stale target dirs, the unused
  sandbox Python packages and the default Next.js project; rebuilt into /home/z/target-cache with
  the chunked runner (background jobs die between tool calls here).
- (17:00) reproduce: targeted sweep (SWEEP_LEVELS=0700000c,0700000d) — 0700000c drone 15 wedged in
  GO_TO_GOAL at (9.30, -220.82) after the sweep's own alarm; a no-alarm probe shows the same drone
  frozen in PATROL at the same spot (creep step 12/15 of path#9 node2→node3, next point 0.43
  ahead, bounds_node_test false, walk anim on, ±0.02 jitter). 0700000d drone 33 walks its pulley
  line on this master (0 sweep hits) — the driver's d-row does not reproduce.
- (17:25) cause [M 0x159f28 LinkCreep_Increment]: the game skips the boundary post read while
  SqDist2D(feet, q) < 1.0 and tests at r = 0.35 beyond; the port ran the 0.4 test bare in the
  0.4..1.0 band — a guard pinched between boundary posts (walls #313/#314 [D]) froze there. The
  same escape band lives in LinkCreep_Decrement 0x15a148 (the step-back's 0.1 test) and
  LinkCreep_Handler 0x15a2c8 (the block gate, 2D). Fixed all three in the shared Route::follow.
- (17:45) verified: probe shows step 12→13→next node; the two sweep rows are now assertions
  (patrol_walk_0c_0d_live: drone 15 completes a full 16-node lap, max 19.27 from start; drone 33
  covers 17.6 of its 20.4-unit line); two unit pins in nav.rs for the escape bands; nf-game lib
  524 passed, nav_kick_live 5 passed. Committed and pushed (e6e8fce09) after rebasing onto the
  driver's rewritten branch (5182dd38f).
- (18:15) report docs/reports/stuck-patrol-0c-0d.md: the residual GO_TO_GOAL wedge at (27.9,
  -218.0) is documented as left-over (mover-layer, target 2.04 away and clear — not the creep);
  the final-tree full-suite chunk run was still accumulating at the deadline.
