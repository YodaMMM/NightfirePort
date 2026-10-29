## Your task: `level-0c-real-crossings`, Chain Reaction Part 1: the hinge-burn drop and the wire crossings on real input

Read the level-0c-every-objective-blockers report (on master). Every remaining stand-in in `walkreal_0700000c_live` now has its game mechanism: blocker 2 is the hinge-burn route (the four hinge locks #139-142 burned by the laser watch's real fire path, AND #505 -> SS #138 the vent mesh, the drop from the duct fires plate #274 mid-fall); blocker 4 is the wire crossings #284, #282, #283 (jump-grab at the type-3 icons, the hand-over-hand ride, the drop onto the floored ends); blockers 3 and 5 are the descents the report maps. Replace each stand-in with the real-input leg in that order (laser-watch fire, jump grabs, strafe rides, drops), fixing any port gap you hit [M/D]. The walk must stay green. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-real-crossings`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-29T22:21Z
