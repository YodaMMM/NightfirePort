## Your task: `level-14-full-walk-4`, Countdown Part 1's walk past the hood to the exit

A previous session (`s/level-14-full-walk-3`) ended after setup with no change. Start from origin/master (it has `level-14-full-walk-2`: the release onto the east vent lane is green, the walkable gate is up-facing only [M Intersect_CylGeom 0x109da0 0x10a650-0x10a674], the phase-2 hang drift is ported) and read `docs/reports/level-14-full-walk-2.md`. `walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit` is `#[ignore]`d on the leg-1f hood-underside blocker. Clear it from the game's rule [M] (collision, the lane geometry [D]), walk leg 1f (the lane south over the flush lid, the connector west to the #301 drop), the ch 0x47 wait, then legs 2..6 to the exit on real input only. Un-ignore when green end to end; otherwise back to `#[ignore]` with the exact blocker. Commit and push progress early.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-full-walk-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

