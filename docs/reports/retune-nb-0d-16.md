# retune-nb-0d-16 — round 16 of the 0d arrival-to-C5-door real-damage walks: the batch-3 re-measure + the round-16 d2 lever set; BOTH walks still red, the ledger refreshed

Branch `s/retune-nb-0d-16`, from current origin/master (6691acd8c, stream batch 3 landed) with
origin/s/retune-nb-0d-15 merged in first (0d-15 was not on master; the merge was clean — one
false start aborted: the merge first landed on the stale sandbox checkout `s/port-map-scripts`
and was aborted there before the branch was created correctly). **The task walk
`walk_arrival_to_the_c5_door_real_damage_agent` is still RED — the ignore stays, refreshed with
the round ledger and the next levers.** Seven measured runs this round (16-0..16-5 plus the
whole-binary set).

## The batch-3 re-measure on the merged tree (the task's first instruction, Agent first)

1. **16-1, the Agent walk with its committed shape (11000/45)** [L, the direct test binary]:
   DIED t4397, 6 hits for 100.0 (#19 28.0 @ d 55.3, #31 28.0 @ d 65.5, #20 28.0 @ d 29.7, #15
   10.0, #22 6.0) — the batch-3 ignore ledger's exact signature, REPRODUCED on the merged tree.
2. **16-0, the 00agent walk (the 0d-15 committed green shape)** [L]: **DIED t3995 at the same
   ladder top (213.06, 17.37, −234.51), 4 hits for 100.0** — stream batch 3 (guards hit body
   boxes and aim at the body) broke the 00agent green too. The 0d-15 green was measured on the
   pre-batch-3 tree; the merged tree's 00agent red is the batch-3 regression, present BEFORE
   this round's changes (the run was taken before any edit).
3. **16-2, the ledger's best shape (3000/60 + a second stop 11000/45)** [L]: DIED t19710, 34
   hits for 70.6 + 29.4 other (falls), at (137.59, 4.63, −296.21) — the ignore ledger's
   yard-landing wall, reproduced.

## The round's new fact: the landing sweep's own acts are the killer

The deep run's trace (16-2..16-4) pinned what the old ledger could not: Bond does NOT stall in
a post-sweep fight pin and the walk legs never start — **Bond dies INSIDE the landing sweep's
own pass-3 act**. The nearest-first pick law picks #2/#7 (their box aims pass the pick's
clear_box_aim gate), the act stands Bond the 200-tick cap in the open, the act's ray never
converges (no kill print), and the unpicked chippers (#2 (143.3, −299.8) d 6.7, #7 (160.8,
−288.4) d 24.5) finish him mid-act. "0 return-fire fights" — the walker's fight answer never
engaged anything; the earlier fight-pin theory is refuted for this death. The probe prints
(A/B) added and removed this round proved the flow never reaches post-sweep code in the
baseline shape.

## The round-16 lever set (all d2-gated, d1/d3/plain byte-identical)

Committed in `crates/nf-view/tests/walkreal_0700000d_live.rs` only:

- **The d1-proven sweep discipline extended to d2** (the Operative walk is GREEN with it on
  this level): the 200-tick pass caps + the failed-guard blacklist (`skip_env` now d1+d2) and
  the STANDING posture (the 5 crouch presses → 0 for d2; the 0d-8 law: the crouched pass still
  drains ~0.1/tick; the d1 ledger: standing swept better). The post-sweep stand press only
  stands a crouched sweep now. Measured (16-3): pass 1 failed at exactly 200 (the cap works),
  pass 2 killed #3 at +53 — the sweep shape changed as designed.
- **The pool floor 18 break** (the yard_run `sweep_floor` law) inside the d2 landing sweep.
  Measured (16-5): the floor broke at pool 6.5 after three passes — the three acts cost ~30
  through the armour.
