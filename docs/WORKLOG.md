# WORKLOG

## level-0a-every-objective (2026-10-29, UTC)

- Task: make `crates/nf-view/tests/walkreal_0700000a_live.rs` walk the WHOLE 0700000a mission from a fresh load on real input (`step_bond` only), every mission-data objective asserted complete in player order, ending on the level exit firing.
- Branch `s/level-0a-every-objective` created from origin/master at start of session.
- Plan: (1) audit existing test coverage + the 3 ignored tests; (2) list objectives from mission data [D] / Mission_MonitorObjectives 0x1827d8 [M]; (3) add legs for uncovered objectives, fixing port gaps found on the way ([M/D] cited); (4) full suite + report.
