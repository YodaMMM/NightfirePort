## Your task: `level-0b-every-objective-2`, 0700000b walked end to end - the Q-Worm install and the lift

Continue `origin/s/level-0b-every-objective` (merge `origin/master`). Its rewritten walkthrough fails at the Q-Worm install leg ("the Q-Worm C install never landed" [M 0x1aee20]) though the pre-shot state matches the merged version - bisect the legs before the shot (the walker's stuck-mitigation side effects in the wedged terminal/lock walks, the exit-request state vs the bullet pass) and fix the real cause. Then the lift boarding (cab_y never matched hash 2000367 - pair the entity id/hash with one probe) so the balcony -> exec floor transit is walked, and the walked lock and terminal approaches past their pinned furniture blockers [D probes] - each from the game's behaviour [M/D]. Real input only. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0b-every-objective-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

