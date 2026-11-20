# retune-nb-0d-13 — round 13 of the 0d arrival-to-C5-door 00 Agent walk

Branch `s/retune-nb-0d-13`, from origin/master ccd79b5be with origin/s/retune-nb-0d-12 merged in
first (0d-12 was not on master; clean merge). The walk `walk_arrival_to_the_c5_door_real_damage_00agent`
stays RED — but this round **LIFTED the round-12 binding constraint** (the escape walk's
exposure) and **re-measured the wall beneath it**: the armed pack's steady chips. Seven real
runs this round; the refreshed ledger and the round-14 lever are in the ignore message item (13).

## What was done

1. **The round-13 lever (a) as written (the waypoint thread) implemented and measured WORSE** —
   `NF_0D_WIRE_THREAD="x1,z1,x2,z2,..."` walks explicit mid-waypoints before the stand walk
   (each `try_walk_to`, arrive 0.6, test-file only, unset byte-identical). Run 13-b: the thread
   legs each paid ~90-175 ticks (the per-leg overhead), the whole stack 379 ticks, the stand at
   9.8 — worse than the single leg's 180/15.2. The premise was wrong, and the run that measured
   it told us why:
2. **The REAL drag pinned by the trace [L run 13-d]** (the same stack, `NF_WALKTRACE=1`, print
   only): the escape leg spent ~10 ticks turning in place, ~12 at full stick, then **~160 ticks
   crawling the walker's arrival zone** — the shared walker's `d < 2.2 -> forward 0.45` rule
   (`walker_b2.rs` steering), where the stick curve moves ~0.01 u/tick. The escape walk's
   167-241 ticks are MOSTLY this crawl, not the nav steering or the guards.
3. **The round-13 lever (a) v2, PROVEN: the fast-arrival leg** — a TEST-LOCAL `fast_leg` helper
   (no `common/` change): it holds the stick at 1.0 to the arrive check through the walker's
   own `tick()` (the same tick discipline, the fight hooks and the RNG draw pattern ride
   unchanged; only the stick magnitude in the last 1.6 u differs), the same 0.07 turn clamp and
   the walker's own stop-and-turn rule for err > 1.2, a 1200-tick leg cap. `NF_0D_WIRE_FAST=1`
   walks the escape leg with it; `NF_0D_YARD_FAST=1` generalises it to the yard's plain legs
   (Legs 2-4: the yard south 1, the plate, the display trench, back to the plate, the shooting
   stand). Runs 13-e/f: **the escape walk 105 ticks at cost 3.4 (the stand at 31.4 vs 15.2)**,
   the yard south 1 110-122 ticks. The round-12 "binding constraint" (the escape walk's
   exposure, ~0.106/tick × 167-241 ticks) is LIFTED.
4. **The round-13 lever (b), PROVEN: the land_wait shortened** — `NF_0D_LAND_WAIT=10` (the
   round-6 knob, already env-gated): the landing kept **34.8** — the committed 40-tick wait's
   burst ate armour 50→0 + 3.4 of health inside the wait; the 10-tick wait ate armour 0.2 only.
   The landing burst's sight events resolve after the walk starts instead.
5. **The round-13 lever (c) implemented and measured UNECONOMICAL** — `NF_0D_NORTH_KILL=1` runs
   the round-11 lever (c) (the certain box act on the north band, `kill_guard_certain_at` on the
   C5-door corner box, gated to the run variant; unset byte-identical). Run 13-c: the act's aim
   phase at d ~17-21 burns ~15-25 under the converged pack (the death mid-act; the plan print's
   cd(snap) 5.367 still far from converged). Killing #6 costs more than its back-fire chips.
6. **The new binding constraint measured** (runs 13-e/f/g): the armed pack's steady
   sight-event-keyed chips (~0.06-0.10/tick across the ~1000-1900 ticks of legs + acts) exceed
   the ~31 entry budget unless the pack thins. The 13-e sweep pass 0's act REFUSED (the aim-gate
   oscillation roll) and the pack drained 31.4 → 0 in 61 ticks (#6/#4 1.8-chips); the 13-f
   pure-run shape died at the plate to the accumulated chips + #29's roof 2.6 (d 15.8); the
   13-g sweep-1-pass shape died on the plate leg to #6's north-band back-fire (5 × 1.8 from
   d 26.9). Every certain act at d 9-21 costs 12-25 under the converged fire.
