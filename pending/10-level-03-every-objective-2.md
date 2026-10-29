## Your task: `level-03-every-objective-2`, finish the 07000003 whole-mission walk

Continue `origin/s/level-03-every-objective` (merge `origin/master` first; read its report). Its rewritten `walk_the_full_route_and_complete_every_objective` is RED: (1) the Dragon Safe case E is edge-triggered - converge the view first, then a single use tick (the proven shape in `doublecross_items_live`), then the take; (2) the #389 0x21 plate approach routes into the moat / onto the ledge - find a real-input approach the player uses; (3) then the 0x52 / obj2 / no-fail asserts and the exit leg. Fix the stale "07000003 ch 0x4e -> NONE GAP" line in docs/systems/levels.md. Every assert cites [M]/[D]/[L]. The test must be green; run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-03-every-objective-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

