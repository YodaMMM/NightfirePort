# WORKLOG

## cite-take-ray-tolerance

- (13:30 UTC, 10-24) Branch `s/cite-take-ray-tolerance` created from `origin/master` (26543436).
  Task: find the decompile line that gives the take ray's 1e-3 end-band tolerance claimed at
  `crates/nf-game/src/pickup.rs:810` (VU0 fixed-point triangle test absorbing a sub-quantum
  2^-14 contact, 0x18e148's fall rest); cite it [M address:lines]; if the decompile shows a
  different band or none, fix the code and test to it; if it cannot be shown, mark the band
  `// [S] unverified` and name the settling PCSX2 read. Note: master dropped the root WORKLOG
  and docs/WORKLOG.md (batch-1023c); re-adding docs/WORKLOG.md on this branch per the current
  task-branch convention.
