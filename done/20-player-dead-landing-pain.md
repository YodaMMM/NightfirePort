## Your task: `player-dead-landing-pain`, a dead Bond's landing follows the game's pain rule

The player-ceiling-tails report (origin/s/player-ceiling-tails, docs/reports/player-ceiling-tails.md) left the dead landing's pain: the game calls `Player_HandlePain` on a dead body [M 0x1949a8:120], and the port sends no fall damage from the dead tick. Read `Player_Damage`'s dead gate and `Player_HandlePain` in full, with addresses, and settle what a dead Bond's landing does in the game (any health change, flash, sound or direction bits), from the disc and the decompile. If something visible or audible happens, port it in the game's order; if the gate drops it, say so with the address and leave the port as it is. Pin it through real input on a level where Bond can die mid-fall. Run the player binaries (grep tests for 0x1949a8), walkreal_07000005_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/player-dead-landing-pain`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-09T09:49Z

---
## Report (done, 2026-11-09T10:26Z)

