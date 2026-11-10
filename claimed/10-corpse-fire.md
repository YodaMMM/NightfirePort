## Your task: `corpse-fire`, dead guards never shoot Bond

The level-0b-roof-sprint-5 report (origin/s/level-0b-roof-sprint-5, docs/reports/level-0b-roof-sprint-5.md, its merge-resolution check) found that on current master walkreal_07000011_live fails 2 of 4: the d1 and 00 Agent real-damage walks die to fire from dead guards (hp -72 and -75), since master's change that made guards and corpses solid to Bond (0a9d39435). Reproduce it on master through real input first. Then read which states the game lets a guard fire in, and how a dead guard (state 0x47 and the death states) leaves the firing and aiming paths, with addresses. Find why a dead guard's shots reach Bond in the port and fix it in the game's order, keeping the Rand count. Test it red on the old code. Then run every walkreal_*_live one level per tool call, because corpses exist on every level, and repair any walk that shifts, gated to its level. Real input only, never lower damage or health. Run the ai and bullet binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/corpse-fire`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-10T04:24Z
