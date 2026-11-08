## Your task: `level-07-actor-stuck-doorway-2`, the 07000007 actor walks through the doorway instead of freezing in it

Branch from origin/s/level-07-actor-stuck-doorway, merge current origin/master, and read its docs/reports/level-07-actor-stuck-doorway.md. Take "What's left" in order. Narrow the body-side freeze (the CivilianMission move gate at the jam pose) with a trace and pin it. Port the LinkCreep hit_door skip into `Route::follow`: the blocked test's increment runs while the binding stands [M 0x15a2c8:67-69]. Check how the dispatch arms for a door that is already open, the wake-into-an-open-doorway path. Land the fix with a test through the same real-input path that is red on the old code (upgrade the probe to assertions). Run every walkreal_*_live, npc_doors_live, fix_guard_misc_live, the route and door binaries (grep tests for `Route::follow` and `hit_door`) and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-07-actor-stuck-doorway-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-08T03:28Z
