# WORKLOG

### level-07-every-objective (started 03:35 UTC, 2026-10-29)

Task: make `crates/nf-view/tests/walkreal_07000007_live.rs` walk the WHOLE mission 07000007
from a fresh load on real input (`step_bond` move/aim/fire/use; no teleports past a
blocker, no channel/state pokes): every objective the level's mission data lists [D the
objective records / Mission_MonitorObjectives 0x1827d8 M] in player order, each asserted
complete, ending on the level exit firing. First list which objectives the current tests
already cover (and every stand-in/poke they still carry); add legs for the rest. Every
blocker is a port gap: find the game's behaviour [M/D] and fix it, or list drawing-only
gaps in the report. Keep any ignore only with the exact blocker and evidence. Full suite,
quote its total line.

Plan: push this branch, start tools/test/suite.py in the background now, inventory the
disc objective records for 07000007 and the current test's coverage/stand-ins, then add
the missing legs on the real input path; fix port gaps the walk proves broken.

Progress (05:34 UTC): the objective-2 reveal chain is committed and pushed (91927210) and
re-verified on the rebuilt binary — `walkreal_07000007_live` 2 passed / 0 failed / 14.40 s
directly, both new assert messages confirmed in the binary; `walkreal_07000008_live` passed
inside the suite (3 passed, 94.12 s). The full-suite runner is grinding in the background
(329/892 binaries, interim 1134 passed / 2 env-only failed / 86 ignored, 2 OOM CRASHes at
last read; it needs longer than the sandbox window). Report drafted at
docs/reports/level-07-every-objective.md.
