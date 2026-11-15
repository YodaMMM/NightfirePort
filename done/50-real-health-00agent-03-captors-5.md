## Your task: `real-health-00agent-03-captors-5`, 07000003's 00 Agent full-route real-damage walk goes green

Branch from current origin/master; if origin/s/real-health-00agent-03-captors-4 is not on master, merge it in first. Read docs/reports/real-health-00agent-03-captors-4.md there and the ignore message on walk_the_full_route_and_complete_every_objective_real_damage_00agent. Take the next levers in the ledger: the saddle route without the (22.1,-2.0) anchor, then the #7 aimed stand once armed. Helpers stay gated to the level and the difficulty, so the other walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health. Judge each lever on its own real run. Remove the ignore when green; if still red, push the refreshed ledger and the next lever into the ignore message. Run the whole walkreal_07000003_live binary and interactables_07000003_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-03-captors-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-14T14:00Z

Main 11-14 (master 06db4a156, NF_DMG_TRACE): the t74135 death is a DRY STAND at the east lawn saddle (23.18,5.50,7.20): Bond reaches the saddle with the P2K 3+13 and 41.2 health, the saddle wake sweep burns it, then the stand cycles weapons (0x1 clip 1 spare 0, cycling 0..11) while #7 (0x5000039, 0x12, hp 4) at (11.4,10.1,-16.1), 26.5 u, lands 1.8 a hit for ~40. A loot + resupply pass before the saddle finds nothing (3+13 unchanged). Next: leave the saddle line when dry (the sweep breaks on dry, but the stand inside it keeps cycling), or carry rounds from the servants-ground fights.

Main 11-14 more: the saddle sweep has NO dry check (the comment claims one) - Bond arrives at the saddle with the P2K 3+13 and 41.2, the sweep and the stands spend it, and the walker switch_to_armed then stands ~730 ticks cycling an empty wheel (0x1 clip 1) in #7s line. Measured: a dry stop inside the sweep / skipping the sweep when nothing fires - byte-identical t74135 (the cycling comes from the loot pass after it); return fire off on the walk to the saddle (3+13 kept) t73170; saddle sweep radius 15/20 (skip #7 at 26.5) t74135, with return fire off t72825. Not shipped. #7 at (11.4,10.1,-16.1) needs an answer that is not a 26.5 u pistol duel.

Main 11-14: a positioning shoot_bounded on #7 at the saddle (900) dies inside it at t73864 at (29.49,6.38,18.38). Not shipped.

