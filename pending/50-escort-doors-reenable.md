## Your task: `escort-doors-reenable`, the 01 escort's interior-doors test, re-enabled

`fix_npc_doors_live::the_escort_opens_the_interior_doors_on_his_route` is ignored on master ("walker resupply deadlocks door #2 with the escort's DSTATE_OpenDoor (harness, not the port)"). `origin/s/escort-door-harness` (not merged) showed it passes un-ignored on master alone — but with the other changes merged since (the FindOpponent alive filter and LOS budget, the stuck-guard side-step probe along the guard's yaw), it fails at line 249.

1. Un-ignore it on current master and run it; if it fails, find which change moves the escort or the walker and whether the port or the test is wrong [M], fix that, and keep it un-ignored. Report the line and the cause.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/escort-doors-reenable`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


