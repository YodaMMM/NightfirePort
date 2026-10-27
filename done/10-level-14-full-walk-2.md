## Your task: `level-14-full-walk-2`, finish Countdown Part 1's start-to-exit walk

Start from `origin/s/level-14-full-walk` and read its report (merge origin/master). Leg 1a now walks the east route green through the Q-Claw shaft entry (the #626 reel to the hang at (45.37, 4.59, 43.91) over the east vent lane); the release onto the lane is the one red step (Bond stayed on the wire at (45.37, 4.22, 49.09)). Reconcile the release with the game's rule [M Player_SSGrapple 0x192860 phase 2: the first no-fire tick detaches] by printing the grapple state through the release loop, land on the lane, walk leg 1f (the lane south over the flush lid, the connector west to the #301 drop), then legs 2..6 to the exit on real input only. The test stays green end to end, or goes back to `#[ignore]` with the exact blocker; never leave it un-ignored and red. Every other walkreal test stays green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-full-walk-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-27T07:36Z

---
## Report (done, 2026-10-27T08:42Z)

Closed by the driver: the report is on `s/level-14-full-walk-2` (docs/reports/level-14-full-walk-2.md); the worker didn't run finish.sh.
