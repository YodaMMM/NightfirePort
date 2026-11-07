## Your task: `smash-stat-gate-02`, the named smashable on 07000002 counts in Bond's accuracy as the game's does

`Destroy_Smash` 0x12bb60:42-55 calls `PlrStat_LogShotHitScenery(0)` on 0x07000002 when a switch-channeled smashable is destroyed and its obj+0xd8 equals `hashtable_getitem(0x20001d9)`. The port has no such arm: the level-gates-world report in the queue's done/30-level-gates-world.md records it as open, because destroy leaves carry no object-name hash. Read `PlrStat_LogShotHitScenery` 0x1998d8 and its port in crates/nf-game/src/progress.rs to see what the call does to the mission's accuracy figure. Find which placed object has that name on 07000002 [D], then carry just enough identity to the destroy path to match it, without a level-wide name table if one field will do. The arm runs only on that level, only on that object, and only after the channel write in the same block. Pin through the real path: Bond shoots the object on 07000002 and the Accuracy figure on the results data matches the game's arithmetic, and the same break on another smashable changes nothing. Run walkreal_07000002_live, the stats binaries (grep tests for the progress.rs function) and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/smash-stat-gate-02`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-07T02:48Z
