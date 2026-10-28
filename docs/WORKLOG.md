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

## level-09-every-objective progress (21:15 UTC)

Coverage survey [D mission/c07000009.rs — 3 rows]: the current walkreal_07000009_live already
covers ALL of 07000009's mission objectives through real input — obj 0x4000016 (ch 0x61 flags 2)
asserted UNFAILED at the exit (state 2) with the fail arm pinned by the alarm-switch sister +
security_alarm_09_live's real chain; obj 0x4000017 (ch 0x46) asserted COMPLETE (state 4) at the
real panel E-press; obj 0x4000018 (ch 0x47, enable 0x46) asserted ANNOUNCED (state 3) — ch 0x47
has no writer on 09 (it completes on 0700000a) [D census; the doc's objective table]. The walk
ends on the exit firing (0700000a, 300-tick fade, end markers). Verified GREEN on the merged
master tree (3/3, exit at +28 cab ticks, 4010 ticks total, 0 stuck/falls).

The one remaining task bar was the walk's `hold_npcs = true` state hold. Live-guard walk
attempted (v1-v3, real fire path): the car-park sentry never sights the spawn (732 ticks beside
him, alarm low — Bond's east route is safe); the stair-head pinch guard (drone 4, CIVILIAN_
CHALLENGE state at (20.59, −372.08, 22.0), k15 13 hp [D drone-props]) takes ZERO damage from 15+
rounds at 1.2-7.7 u point blank across three runs (rounds fire and drain, no damage, no flinch)
— either a bullet-pick gap against a challenge-state drone or a geometry shadow unique to the
mid-stairwell pose; needs a dedicated session (the decompile hooks: NDrone2_BulletImpact
0x1461c8, the challenge state 0x3E NDrone2_DSTATE_CivilianChallenge 0x162558, the port's
bullet-vs-drone pick). Reverted the walk to the held-NPC version (the interactive chain +
objectives all assert; the hold is the documented dev facility used across these tests); the
live-guard walk stays the open item for the next message.

Suite slices to follow before the final push.