7. **The round-14 lever written into the ignore message: THE CROUCHED STAND ACTS** — press
   crouch at the stand after the fast arrival (the escape walk and the yard legs stay
   standing/fast), run the sweep's passes AND the trench certain acts CROUCHED: the crouch
   drops the sight target 0.45 [M Player_GetHeadPos 0x195aa0:39-51] and multiplies the
   visibility by 0.75 [M Drone_ProcessOpponents 0x1390c0:40-42] — the round-9 run 8 measured
   the crouched pass drain ~0.1/tick vs ~0.25 standing. The acts' costs halve; the crouch
   engages only at the stands, never on a walk gait.

No guard code changed, no damage/health constants changed, no top-ups, no `crates/*/src/`
change, no `crates/nf-view/tests/common/` change (the `fast_leg` helper is test-local in
`walkreal_0700000d_live.rs`; every new path is env-gated and default byte-identical). The
knobs-unset tree is the merged 0d-12 tree plus additive test-file code.

## How it was verified

- The runs (all [L], the direct test binary, --test-threads=1, --ignored, each its own roll,
  the composed shape STAND_FIRST+YARD_RUN+ANNEX+CORRIDOR 1+PIT_KILL+TAP_CADENCE 14+EXIT_CROUCH
  unless noted): 13-0 (the round-12 base) died on the RIDE (g18's 24.0, t22045 — the ride
  lottery; CORRIDOR=2 wedged the L4 pickup so armour stayed 0); 13-a (CORRIDOR=1) the ride
  survived, the escape walk 180 ticks −16.2, the stand at 15.2, death t22762 mid-sweep; 13-b
  (the thread) 379 ticks, the stand at 9.8, death t22997; 13-c (the thread + LAND_WAIT=10 +
  NORTH_KILL) the landing 34.8 but the thread still 211+75 ticks, the stand at 15.2, death
  t22898 mid north-act; 13-d (13-c + the trace, print only) the arrival-zone law above; 13-e
  (WIRE_FAST + LAND_WAIT=10 + NORTH_KILL) **the escape walk 105 ticks −3.4, the stand at
  31.4**, death t22803 mid-sweep (the pass-0 act refused, the pack drained 31.4 → 0 in 61
  ticks); 13-f (the pure-run shape + the fast legs) the legs fast, death t23177 at the plate
  (#29's roof 2.6 finishing the accumulated chips); 13-g (sweep 1 pass + the fast legs) death
  t22927 on the plate leg (#6's back-fire 5 × 1.8 from d 26.9).
- Whole binaries on the knobs-unset final tree (this sandbox holds 4 GB and ran them WHOLE,
  unlike round-12's 3 GB OOM): **walkreal_0700000d_live 3 passed / 0 failed / 2 ignored
  (102.0 s)** — `walk_arrival_to_the_c5_door`, `walk_arrival_to_the_c5_door_real_damage`,
  `walk_arrival_to_the_c5_door_real_damage_agent` all green; the 00agent walk and the
  hurt_census probe stay ignored — and **interactables_0700000d_live 11 passed / 0 failed
  (31.4 s)**.
- The fast runner (`tools/test/suite.py`) was started twice in the first minutes (plain nohup,
  then setsid) and this sandbox killed it between tool calls both times with `suite.out` empty
  (the rounds-1-10 signature, documented in the 0d-11 report); the four-crate test-binary build
  also exceeds the rootfs left after the ISO + work + toolchain (2.8 GB free vs ~5.5 GB
  needed). The per-binary foreground runs above are the evidence, as in rounds 1-12.
- Lesson-10 grep: the touched file is the 0d test file itself; no `crates/*/src/` and no
  `tests/common/` file changed, so no other binary is affected by construction; both 0d
  binaries were run whole anyway (above).

## What's left

- The walk: still red — the wall is now **the armed pack's steady chips vs the act costs**
  (the ledger item (x) in the ignore message). The round-14 lever (the crouched stand acts) is
  written there with its [M] citations. The machinery this round added (the fast legs, the
  land_wait=10) is proven and rides the knobs; the round-14 session can build the crouch
  presses on top (the presses gated to the container roll, the walk gaits untouched).
- The suite: needs a sandbox that keeps background processes alive between tool calls and a
  bigger rootfs; the four-crate build wall stands (2.8 GB free vs ~5.5 GB). Next session should
  run `tools/test/suite.py` whole on the merged tree if its sandbox allows, or per-binary with
  the tests chunked per process.
- Live checks for the main agent: none needed this round — the levers were judged on the
  direct test binary's own runs; the round-12 census [D] answer stands.

## Usage

not visible to me — this environment shows no token, credit, cost or context-window figures for
the session (no gauge, no billing panel, no API counters are exposed to the agent).

## ntfy

(to be filled after the finish steps)
