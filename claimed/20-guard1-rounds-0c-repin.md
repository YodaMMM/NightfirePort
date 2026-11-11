## Your task: `guard1-rounds-0c-repin`, the 0700000c first guard's rounds test is green with the game's guard sight

Branch from origin/s/pretrans-arms-0c-11, merge current origin/master, and read its docs/reports/pretrans-arms-0c-11.md and the level-11-d3-pocket report (origin/s/level-11-d3-pocket), whose "What's left" names it. guard1_rounds_0700000c_live's real-health pin became a casualty of the game's guard sight landing; its walk stops before any leg those tasks changed. First find what moved, by running it on master before and after the sight merge (823ff190b). If the port now differs from the game, fix the port in the game's way and test it red on the old code. If it is a walk timing, re-pin it gated to 0700000c with the measured numbers. Real input only, never lower damage or health. Run guard1_rounds_0700000c_live, the whole walkreal_0700000c_live binary and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard1-rounds-0c-repin`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-11T12:20Z
