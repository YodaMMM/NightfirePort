# WORKLOG

## level-01-every-objective (2026-10-29)

Branch `s/level-01-every-objective`, task: make `crates/nf-view/tests/walkreal_07000001_live.rs`
walk the WHOLE mission 07000001 from a fresh load on real input, every objective [D] MissionData
/ [M] Mission_MonitorObjectives 0x1827d8 asserted complete in player order, ending on the level
exit firing.

- 03:57 UTC: sandbox intact (ISO + work/ + nf-queue). Branch created from origin/master 113dcdcd.
  Background suite starting next.

## level-01-every-objective — 06:45 UTC 2026-10-29
Sandbox reset mid-task; re-setup (clone, work-slim, ISO, user-prefix ALSA with pkg-config libdir fix, rustup). Merged origin/master (a213e841) so the suite runs on the final tree. Test fixes from d9c51ceb stand: walkreal_07000001_live pins obj0 announced-but-not-folded (fold belongs to 07000002 intro stream 7 [D 07800002 0x06000084]); walkreal_07000002_live asserts the cross-level fold after two settle ticks [M 0x1827d8 case 3]. Next: full suite on this tree, then report.