Main 11-14: crouched at the saddle (one press before the sweep) - t74068, no gain (#7 at 26.5 u still lands). Not shipped.

Main 11-14: an aimed_stand_03 on #7 (900, torso) from the saddle before the sweep dies inside it at t74105 - no line on #7 from the saddle either. #7 at (11.4,10.1,-16.1) needs a different spot entirely.

Main 11-14: a bullet-line scan at the saddle finds #7 (11.39,10.09,-16.13) clear from (14,-4) d12.4, (14..20,0) d16-18, (20,2..4), (18..24,8) - the saddle itself (23.2,7.2) is not clear. An aimed_stand_03 from (20,0) (900/1800, torso or head) leaves #7 at hp 4.3 - NO hit lands, the stand has nothing to fire (the P2K 3+13 arrives nearly dry) - death t78288/t78447 (later than t74135 only because of the walk); (16,0) t74384, (18,0) t74710. 03 at this point is AMMO-bound: rounds for the saddle are the lever.

Main 11-14: pickup scan before the si-1 saddle walk (Bond (11.9, 6.5, -20.9), P2K 3+13): every weapon/ammo pickup within 45 u of the saddle is already gone (0x12/0x1c/0x1e/0x35/0xe drops); live are only the armours (21.5, 9.2, 11.6) and (22.1, 10.2, -17.7), both on upper floors. Rounds for the saddle must come from earlier in the route.

Main 11-14: inventory at the si-1 saddle walk (t71943): owned 0x1, 0x2 (P2K, clip 3 + carried 13), 0xe and 0x1c (shotgun) both at 0 rounds, gadgets 0x4a/0x4e/0x5b; the records give the laser 0x4e/0x4f and stunner 0x4a damage 0 and range 4-8 - no long answer to #7 there. The only rounds left are the P2K 16 (3.5 a hit, range 50). Where the 0xe/0x1c rounds went upstream is the lead.

Main 11-14: the saddle-side armour (21.5, 9.2, 11.6) stands on a y 9.09 floor behind the door (22.3, 5.3, 10.9); the nav to it dies opening the door at t73685 - worse.

---
## Report (done, 2026-11-15T03:07Z)


---
## Report from the worker (done, 2026-11-15T03:07Z)

## Report (real-health-00agent-03-captors-5, 2026-11-15)

Task: continue the 07000003 00 Agent full-route real-damage walk (still ignored). The walk is
NOT green; the ignore message on the branch (commit bbde5ed94) carries this round's full ledger
and the next levers. The walk stays ignored.

### What was done

Branch base: origin/master afec3f768 (the captors-4 code was already merged to master,
b0d18af5d; no captors-4/captors-5 branch existed to merge). The prior captors-5 sandboxes
(queue claimed/*.md) shipped nothing; their measurements were used as leads only.

All runs are real-input d3 walks of `walk_the_full_route_and_complete_every_objective_real_damage_00agent`
(each lever judged on its own real run; no top-ups; damage/health untouched):

- **Run 22 — the -4 ledger item 2 as written (the saddle route without the (22.1,-2.0) anchor):
  RED, byte-identical to run 20.** With the anchor line deleted, the saddle nav from the si-1
  resupply end wedged 22 stuck stops on the lane doors at (11.88,6.46,-20.90), the climb never
  engaged, and the walk died at 80288 at (25.30,7.37,8.79) — run 20's exact tick and spot
  [L d3run22]. Conclusion: the anchor nav's own wedged-bridge ladder is what moves Bond onto
  the route the saddle nav can climb from (run 21: the anchor capped 20x3000, the saddle nav
  then worked and arrived (23.18,5.50,7.20)). The anchor stays in the tree.
- **Run 23 — the arming lever (the -4 ledger item 1) + the (20,3) aimed stand on #7: RED.**
  The lane/moat legs' d3 clears got a loot_the_drops_03 cycle — it found NO visible drop on ANY
  leg (the kills' guns land in the moat ditch, outside the loot helper's ±1.5 y floor band; the
  cycle never walked once). The west-dragging resupply is now gated to `difficulty == 2` (d1/d2
  byte-identical; d3 drops the 25.7 u west walk that returned `resupply done (false) clip 3
  spare 13` [L d3run20]). The stand engaged (#7's exposure confirmed from the lawn band) but
  landed nothing: a temporary tap-gate trace shows the aim CONVERGED (err 0.0000, perr 0.0000,
  aim on #7's boxes, still 1502, aiming true) with the gun already the FISTS (clip 1 spare 0)
  from t0 — the return-fire fights on the (20,3) approach walk drained the 16 rounds before the
  stand. Death 75056 at the saddle doorstep (22.96,5.55,7.28), #7 still hp 12 [L d3run23/24].
- **Run 25 — the cold approach + the #7 aimed stand once armed: the round's win.** The si-1
  approach runs COLD (return_fire off across the anchor nav + the (20,3) walk — the
  crossing-legs doctrine keeps the rounds), the ammo is re-checked AT the stand (>= 5): the
  #7 aimed stand KILLED #7. He is absent from the death list for the first time in the ledger's
  history (runs 20/21: alive at hp 4-10, 41.2 in 43-51 taps), the walk took 35 hits (was 59),
  and the run died at 73614 at (23.61,6.37,10.57) PAST the saddle — the mouth nav's own door
  crossing fired the 0x23 plate mid-leg and the waking trio (#5 0x1c, #4 0x12, #3 0xe, all at
  FULL hp) point-blanked the dry Bond (#5 d 3.3 for 16.8, #4 12.6 in 7, #3 11.8 in 2)
  [L d3run25].
- **Runs 26-28 — the 0x23 trio-wake block (the -4 ledger item 4): the race is real.** Stepping
  onto the 0x23 plate before the mouth nav (the door-8 cluster block's own shape) was a
  byte-identical no-op until the trio-alive gate widened to 12 u — the trio LOADS at
  (26.3-29.4, 16.3-16.4), 8-10.5 u from the plate centre (22.9,6.1,9.8) (kind 1 TOUCH_ONCE;
  the census also shows three 0x23 plates: (22.9,9.8), (38.9,11.1), (25.7,9.4,8.3)) and only
  walks to the doorway after the plate fires [L d3run27]. With the plate stand live, #5's
  charge STILL won the race: he reached d 1.2 at hp 12 and blasted Bond at the plate itself
  (death 73318, 39.4 in 16 hits) while the wake sweep's switch+raise (~330 ticks) never fired a
  stand [L d3run28]. The diagnostic census/trace prints were removed before the push.

All new/changed behaviour is gated to `difficulty >= 3` (+ the si-1 leg where noted) inside
`crates/nf-view/tests/walkreal_07000003_live.rs` — no shared helper
(`crates/nf-view/tests/common/`) was touched, no `crates/*/src/` change at all.

### Verification (what ran, with pass counts)

- walkreal_07000003_live WHOLE on the final tree: **6 passed / 0 failed / 2 ignored**
  (305.29 s; the ignored = the d3 walk + the door-8 probe). This is the byte-identity proof for
  every other stream in the binary (the d1/d2 walks and the replays stayed green).
- interactables_07000003_live WHOLE: **9 passed / 0 failed** (42.28 s).
- The FULL suite did NOT run: the background start dies between tool calls in this sandbox
  (suite.out stayed 0 bytes with no process), so the four-crate build ran in six 570 s
  foreground chunks and was not `Finished` by the 03:09 stop; the suite's run phase never
  started. Twice pruned stale per-unit hash dirs under target/release/build (nf-view grew to
  3.8 G / 1263 dirs on the 9.9 G disk). No test FAILED in any run. Diff scope: only
  walkreal_07000003_live.rs (+ docs), so the two named binaries cover every test using the
  touched code.

### Live checks for the main agent

- #7 (skin 0x5000039, (11.4,10.1,-16.1) upper floor): after the aimed stand kills him, his
  0x12 drop lands at his feet ON the upper floor — the loot helper's ±1.5 y floor band never
  sees it from the lawn. Is the lane building's upper floor climbable on real input from
  (11.88,6.46,-20.90) (the anchor wedge end), and does the 0x12 pickup have rounds?
- The trio wake: #5 (skin 0x500003a, loads (29.36,6.25,16.36)) closes to d 1.2 in ~200-300
  ticks of the 0x23 plate firing. Does the port's guard charge speed match the decompile's
  (DroneFunc walk/run speed at difficulty 3), i.e. is the ~250-tick charge correct, or is the
  port waking them early?

### What's left

- The walk (the refreshed ledger in the ignore message is the hand-off): (1) the trio wake
  race — the arm must precede the plate (the door-8 cluster's guns are behind the trio; #7's
  0x12 drop is on the upper floor [S] unverified); (2) the health margin — the run-25 crossing
  cost ~41 while Bond entered at ~35; the lane-mouth fight (36.0 in 3 hits) is where the margin
  went; (3) the older levers (north of door 8, the (35.6,8.7) door stall, the exit-lawn #16
  volley).

### Usage

not visible to me
