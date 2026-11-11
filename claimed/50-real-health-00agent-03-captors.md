## Your task: `real-health-00agent-03-captors`, Double Cross part 3 can be walked with every objective on 00 Agent with real health

Branch from origin/master. walkreal_07000003_live::walk_the_full_route_and_complete_every_objective_real_damage_00agent is ignored: the pre-clear rally kills the door-8 guards (#18/#19/#2) but its approach crosses their gun line (40.8 to 15.6-19.2 health) and the exit lawn needs ~29. Read the full ignore message. Take its levers: an approach that stays north of door 8 (fight through the open doorway from (36.2, 5.7)), then the post-rescue west-lawn door stall at (35.6, 8.7). Real input only, no top-ups, never lower damage or health; helpers gated to 07000003 and difficulty >= 3. Remove the ignore when green, otherwise push the ledger. Run walkreal_07000003_live and interactables_07000003_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/50-real-health-00agent-03-captors`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-11-11T01:28Z
