## Your task: `level-11-stairwell-goal-stall`, the 07000011 stairwell guard walks to his goal instead of standing still

On the real walk walkreal_07000011_live::walk_spawn_to_the_lower_floor_load_point_real_damage_agent, drone #13 (one of the stairwell guards that wake on the flight plates) sits in GO_TO_GOAL (0x63), anim state 0x1e, at (22.13, 44.14, -3.44) on the flights with its goal at (13.48, 46.86, -6.35), route_blocked false, and does not move for 3+ s (frames ~5280-5640 of that walk) while Bond walks up to it (distance 5.4 -> 1.2). A guard that wants to travel, has an open route and stands still looks like it is walking into an invisible wall on the stairs.

Find why the port's GoToGoal does not move him there (what anim 0x1e is, which arm of the GoToGoalPosition / nav follow keeps him in place - a collision push-back, a step-height refusal on the flight, a facing/turn gate), and trace the SAME guard in PCSX2 (tools/trace/goto_level.py 07000011, trigger the stairwell flight plates, log Drone+0x10C / obj+0x30 each 10 frames) to see whether the game's guard walks the flight to that goal. Fix the port against the decompile and pin it (the guard reaches within 1 u of its goal, or matches the game's measured behaviour).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-11-stairwell-goal-stall`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
