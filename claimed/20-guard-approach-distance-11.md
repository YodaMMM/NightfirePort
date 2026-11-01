## Your task: `guard-approach-distance-11`, 11: #126 stops 10.7 u from Bond where the game's stops 8.8 u

Live PCSX2 (docs/live/2026-11-01-07000011-start-corridor-guards.md on origin/master): at difficulty 1 the game's #126 is 8.8 u from an idle Bond at f200; the port's stops at ~10.7 u at t200 and only closes by t400 (the pin allows 11). Find the game's run-to-point / approach stop rule for this guard [M EnemyRunToPoint 0x168910, NDrone2_FindRunToPoint 0x153168 / ClosestAIPoint 0x151b18, the arrival radius and the attack-range handoff], fix the port gap with the address cited, and tighten the pin to the live distance (sister: the old stop fails it). Run every *07000011* binary, ai_live and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-approach-distance-11`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-01T17:44Z
