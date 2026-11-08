## Your task: `los-mask-envelope`, every line-of-sight check uses the game's collision mask

Branch from origin/s/markers-world-objects, merge current origin/master, and read its docs/reports/markers-world-objects.md, item 1 of "What's left". The mask bit math is proven, but `LOS_MASKS_ON` stays off because five places where the masks are applied are not ported yet; the report lists them. Port those five in the game's order, with addresses, then turn the gate on. This touches guard cover and turret behaviour, so run every walkreal_*_live one level per tool call and repair any walk it moves, gated to its level, never lowering damage or health. Rand draws must keep the game's count. Pin through real play on two levels: a guard or turret sees, or does not see, Bond exactly where the game's mask says. Run the ai binaries, the turret binaries, glass_pass_every_level_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/los-mask-envelope`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-08T21:52Z
