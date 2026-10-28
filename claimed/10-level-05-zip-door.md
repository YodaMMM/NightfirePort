## Your task: `level-05-zip-door`, The Exchange Part 1's cable route reaches the zip door

`walkreal_07000005_live` now walks the cable landing but pins a blocker before the zip door #112 (read the negative test and the level-05-every-objective report). Find how the game's player gets from the cable landing to the zip door [D the walkway/door/trigger placements and collision there; M the door's use/approach path and any scripted carry] and port the missing link, then turn the pinned negative into the positive leg on real input through the zip line. Also: the whole-mission chain 05 -> 06 -> 07 is blocked at 07000006's OPEN LEG (the upper gallery -> dressing-room LoadPoint #746 entry has no walked route; ai point 5 patrols a 9-node path above the tunnel) - find the game's route there [D nav, placements] and walk it. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-05-zip-door`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-28T20:03Z
