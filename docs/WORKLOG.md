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

- (13:55 UTC, 10-24) Census done. The take ray's triangle test is `Intersect_RayGeom` 0x10a8c8
  (0x18e148:212-214 → 0x1edf50:11-12 → 0x1ebaa8:180-229 dispatches ident-0x201). Quantisation
  shown: int16 vertices × (2^-14, 2^-14, 2^-14, 1.0) [0x10a8c8:138-149, 315-317; 0x1e92d8:185-187].
  End band NOT shown: t-range exact (0<=t<=1.0 [168-172], candidate needs t < best, best = 1.0
  [95,135,180]; edge tests no epsilon [220-244]) — only an exact t == 1.0 contact is clear. The
  "absorbs a 2^-14 contact" claim removed; the 1e-3 band marked `// [S] unverified` (port's
  float-noise slack), value unchanged (the game's rest never sinks the endpoint [0x18e148:150-173],
  the port's can — batch-1022l). pickup lib tests 18 passed. Report:
  docs/reports/cite-take-ray-tolerance.md (live checks for the main agent listed there).
