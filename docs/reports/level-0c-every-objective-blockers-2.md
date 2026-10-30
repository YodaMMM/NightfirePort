# level-0c-every-objective-blockers-2 — the #283 ride re-derived; the whole 0c mission walk green on today's master

Branch `s/level-0c-every-objective-blockers-2` = origin/s/level-0c-every-objective-blockers
(a0cdf6dc WIP) merged with origin/master (21fbeed3 — the cable third-person work, the scoped
bullet re-query, the level-04/05 merges). The merge was clean. Task: the merged
`walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff` went RED —
"the #283 ride must drop onto the ring SE corner (11.09..11.17), got [35.15, 13.55, -209.95]
dropped true". Find what moved the drop, fix or re-derive with evidence, keep the walk green.

## Root cause: the drop was never moved — it was never validated

The merge brought **no player/world changes** (02e306a6..21fbeed3 touches
drone/lock/gadget/nis/progress/stats only — `git diff --stat`); the wire mechanics are
untouched. The WIP's own message says "the #283 ride unvalidated". Two stacked leg bugs:

1. **The drop timing put Bond over a void.** The corner floor's west edge is x ≈ 35.8 — a
   10.89/10.69 step at z −210..−209.8, the 11.06..11.09 corner east of it, a 11.93 lip at
   (35.8, −209.6..−208.8) [D ray maps, 0.1-step scan; the corner's 11.09..11.17 east half is
   the prior task's [D probe door2]]. The old `jump: d < 0.9` fired the FIRST tick inside 0.9
   of B (35.89, −209.44) — at (35.15, −209.95), 0.65 WEST of the edge, where the world has NO
   face at all (down-rays y 15..8 empty [D]) — the probe shows Bond falling straight to the
   yard (root 1.41). The RED's [35.15, 13.55, −209.95] is not a landing at all: 13.55 is the
   HANG pose (wire y 14.44 − the 1.0207 hang offset [M wire_tick 0x1976dc `hang`]).
2. **The settle loop snapshotted the hang.** wire_tick's drop path zeroes vel and returns; the
   stale `grounded` flag clears the NEXT tick [M 0x197758's exit arm; the probe's per-tick
   trace: exit tick grounded true, t1 false, then the fall]. The old `if grounded break`
   settled on the exit tick, so the assert read the hang root.

The ride itself reaches past B: the strafe zero rule parks the input at d_B ≈ 0.5
[M 0x1977c0's `d_b <= 0.5`], but the playing mode-7 move anim finishes its root motion and
overshoots to d_B ≈ 1.1 PAST B — (36.70, 13.41, −208.66) — over the 11.06 corner floor
[probe trace t720→stop].

## The fix (test-only; the assert is unchanged)

`walkreal_0700000c_live.rs`: the #283 leg now rides to the natural stop (d no longer shrinking
for 90 ticks) and jumps there; the settle waits air-then-ground (300-tick cap). The landing is
**[37.28, 12.10, −208.45]** — root 12.10 on the 11.06 corner floor — inside every existing
[D]-backed band (x 33.5..40.5, z −211.5..−207, y 12.2±0.8). The assert's bands and its [D]
citations are untouched; no assertion was weakened.

## Verified

- `walkreal_0700000c_live`: **2 passed / 0 failed** (the whole mission: 6366 ticks, stuck 0,
  falls 0; the real legs — the yard ladder, the #284/#282/#283 wire rides, the door2turbine E
  press, the x 41..43 slot jump — all green through the 0d handoff and objective 3).
- The merge commit + WORKLOG entries pushed at each step (5c458a62, e924cc99, 1c03a641).
- Scratch probes (`scratch_0c_ride{,2,3}.rs`, `scratch_0c_corner.rs` — untracked) carried the
  [D] ray maps and the ride traces cited above.

## Suite

The fast runner (`tools/test/suite.py`) launched 00:05 UTC with the full env: build ok,
**923 test binaries in 1168 s**, then the sweep on this tree (the 0c binary re-linked with the
fix at 00:46, before its turn). At the 01:30 UTC finalize point the sweep had logged **≈400 of
the 923 binaries** with **7 failure lines, every one a pre-existing environmental pattern on
this sandbox**: `identity_disc` + 3 × `disc_golden` FAILED (slim-ISO disc checks — the driver
allows the 3 `disc_golden`), `net_soak_faults_live` / `mp_shot_gate_live` CRASH exit −9 (the
sandbox OOM), `probe_pylon_render::probe_render_pylon_only` (no GPU — wgpu `adapter: NotFound`,
the lesson-6 pattern). **The 923-binary sweep does not fit the ~2 h window** (~8 binaries/min);
the full total line is left for the next window (the runner's log is not a repo file — rerun
`python3 tools/test/suite.py`). The task's own binary is the manual 2/0 above.

## What's left

- The suite sweep's remainder (the next window quotes the full total line; the runner's log is
  not a repo file — rerun `python3 tools/test/suite.py`).
- The blockers branch's remaining stand-in legs (the vent/display/secret-door/yard descents and
  the band crossing — the 0xE2-route gaps the blockers report maps) are OUT OF SCOPE here: the
  task was the #283 drop + the walk green on merged master, which is done.

## Live checks for the main agent

1. **The #283 ride in the real game**: fresh boot 0700000c, ride `wire_silo2turbine` to its end
   and jump. gdbwatch on `glb_players+0x30` (write) around the drop: expected — the ride
   carries Bond past B (35.89, −209.44) to ≈ (36.7, 13.4, −208.6) before the fall, and he lands
   on the 11.06..11.17 corner (root ≈ 12.1), NOT at B's x/z. If the real game instead drops him
   AT B (35.89) and he still lands, the corner floor's west edge on disc reaches x ≤ 35.8 —
   re-check the port's edge tris.
2. **The corner floor's west edge**: fieldprobe a screenshot at
   `--bond 37.3, 12.1, -208.4, yaw π, pitch 0` — should show the door2turbine corridor mouth
   west of the corner and the slot east.

## Usage

not visible to me — this environment shows no token, credit or context-window figures (no
status line or /usage output available); nothing to copy, and per the task's rule I did not
estimate.
