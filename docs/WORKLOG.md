# Worklog

## level-09-every-objective (branch s/level-09-every-objective, from origin/master ad2f0791)

Sandbox opened 20:05 UTC, expires ~22:00 UTC (stop new work 21:40). Scope: make
`crates/nf-view/tests/walkreal_07000009_live.rs` walk the WHOLE 07000009 mission from a fresh
load on real input — every objective the mission data lists [D objective records /
Mission_MonitorObjectives 0x1827d8], in player order, each asserted complete, ending on the
level exit firing. Blockers are port gaps: fix with [M/D] evidence, or list draw-only items for
the main agent. Full suite + total line before the final push.

Setup reused from the previous session's sandbox (repo, work/, ISO, rust, ALSA prefix,
../nf-queue all present); freed 3.4 GB of stale build artifacts first.

Plan: (1) read the existing walkreal_07000009_live.rs coverage + the mission-objective records
for 07000009 [D]; (2) diff covered vs listed objectives; (3) add legs on real input; (4) fix
port gaps with cites; (5) suite slices + report.
