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
