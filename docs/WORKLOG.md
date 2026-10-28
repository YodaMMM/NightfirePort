# WORKLOG

## level-05-every-objective (2026-10-28, branch s/level-05-every-objective)

Task: make crates/nf-view/tests/walkreal_07000005_live.rs walk the WHOLE
mission 07000005 from a fresh load on real input (step_bond only; no teleports
past a blocker, no channel or state pokes): every objective the level's mission
data lists [D the objective records / Mission_MonitorObjectives 0x1827d8 M],
in the order a player does them, each asserted complete, ending on the level
exit firing. First list which objectives the current tests already cover; add
legs for the rest. Every blocker is a port gap: find the game's behaviour
[M/D] and fix it, or list it for the main agent if drawing only.

Plan:
- Fast runner started in the background in the first minutes.
- [D] list 07000005's objective records; [M] read Mission_MonitorObjectives
  0x1827d8 and how each objective type completes.
- Audit walkreal_07000005_live.rs coverage; add legs for the uncovered
  objectives in player order.
- Fix port gaps found on the way; drawing-only gaps listed for the main agent.
- Full suite on the final tree; push; report; finish steps.
