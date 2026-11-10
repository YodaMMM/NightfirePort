## Your task: `real-health-00agent-04`, Double Cross part 4 can be walked start to finish on 00 Agent with real health

Branch from origin/master. walkreal_07000004_live::walk_the_full_mission_start_to_finish_real_damage_00agent is ignored: the assassin fires at accuracy byte 0 [M 0x14b300:752] and the walker's P2K stand loses the trade by a few rounds (Bond dies at tick 1627, 32 hits for 100.0, with her at 24); the note says the stand needs cover or the armour first. Take those levers: pick up the level's armour before the fight [D its placement] and fight from cover the level offers, with real input. Never lower damage or health, no top-ups; helpers gated to 07000004 and difficulty >= 3. Remove the ignore when green, otherwise push the ledger. Run walkreal_07000004_live and interactables_07000004_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/50-real-health-00agent-04`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
