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
