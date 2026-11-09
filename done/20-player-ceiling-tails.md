## Your task: `player-ceiling-tails`, Bond's body stops at ceilings and handles death as the game's player code does

The control-funcs-collide audit (origin/s/control-funcs-collide, docs/reports/control-funcs-collide.md) lists two unmodelled tails of the player's collision handler 0x1949a8: the dead-state arm (:33-74) and the ceiling arm (:168-175). Read both in full with what they write and when they run. Find where the port handles the same cases today (a ceiling met during a jump, a climb or a lift, and contacts after Bond's death) and say what differs. Port both tails in the game's order, keeping the Rand count. Find places on the on-foot levels where Bond can meet a ceiling [D], and pin through real input on two of them: Bond's head stops at the ceiling the way the game's code says, and a dead Bond's body behaves as the dead arm says. Run the player and collision binaries (grep tests for 0x1949a8), every walkreal_*_live one level per tool call, and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/player-ceiling-tails`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-09T01:10Z

---
## Report (done, 2026-11-09T02:26Z)

