## Your task: `level-13-guards-start-alerted-8`, the vision-bit census with Countdown Part 1's last leg green

Continue `origin/s/level-13-guards-start-alerted-7` (merge `origin/master`). The escort and Rook reds are fixed there; one red remains: walkreal_07000014_live `walk_the_infiltration_to_the_delta_exit` wedges at leg 6 ("to the Delta exit #944") at (23.31, 1.14, -24.64), target (27.79, 1.28, -25.32), segment_to_target Some(0.1227), cel 16, no bot within 3.0 - world geometry the RNG-shifted route now steers into (master walks it green). Diagnose the geometry [D cel 16's faces at that spot] and the route difference, then fix the port if a port collision rule is wrong [M Player_Collision], or give leg 6 the player's real approach with the cite. Run the FULL suite and quote its total line; Double Cross Part 1 and Countdown Part 1 must stay finishable.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-13-guards-start-alerted-8`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

