## Your task: `mission-stats-rows`, the Mission Stats page numbers and the Records page

On the Double Cross Part 4 results, the Mission Stats page shows numbers that look wrong (e.g. Time 00:08 -> 1,585,205 points; Opponents Dispatched target 45 on a level with one assassin), and the Dossier Records page shows High Score / Bond Moves / Upgrades all 0 for Double Cross after finishing it. (1) For every stats row, port the game's value, target and points exactly [M the stats builders: the PlrStat_* tallies, the per-level target table, the points formula] and pin each with a test from a real 07000004 finish. (2) Records: the game saves the mission's high score / Bond moves / upgrades on mission end [M the records writer]; port it so Records shows the finished mission's values. Run the FULL suite and quote its total line. (The page layout itself is on the s/menu-stats-page branch; build on it - merge it in.)

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mission-stats-rows`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-29T18:34Z

---
## Report (done, 2026-10-29T20:15Z)

