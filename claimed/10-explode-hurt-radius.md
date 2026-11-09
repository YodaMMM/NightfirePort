## Your task: `explode-hurt-radius`, explosions hurt only as far as the game's blast radius

The level-0b-roof-sprint-4 report (origin/s/level-0b-roof-sprint-4, docs/reports/level-0b-roof-sprint-4.md, item 2 of "What's left") measured the port's explosion hurting Bond at about 13.7 u where the disc's radius for that weapon row (0x2e) is 8.0 [L both sides in the report]. Read the level-08-rocket-self-blast report (origin/s/level-08-rocket-self-blast) first so you do not redo its findings. Then find the port's hurt reader for `Explode_Create(weapon+0x08 radius, weapon+0x0C damage)` [M near 0x12df3c, bullet.rs:52-54's comment] and compare its distance gate and falloff with the game's `Explode_Update` and the hurt it calls, with addresses, for Bond, guards and objects. Fix any difference in the game's order. Pin it with a test on the 0x2e row (hurt at 7.9 u, none at 8.1 u) and through real play on 0700000b. This shifts every walk with explosions (rockets, satchels, grenades, barrels), so run every walkreal_*_live one level per tool call and repair any walk that shifts, gated to its level. Real input only, never lower damage or health. Run the bullet and explosion binaries (grep tests for Explode_) and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/explode-hurt-radius`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-09T23:42Z
