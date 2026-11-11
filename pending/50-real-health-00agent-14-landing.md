## Your task: `real-health-00agent-14-landing`, Countdown's 00 Agent infiltration walk passes the 1.72 landing

Branch from origin/master. walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit_real_damage_00agent wedges (not a death): the '1.72 landing' resupply leg floats Bond 0.41 over the floor at (33.84, 0.82, 30.95) at tick 26340 and the leg cap panics. A floating Bond is a collision question first: find why the port holds him 0.41 above that floor [M the player floor snap] and fix it if the port is wrong (test red on the old code); otherwise take the ignore's own lever (a re-ground before the leg). Real input only, never lower damage or health. Coordinate with 50-real-health-14-00agent-10 if it is claimed. Run walkreal_07000014_live and interactables_07000014_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/50-real-health-00agent-14-landing`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
