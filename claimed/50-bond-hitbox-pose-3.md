## Your task: `bond-hitbox-pose-3`, Bond's crouched hit boxes without breaking the duct walks

`origin/s/bond-hitbox-pose-2` merged with master turns these RED (green on master and on master + walk-move-clip): walkreal_07000014_live::the_east_duct_route_reaches_the_stairwell_slot, ::the_vent_drop_hands_bond_into_the_facility, ::walk_the_infiltration_to_the_delta_exit, walkreal_07000004_live::the_results_page_opens_stats_and_dossier_and_scores_the_real_tallies. Continue from -2 (merge `origin/master` first). The 14 reds are crouch/duct legs: find what -2 changes on them (the posed boxes, the crouch root offset, the bullet walk vs the boxes, the load path) and what the game does there [M/L]. The 04 red is the results tallies (hits/accuracy): find which tally moved and why. Fix the port where it is wrong; re-derive a test only with evidence. Run the FULL suite on the final tree and quote its total line - no new reds.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/bond-hitbox-pose-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-30T02:13Z
