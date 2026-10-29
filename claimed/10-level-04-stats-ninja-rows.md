## Your task: `level-04-stats-check-rows`, Double Cross Part 4: every Mission Stats row from a real finish

Read the mission-stats-rows report (on master). Pin the whole Mission Stats page of a real-input 07000004 finish (kill the assassin on real input, the outro, Results, the Stats page) cell by cell against the game's formulas [M PlrStat_GetScore 0x1984b0, the base-map design rows L ee_start_05 @0x2bc080], including Bond Moves, Subdued/Surrendered, Stealth, Accuracy, Health, Time, 007 Bonus, the subtotal, the difficulty bonus and the total - and the Records page after it. Any cell the port shows differently from the game is a port gap: fix it. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-04-stats-check-rows`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-29T20:44Z
