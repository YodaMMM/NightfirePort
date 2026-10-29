# WORKLOG

### level-13-every-objective (started 01:38 UTC, 2026-10-29)

Task: 07000013 (Air Force One) completable end to end on real input —
walkreal_07000013_live.rs walks the WHOLE mission from a fresh load (step_bond: move/aim/
fire/use; no teleports past a blocker, no channel/state pokes): every objective the mission
data lists [D objective records / Mission_MonitorObjectives 0x1827d8 M] in player order, each
asserted complete, ending on the level exit firing. First list what the current tests already
cover; add legs for the rest. Blockers are port gaps to fix with [M/D] evidence (draw-only
ones go to the main agent). Full suite, quote its total line.

Plan: push the branch first, background suite.py, inventory the mission data's objectives vs
walkreal_07000013_live.rs's current legs, then extend the walk.
