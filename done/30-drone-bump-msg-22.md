## Your task: `drone-bump-msg-22`, a guard hit by a thrown or pushed object reacts as in the game

The control-funcs-collide audit found that the port models the physical push of a body against a guard, but never sends the guard msg 0x22 on contact [M `NDrone2_DealWithObjHit` 0x13fff0:108-126; docs/fields/drone-statemachine.md:167 lists 0x22 as hit by a thrown or picked object, with 2 DSTATEs]. Read every sender of 0x22 in the decompile and what the two states do, with addresses, and find which objects in single player can hit a guard this way (Bond's bump, thrown items, pushed props). Wire the message from the port's contact path in the game's order, with the +0x26 latch the port already documents. Rand draws must keep the game's count. Pin through real play on two levels where it happens: the guard enters the game's state and comes out of it as the game's does. Run the ai binaries, guard_ai_*_live, walkreal_07000001_live, walkreal_07000009_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/drone-bump-msg-22`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-08T07:16Z

---
## Report (done, 2026-11-08T08:36Z)

