# retune-nb-0d-6 — the 0d arrival-to-C5-door 00 Agent walk, round 6

Branch `s/retune-nb-0d-6` cut from `origin/master` (72a68f828). `origin/s/retune-nb-0d-5` does not
exist on the remote (its queue done/ entry is a bare header with no report body), and master already
carries the rounds 1-5 shipped tree per the test's ignore-message ledger, so the branch is master +
this round's work only. Task: take the round-6 levers from the ignore message — (A) the g18 ride
lottery at the untried knob families, (C) the corner shave — with the driver's fallback (the pickup
scan near the death, then the idle-wait sweep) once the ignore-message levers run out. Helpers stay
gated to level 0d + 00 Agent (every knob is env-gated with the committed default = byte-identical);
real input only, no top-ups, no damage/health writes, no guard-code changes. Each lever judged on
its own real run.

## What's done

1. **The baseline reproduced byte for byte** (the committed shift-30 defaults): death t19772,
   21 hits for 80.8, other 19.2, #22 46.0, the floor-landing burst (#3 10.8 / #1 9.0 / #2 6.0),
   0.0 real-damage test FAILED as committed.
2. **Lever (A) — DEAD ON THE ZONE AXIS** (eight fresh rolls): every untried family
   `NF_0D_TAP_ZONE {40,45} x NF_0D_TAP_CADENCE {15,17} x NF_0D_RIDE_SHIFT {30,60}` dies ON THE
   RIDE — the girder health (24.0 at shift 30, 16.8 at shift 60) drains to 0 by the container-top
   landing (fall timer 99, landing (149.26, 8.23, -296.08) identical across all eight); the panic
   is the rim sweep's own guard ("the rim sweep must not kill Bond on the top"). The 40/45 tap
   zones fire extra taps on the ride and the extra draws re-roll the ride's fire onto the rider —
   the zone-axis twin of the round-4 cadence law.
3. **The pickup scan** (new `NF_0D_PICKUP_SCAN`, d3-gated, prints all placed pickups once):
   NO health pickup exists on 0700000d (kinds 0/1/3 only; no kind 6 BOND_BONUS) and NO pickup sits
   near the yard-floor death (141.22, 4.88, -296.08). The armour pickups (kind 3, +50):
   #0 (147.00, 21.76, -252.77) — y 21.76 is 2.1+ u ABOVE every route waypoint; #1 (205.41, 0.11,
   -205.31) — the closed pocket (the round-5 wake law); #2 (192.82, 10.79, -272.03) — the annex
   pocket, 10 u north of the bay-enter stand (194.0, 10.6, -282.0) at the same floor level.
4. **The post-landing idle wait sweep — DEAD** (the driver's fallback lever; new `NF_0D_LAND_WAIT`,
   d3-gated, committed default 40): {0, 10, 20} all die in the same 80.8 pool (deaths t19757 /
   t19767 / t19682). The burst fires at the landing regardless and the sweep's ~39-tick first
   kill cannot beat it; wait 20 rolled WORSE (t19682).
5. **The exit-fall drift steer — DEAD** (new `NF_0D_EXIT_YAW` radians knob in the drift loop,
   unset = byte-identical): {-2.356, -2.8, -3.14159} (south-west → south) move the landing
   ~1.5-2 u and stretch the deaths to t20135 / t20177 / t20171 (~400 ticks deeper into the sweep)
   but the same 80.8 lands and the landings cost MORE (11.4 / 13.2 / 9.6 vs 16.8).
   **THE INVARIANT: sixteen rolls this round all landed exactly 80.8 guard damage + 19.2 fall —
   the fight's damage classes are dice-stable; the walk needs a REMOVED damage source, not a
   re-roll.**
