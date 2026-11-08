## Your task: `drone-bump-latch-turn-2`, a bumped guard turns its walk the game's way without breaking the 01 walkers

Branch from origin/s/drone-bump-latch-turn, merge current origin/master, and read its docs/reports/drone-bump-latch-turn.md. The provenance run confirmed the latch arm breaks the 01 followed-walker walk (it passes 2/2 without the arm). Take "What's left" in order: find whether the rotation's frame is mirrored against the game's [M `DroneAnim_PreTransCallback` 0x13b490:98-118] and fix it so both 01 tests pass with the arm in. Then the second pin, through the hall groups the report names or its live check 2. Then the three documented PreTransCallback arms: the +0x38 zero with its setters, the speed ramp and the 0x1e difficulty scale. Rand draws must keep the game's count. Run the ai binaries, guard_ai_*_live, walkreal_07000001_live, walkreal_07000009_live, every other walkreal_*_live one level per tool call if guard motion changes, and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/drone-bump-latch-turn-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-08T17:59Z