- **The return-fire skip for the d2 yard legs** (the run-9 law) + **the stand re-open**: the
  fast_leg walk to the probe's proven-clear stand (142, −291.5) + a second nearest-first sweep
  from there (the container path's own proven shape). Measured (16-5): the stand walk
  fast-arrived at +64 ticks (141.10, −292.69), the re-open sweep ran from health 2.25 armour 0
  — **the death moved +402 ticks deeper (t20142)** and Bond got further than every prior round.
- The test fn's shape is the ledger's best (3000/60 + 11000/45) — the deepest known point.

**The wall is the pool**: 29.4 of falls (raw 42 × the d2 Normal 0.7 [M 0x1949a8 x 0x1902c8] —
the slot hop re-rolled long at this stop shape; hold 13 measured 10.2 fall damage green on the
old shape [L s/hard-walks-a]) plus the ladder/deck drains leave ~2 health at the re-open stand.
No lever inside the yard can spend a pool that is not there.

## NEXT LEVERS (in the ignore message, untried)

(a) the hop/fall re-tune at this shape — the `NF_0D_HOP_HOLD` grid re-rolled (8/12/13/14/20/24);
(b) the 0d-15-proven d3 acts gated to d2 (NORTH_S1 the #6 corner act from yard-south-1 d ~15.6,
NW_KILL the #4 early catch at the park band — the act sites already run on the d2 legs, only
the knob gates differ); (c) the pool floor value + the re-open sweep's pass cap re-tune.

## How it was verified

- The runs (all [L], the direct test binary, --test-threads=1, each its own roll): 16-0 the
  00agent t3995 DIED (pre-edit); 16-1 the Agent t4397 DIED (pre-edit); 16-2 the deep shape
  t19710 DIED; 16-3 the sweep discipline t19740 DIED (identical totals — the death INSIDE the
  pass-3 act); 16-4 the skip flag t19740 DIED (identical — the flag never reached: the death
  precedes it); 16-5 the full lever set t20142 DIED at the re-open stand +402 deeper; the
  whole-binary run below is 16-6.
- **Whole binaries on the final tree**: `walkreal_0700000d_live` **2 passed / 1 failed / 2
  ignored** (the plain walk + the d1 Operative walk GREEN — the d1/plain byte-identity holds;
  the failure is the 00agent batch-3 regression, pre-edit [L 16-0]; the Agent walk ignored) in
  70.87 s; `interactables_0700000d_live` **11 passed / 0 failed** (30.50 s).
- **The suite** (`tools/test/suite.py`): this sandbox kills background processes between tool
  calls (the rounds-13/14/15 signature, reproduced — `suite.out` empty twice), and the four-crate
  build does not fit the rootfs with the disc+work mounted (the rounds-13/14/15 ~5.5 GB wall,
  hit twice this session; the target dir was wiped and rebuilt around it). The suite ran
  per-binary in the foreground on the final tree: **the three non-view crates' whole suites
  (every built test binary, --test-threads=1): TOTAL 2153 passed, 71 failed, 11 ignored
  (203 s)**. The failure classes, none from this round's diff (it touches only the 0d nf-view
  test file — no `crates/*/src/`, no other test file): the slim-ISO permitted misses (disc_golden
  3, crp_mesh_disc 2, identity_disc 1 — the missing cross-check dumps); the dump/probe/census
  binaries that are explicit-run tools, not pass/fail tests (the `dump_*`/`probe*`/`*_census`
  set); and the driving_* failures (driving_sniper_live 0/9, driving_roads 18/3, driving_ai_*)
  which are pre-existing on the merged master (0d-15's tree showed them green — master advanced
  past 0d-15's base with the drive/moving-collision work; the round had no time to audit them
  and changed no code that could affect them).
- The two 0d nf-view binaries above ran on the final tree; the remaining nf-view binaries were
  not built (the rootfs wall — same statement as rounds 13/14/15).
- Lesson-10 grep: the touched file is `crates/nf-view/tests/walkreal_0700000d_live.rs` only;
  `grep -l kill_guard_certain_at|shoot_far_box|fast_leg crates/*/tests/*.rs` lists other files,
  but this round changed none of them — no other binary is affected by construction; both 0d
  binaries were run whole anyway (above).
- No `crates/*/src/` change, no `crates/nf-view/tests/common/` change, no guard code changed,
  no damage/health constants changed, no top-ups on the walk phases; every lever gated
  `difficulty == 2 && real_damage` (or d2-only branches of the shared sweep knobs).
- Live checks for the main agent: none required this round (no visual work; the deaths are
  fully instrumented by the test's own damage log).

## What's left

- The Agent walk is still red: the ignore message carries the refreshed ledger and the three
  next levers (a)/(b)/(c) above; the committed lever set is the base the next round tunes.
- The 00agent walk needs its own re-green on the batch-3 tree (t3995 at the ladder top — the
  0d-15 shape's ladder-top acts now stand in batch-3 body-box fire; the same act-exposure law
  this round measured for d2).
- The full four-crate suite still needs a bigger rootfs (the rounds-13/14/15 wall); the
  per-binary evidence above is this round's substitute.
- The driving_* suite failures on the merged master deserve an audit round of their own.

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures
for the session (no gauge, no billing panel, no API counters are exposed to the agent).