6. **The annex armour lever probed to its geometry**: the plain nav walk, the climb_stairs hop
   (reached y 11.08) AND the crouch crawl all WEDGE at z ~-276.4 (a waist-above wall on the bay
   side; the STUCK prints' segment_to_target blocked 0.65 u ahead). The 64-ray segment-fan probe
   (a segment from the pickup centre to a 2/3/4/6-u ring at root+1.0/head+2.2, 16 directions)
   maps the pocket OPEN to the west/north-west (r4 dirs 9-14 clear: stand spots
   (190.0,-269.2)..(191.3,-275.7); the r2-3 north mouth dirs 0-2 clear) and the r6 west corridor
   (187.3,-269.7)/(186.8,-272)/(187.3,-274.3) — and the committed roof lane's
   (190.0, 19.65, -268.7) sits DIRECTLY ABOVE the pocket's north mouth.

## The round-7 lever (in the refreshed ignore message)

**THE ROOF-EDGE ARMOUR DROP**: walk the committed roof lane to (190.0, 19.65, -268.7), walk SOUTH
off the roof's south edge into the pocket, land within the 1.91 u touch sphere of pickup #2 (the
+50 armour), exit through the probed west corridor south to the bay enter (the corridor's
along-length is UNPROBED — a probe walk first), rejoin the committed route. The ~9 m fall costs
the SAME ~19.2 the committed annex drop already pays [M 0x1949a8: raw = timer - 60, the d3 factor
1.2] — REPLACE the annex drop, not add. The +50 then eats #22's every-roll 46.0, the ride dice's
0x12 draws and the landing burst (the absorb law [M Player_HandlePain 0x1902c8:84-96; the trace
run's -0.00 entries are the absorbed 0x12 chips [L]]) — the route change re-rolls the ride dice,
but the absorbed ride-kill classes become survivable with the armour alive.

## How this was verified

- Sixteen real runs of the d3 walk this session (`--ignored --nocapture`, each its own roll):
  the baseline, the eight zone-axis configs, the three land-wait values, the three steer values,
  plus the annex-armour machinery runs (the leg wedges + the probe).
- The whole `walkreal_0700000d_live` binary on the final tree: **3 passed, 0 failed, 1 ignored**
  (99.7 s) — the plain master replay, the d1 real-damage walk and the d2 Agent sister all pass
  (every new knob is env-gated with the committed default = the master timeline byte-identical);
  the d3 walk stays ignored with the refreshed ledger.
- `interactables_0700000d_live`: **11 passed, 0 failed** (27.7 s).
- The fast runner (`tools/test/suite.py`): started in the background in the first minutes; the
  sandbox killed it between tool calls every time — `suite.out` stayed empty and no suite process
  survived (the same background-jobs-die signature rounds 1-5 recorded). The per-binary
  foreground runs above are the primary evidence, as in rounds 1-5. Crates touched: only
  `crates/nf-view` (one test file); no `crates/*/src/` and no `tests/common/` files changed, so
  no other level's binary is affected; the two binaries above are the ones the task names.

## What's left

- The walk itself: the round-7 lever is in the ignore message (the roof-edge armour drop; the
  west-corridor probe walk and the drop's landing spot decide it). The d3 gate stays `#[ignore]`d
  with the refreshed ledger (rounds 1-6 measured).

## Live checks for the main agent

- None new this session: the pocket geometry is answerable by the worker's own segment probe
  (done); a live check could confirm the pickup #2 pocket's ceiling/floor structure visually
  (the roof south edge at (190, 19.65, -268.7) vs the pocket floor ~y 10.2) and whether the west
  corridor at x ~186.8-187.3 runs south to the bay — a `--bond 190,19.65,-268.7,0,0` render
  looking south and a `--bond 187,10.6,-272,0,0` render would settle it.
- The carried live checks from rounds 1-2 (the arming event's address on g0; the 18-tap roll's
  config in the main agent's session notes) stand as written in `docs/reports/retune-nb-0d-3.md`.

## Usage

not visible to me — this session's environment shows no token, context-window or cost figures,
so there are no numbers to copy.
