# WORKLOG

## level-08-every-objective (2026-10-28, branch s/level-08-every-objective)

Task: make crates/nf-view/tests/walkreal_07000008_live.rs walk the WHOLE 07000008
mission from a fresh load on real input (step_bond only; no teleports past a
blocker, no channel or state pokes): every objective the level's mission data
lists [D objective records / Mission_MonitorObjectives 0x1827d8 M], in the order
a player does them, each asserted complete, ending on the level exit firing.
First list which objectives the current tests already cover; add legs for the
rest. Every blocker is a port gap: find the game's behaviour [M/D] and fix it,
or — if drawing only — list it for the main agent. Full suite; report; finish
steps.

Plan:
- Fast runner started in the background in the first minutes.
- [D] the level's objective records (07000008.bin) + [M] Mission_MonitorObjectives
  0x1827d8: the objective list and completion channels.
- Survey walkreal_07000008_live.rs: which legs exist, which objectives covered.
- Add the missing legs on real input; fix the port gaps the blockers expose.
- Full suite on the final tree; push; report; finish steps.
