## Your task: `explode-falloff-int`, explosion damage falls off with distance as in the game

The level-14-fusebox-zap report (origin/s/level-14-fusebox-zap, docs/reports/level-14-fusebox-zap.md, "What's left") found that the game's Explode_Propagate falloff truncates the gap to an int before dividing by the radius [M 0x17c518:77-82], while the port's explode::falloff is linear. Confirm it in the disassembly first (the cast, and which value it applies to) and write it down with addresses. If the game truncates, port that law in the game's order and test it red on the old code. Pin it through real play: a guard or Bond at a fractional distance inside a blast takes the game's damage. This moves blast damage on every level, so run every walkreal_*_live with explosions one level per tool call and repair any walk that shifts, gated to its level. Real input only, never lower damage or health. Run the bullet and explosion binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/explode-falloff-int`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-11T13:55Z
