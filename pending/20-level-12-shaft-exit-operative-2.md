## Your task: `level-12-shaft-exit-operative-2`, Phoenix Base reaches the shaft exit on Operative with real damage

Branch from origin/s/los-mask-repins-12-2 and merge origin/master; read docs/reports/los-mask-repins-12-2.md on that branch. That branch made walkreal_07000012_live::walk_spawn_to_the_shaft_exit_real_damage_operative green (the walker's left nudge never decayed: sidestep -40 grew forever, so the crossing leg past door plc 543 never walked) but it was cut on 11-10 and now conflicts with master. Resolve the merge keeping master's newer walker code (check whether the sidestep decay fix already reached master via the walker-sidestep-decay work before re-adding it), keep the Operative walk green and un-ignored, and keep the d2/d3 07000012 walks and the 07000013 walks as they are on master. Real input only, never lower damage or health, no top-ups. Run walkreal_07000012_live, walkreal_07000013_live and interactables_07000012_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/20-level-12-shaft-exit-operative-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
