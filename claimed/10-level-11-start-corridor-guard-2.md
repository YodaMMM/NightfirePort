## Your task: `level-11-start-corridor-guard-2`, 11: the start corridor stays empty

Continue `origin/s/level-11-start-corridor-guard` (merge `origin/master` first; read `docs/reports/level-11-start-corridor-guard.md` on that branch). It fixed the run-to-point owner read (Drone+0x144 = placement param 4 [M DefaultInit 0x14b300:472]) but the pin is not met: on a fresh 07000011 load with Bond idle at spawn #146, #127 (d >= 2) / #126 still walk into the corridor, resume ATTACK at alertness 1.0 (the Tower clause [M 0x168910:23-31]) and kill Bond by ~tick 700, while the PS2 corridor stays empty over f200-f3172 [L]. Find the remaining divergence (what keeps the game's guards at their run-to points or out of sight of the spawn: the point's own state/latch, the 0x4000000 latch, LOS from those points, the difficulty the capture ran at [check GameState+0x40 assumptions]), fix it with the address cited, replace the branch's "EXPERIMENT" comments with plain cited comments, and pin: idle Bond at the start is not approached in 3200 ticks (sister: both guards present). Run every *07000011* binary and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-11-start-corridor-guard-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-01T05:28Z
