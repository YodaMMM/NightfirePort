## Your task: `real-health-00agent-1b-10`, 0700001b's 00 Agent walk survives the console wave and reaches Drake

Branch from current origin/master; if origin/s/real-health-00agent-1b-9 is not on master, merge it in first. Read docs/reports/real-health-00agent-1b-9.md there and the ignore message on walk_arrival_to_drakes_defeat_on_real_health_00agent. Bond now dies at 10816 with the wave 10 strong and k1 unlatched; take the next lever the ignore message names. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_0700001b_live binary and interactables_0700001b_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-1b-10`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-15T04:17Z

---
## Report (done, 2026-11-15T05:22Z)

Closed by the driver: the report is on `s/real-health-00agent-1b-10` (docs/reports/real-health-00agent-1b-10.md); the worker didn't run finish.sh.

---
## Report from the worker (done, 2026-11-15T05:52Z)


---
## Report from the worker (done, 2026-11-15T05:52Z)

# real-health-00agent-1b-10

Task: take the round-9 ledger's named lever (the MID-PHASE spawn-window gate — the screen
window must not open while the f5100 spawn wave is un-thinned) on 0700001b's 00 Agent
real-health walk
(`walkreal_0700001b_live::walk_arrival_to_drakes_defeat_on_real_health_00agent`, still
ignored), measure it on its own real run, and write the ledger back.

## What's done

1. **The named lever implemented and measured RED (anti-lever)** [L /tmp/lever1_r10.log, the
   gate hard-gated to the 00 Agent walk's mid-defense block]. The gate: a spawn wave is
   detected by the wave count growing across an observation (the fight's own `n0` vs the
   fight-end count, or vs the previous fight's end — spawns are the only source of new
   astronauts [S], deaths only reduce); its pre-spawn level is the last observation before the
   growth; the streak window (the two-consecutive-zero-kill-full-cap rule, CHAIN_NEED's
   `tight` arm kept) may only open when the live wave is back at/below the pre-spawn level.

   The run: the gate blocked the 6114 streak window exactly as designed — and the walk died
   ~8100 (health 0.0, k5/k4/k3 latched, k8/k6/k1/k2/k7 UNLATCHED), **worse than the 10816
   baseline**. The mechanism, measured in the log:
   - the gate's pre-spawn level was **1** (the 4930 observation — the last one before the
     5100 spawn waves landed during the k3 screen work, unobserved): the mid waves land
     faster than the fight loop can thin to 1, so the window would stay shut every round;
   - the freed ticks went to the fight loop: fights 6114..6273 (astro 7 -> 5, 2 kills) and
     6414..6635 (5 -> 3, 2 kills, only −5.0 hp — **the thinning itself works at the
     crossing**);
   - but the 6414 fight started with `answered[pi8] == 0` (the chain fence passed) and then
     **covered k8's hold windows #1/#2** (the baseline's 6452/6560/6668 [L round-7]) — the
     holds are a 3-hold resume chain; missing #1/#2 kills the chain. ZERO k8 holds answered
     6635..7774, the channel never wrote, and the walk stood at the k8 station unprotected
     (the wave sniped at lined < 2, no fight fired): hp 69.9 -> 24.9 by 7774, dead ~8100;
   - k3's latch also slipped +299 (baseline +140) with the window's screen ticks gone.

   The gate is **reverted**; the shipped code is exactly master's (the diff on the test file
   is the ignore message only).

2. **The re-pin** [L /tmp/baseline_r10.log, on master 4f3802517 + this file, 22.4 s]: the
   current tree reproduces **lever 7's** round-9 run — death **11002** (the 10816 headline in
   the round-9 ledger was lever 5's kite; the current code carries the kill arm + the gated
   climb, lever 7's shape): k8 −1104 / k6 +1309 latched, the dead window 7120..9083 shelter
   1963 ticks hp 47.7 -> 31.0, astro 6 -> 10 across it, the post-window fights thin 10 -> 6
   by death, k7/k1/k2 UNLATCHED — the wave grind wins the ~1300-tick k7/k1/k2 chain queue.

3. **The refreshed ledger and the next lever are in the ignore message.** NEXT LEVER (named):
   the **CHAIN-WINDOW-FENCED THINNING** — keep the streak window exactly as master (the k8
   chain needs it) and fence every mid defense fight additionally on the pending console's
   held targets being LIVE (`held_hit_targets` non-empty for the pending player
   pi = gate 30 + k [M scripts.held_hit_targets]): a fight that starts while the pending
   screen is offering a hold breaks the 3-hold resume chain (measured above). With the fence,
   the 6414-class fight either fires BEFORE k8's first window (~6452) or waits, and the fresh
   spawn thins in the gaps (7 -> 3 measured at −5.0 hp). Fallback lever if the fence starves
   the thinning: thin the crossing wave BEFORE k3's hold#3 (the 5514/5814 slots sit
   pre-chain — k8's windows start ~6452, a 2-kill fight fits the 300-tick caps).

## What was run

- The lever's own real run: the ignored walk, real input from the shuttle arrival, no
  top-ups, no health/ammo/channel writes [L /tmp/lever1_r10.log].
- The re-pin run of the same walk on the final tree [L /tmp/baseline_r10.log].
- `walkreal_0700001b_live` WHOLE on the final tree: **18 passed / 0 failed / 1 ignored**
  (151.97 s) — includes the d1 and d2 real-health walks and replays, byte-identical paths
  (the shipped diff touches the ignore message only).
- `interactables_0700001b_live` WHOLE: **9 passed / 0 failed** (29.09 s).
- The four-crate suite runner (`tools/test/suite.py`) on the final tree: the build phase hit
  this sandbox's disk wall (656 MB free; nf-view's ~25 test binaries need ~4.7 GB of unit
  products — this workspace keeps the rlib tree under `target/release/build`, there is no
  `target/release/deps` to prune) and died with BUILD FAILED on qworm_fire_mode_live /
  probe_facial_ops / fix_06_fire_live / fix_door_react_path_live; the run phase never started
  [L suite.out]. The three crates whose test binaries the build had already produced ran
  green from those artifacts: **nf-game 1233 passed / 0 failed / 7 ignored**; **nf-audio +
  nf-formats 144 passed / 3 failed** — the 3 failures are exactly the allowed slim-ISO
  `disc_golden` trio; nf-view's required binaries green as listed above. Same fit failure as
  rounds 9 and captors-6.

## What's left

The walk still dies at **11002** with k7/k1/k2 unlatched and the wave 6 strong at death. The
next lever is named in the ignore message (the chain-window-fenced thinning, with the
pre-chain thinning fallback). The chains after a mid-phase survival: k7 (nest, target
~10120) -> k1/k2 (x20) against the 12384 read, then the Drake phase (the y > 9 spawn-band
re-weight, the round-6 lever 3).

## Live checks for the main agent

- Whether the k8 UI script's 3-hold chain really restarts after missed holds #1/#2 on a fresh
  boot at 00 Agent (the port's resume-chain model): watch the gate-38 player's holds after
  Bond's screen work is interrupted by a fight mid-chain — if the game re-offers hold #1,
  the walk can recover the chain and the fence can be softer than proposed.
- The mid-phase spawn schedule on the real boot: the observed waves landed ~5100 (astro 1 ->
  4) and ~5814..6114 mid-fight (4 -> 7) — confirm the second wave's script trigger (the
  port's model has waves at f3000/f4200/f5100 only).

## Usage

Context window/cost figures are not visible to me (no token, credit or cost readout is
exposed in this environment; the session ran from ~04:55 to ~05:35 UTC on 2026-11-15).

Main 11-14: not merged - death unchanged (t11002) on master.

Main 11-15 (on 854334126): the 1b bill by 1000 ticks: t4000 17.3 (laser 12.3 + #3), t6000 30.0 (#11 15, #9 10, #8 5), t7000 16.6 (laser), t10000 25.0 (#16 15, #18 10). The space laser is 28.9 of the 100 and its blast is Explode_Propagate linear falloff over radius 20 (12.3 at 3.9 u; ~5 at 15 u). Lever: while a space laser is in state 1 (target locked at charge start) and its target is within ~15 u, steer the float away from the target - needs a shared input hook across the console/kite/fight loops (the walker on_step sees the scene only).

Main 11-15 (on 854334126): an opt-in walker laser dodge (tick_inner pushes the float away from a charging space laser target within 15 u; view/fire untouched): the laser blasts land 13.6-19.0 u away (world 28.9 -> 15.6) but the astronauts land more (#10/#13 25.0 each) and the death comes at t10028 - worse than t11002. Not shipped; the dodge needs to yield to the console/fight inputs when an astronaut is close.
