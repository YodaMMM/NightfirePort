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
