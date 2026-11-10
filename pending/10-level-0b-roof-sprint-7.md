## Your task: `level-0b-roof-sprint-7`, the 0700000b roof sprint can be walked on 00 Agent with real health

Branch from origin/s/los-mask-repairs once it is pushed (the guard sight masks: guards see across the roof parapet, live-checked in PCSX2); if it is not on origin yet, branch from origin/s/level-0b-roof-sprint-6 and merge origin/master. Read docs/reports/level-0b-roof-sprint-6.md and the ignore message on walkreal_0700000b_live::walk_fresh_spawn_to_the_mission_end_real_damage_00agent: Bond dies at (7.13,58.07,41.22) tick 16517, 22 hits for 100.0, 2.7 u short of the deck edge. Sprint-6 proved the moving-accuracy term and the blast radius are right and measured the carried rocket lever as dead until the 0x2e flight arc is modelled. Next levers: take the long-range #9/#6 pair from cover (they now see across the parapet, so use the parapet's own blocking corners), restock before the close, and request the weapon switch back after any rocket shot. Real input only, no top-ups, never lower damage or health; helpers gated to 0700000b and difficulty >= 3. Remove the ignore when green. Run walkreal_0700000b_live and interactables_0700000b_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-0b-roof-sprint-7`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
