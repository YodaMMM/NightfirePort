## Your task: `level-0c-every-objective-blockers`, Chain Reaction Part 1: blockers 2-5

The 0c walk (on master) is green through the turbine door and the band crossing (read the level-0c-door2turbine-3 report). Blockers 2-5 of the level-0c-every-objective report were never started: (2) plate #274's cell inside the display case (does the glass body-block it in the game, or does the plate fire another way), (3) the display ground -> warehouse/yard descent, (4) the ring's west/south gaps (the south one is the #283 wire ride onto the west ledge), (5) the doorway-ledge -> yard descent. Take them in order, each from the game's behaviour [M/D], fix any port gap, and replace each stand-in in `walkreal_0700000c_live` with the real-input leg. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-every-objective-blockers`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-10-29T20:01Z
