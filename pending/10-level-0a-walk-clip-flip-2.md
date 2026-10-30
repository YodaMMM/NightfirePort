## Your task: `level-0a-walk-clip-flip-2`, finish the move-clip flip

Continue `origin/s/level-0a-walk-clip-flip` (merge `origin/master` first; read its report). Under NF_WALK_CLIP=1 walkreal_0700000a_live is 6/1/2: the pick-ref accumulator and the office-floor keyboard legs are green; the qworm ring (the alarm leg) is still red. Fix that leg (port or test, with cites), then do the flip: `walk_clip()` -> true, remove NF_WALK_CLIP and the dead push-walk branch. Run EVERY walkreal_* binary plus nf-game's collision/player tests; quote every total and list any walk whose route changed (leg + position delta).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-walk-clip-flip-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

