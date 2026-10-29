## level-4a-every-objective (started 13:31 UTC 2026-10-29)

Task: make crates/nf-view/tests/walkreal_0700004a_live.rs walk the WHOLE mission of level
0700004a from a fresh load on real input (step_bond only; no teleports past blockers, no
channel or state pokes): every objective the mission data lists [D objective records /
Mission_MonitorObjectives 0x1827d8 M], in player order, each asserted complete, ending on
the level exit firing. First list what the current tests already cover and every
stand-in/poke they carry, then add legs for the rest. Every blocker is a port gap: find the
game's behaviour [M/D] and fix it, or list it in the report if drawing only. Full suite,
quote its total line.

Plan: branch pushed from origin/master (1b2309ea) now; downloads (work-slim + ISO) next,
then read the existing walkreal_0700004a_live.rs coverage, the level docs and the mission
data, and build the missing legs.
