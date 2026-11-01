# real-damage-routes-2 — report

**Status: done, with one documented known-failure carried** (the merge + conflict resolution
landed; 3 of the 4 real-damage variants pass against master's guards with a walker-side fix; the
14 real-damage leg is the known failure, analysis below).

## The merge (already on the branch)

`origin/s/real-damage-routes-08-11-14-03` (d78b00ad) merged with `origin/master` (6042ee53, 172
commits: enemy-stats-4, stealth-los-4, hud-start-2, the walker keep_off/back_off, prefer_head,
sidestep_decays, rise_watch, bridge_wedged_nav changes) — commits 2bf0dace + 097ef1db on this
branch, both intents kept (the routes' spare + lane_fights windows; master's U-053 and walker
config order), `docs/UNKNOWNS.md` resolved with master's unique U- ids, and master's WORKLOG.md
deletion kept (dropped as a process file in batch-1101j; the routes branch's report stays).

## The re-run against master's guards, and the walker fix

- **03 — 5/5 pass** (incl. `walk_the_full_route_and_complete_every_objective_real_damage`).
- **08 — 4/4 pass** (incl. `..._real_damage`; the Hind's opening blasts now survivable on the
  merged tree — the routes branch's documented 08 blast-economics failure is gone with master's
  guards).
- **11 — 2/2 pass** after the walker fix (below). Without it the real-damage walk died at tick
  1425 on the 07000012 extension at (17.16, 1.39, 3.19): the floor fight engaged at **clip 2
  spare 0**, gave up `killed false` (the guard at 12 hp), and the post-fight dry resupply walk
  died to the shooter's twin (drone #4, weapon 0x19, d 11.8, 10 hits for the remaining 68.5)
  with no round to answer him. Master's decoded placement stats made every dry trade a death.
- **14 — 6/7**: the real-damage variant fails at `walkreal_07000014_live.rs:446` — the east
  stairwell foot reads (45.25, **0.213**, 53.20) against the −0.91 landing window. The plain
  climb's horizontal-only arrival (`climb_stairs`, dxz < r) let the return-fire fight shift the
  approach into the stairwell's deep-throat band (x 45..45.75 [D probe14_corridor]) whose lip
  reads y 0.21; the margin to the passing line is ~0.1 u. **Two walker-side fixes were
  implemented and reverted, each because it broke the calibrated top-up legs**:
  1. `climb_stairs_band` (a y-banded arrival with a step-off back-off) — the jump pulses read
     above-band mid-air and the oscillation wedged five previously-green legs;
  2. retargeting the three climb legs onto the landing's walkable west edge (x 44.6 [D
     probe14_stairs]) — the approach cannot reach x 44.6 (the throat column blocks it; five
     legs wedged in under 11 s).
  The pre-existing margin needs a route recalibration (an approach-leg re-plan west of the
  throat before the climb), which did not fit this window. The assert is [D]-backed and
  untouched; the next session continues from this branch.

## What ran (totals)

All with `NF_ISO` on the slim ISO, `--release`:
- **nf-game: 1670 passed, 0 failed** (lib 645 passed / 0 failed / 5 ignored; 99 test binaries
  1025 passed / 0 failed / 9 ignored, `--test-threads=1`).
- The four real-damage variant binaries: `walkreal_07000003_live` **5/5**, `walkreal_07000008_live`
  **4/4**, `walkreal_07000011_live` **2/2**, `walkreal_07000014_live` **6 passed / 1 failed**
  (the leg above).
- **The full walkreal_* sweep could not run**: building every nf-view test binary is link-bound
  (~100 binaries, release) and two 10-minute build windows did not finish it; this sandbox kills
  background jobs between tool calls (three detached fast-runner starts died within minutes), so
  the sweep could not run detached either. The four variant binaries the task names are the
  reported walkreal coverage; the next session should run the fast runner once on this tree.

## What's left

1. The 14 real-damage leg (above) — the route recalibration west of the deep-throat band.
2. The full walkreal_* sweep + the fast runner on the final tree (build-bound here).
3. The full walkreal sweep may surface more master-guard deltas of the 11 class; the lane kit
   (the 07000013 precedent, per-route `lane_fights`) is the proven remedy shape.

## Live checks for the main agent

1. 07000014 east stairwell foot: `fieldprobe` at (45.25, ~0.2, 53.2) — what geometry is at the
   x 45..45.75 "deep throat" band (a raised lip the walker stood on, or a hole)? Settles whether
   the recalibration should avoid the band or use it.
2. 07000012 lower floor: a live read of the two guards near (28, 1..7) — decoded stats (the
   accuracy/aggression the port now decodes) for the fight that killed the walker at tick 1425.

## Usage

not visible to me (no token, context-window or cost figures are exposed in this environment).
