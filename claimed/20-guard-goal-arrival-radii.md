## Your task: `guard-goal-arrival-radii`, guards stop at each goal's own arrival radius

Read docs/reports/level-12-engage-range.md on origin/s/level-12-engage-range and merge that branch first if it is not on master. The game's `MoveToGoalPosition` 0x1517d8 arrives under route+0x8C = Drone+0x8EC, which `SetupRouteToPosition` rewrites from each goal's radius argument; the port uses a flat 2.0. That session tried the fix and reverted it, because the true radii (mission 1.0, alarm points below 2.0) bring guards closer and 07000001's two real-damage walks then lost Bond. The report has the patch recipe and the per-goal census. Census `AiPoint.accuracy` (+0x14) per level from the placements [D], so the report shows which levels move. Land form 1 (goal-path radii only) first, then form 2 only if the decompile needs it. Where a walk goes red, the walk is what changes: re-route it or change its fights, as the game's guards now stand closer. Never lower damage or health, never keep the 2.0 for a test's sake, and keep helper changes behind a per-level opt-in. Run every walkreal_*_live, the ai binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-goal-arrival-radii`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-05T18:02Z
