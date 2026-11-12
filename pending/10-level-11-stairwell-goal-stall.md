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

---
## Main agent addendum (2026-11-12)
anim state 0x1e is `anim_state::RUN`: in GO_TO_GOAL the port's `move_to_goal` returns 0..=2 (moving) every tick and the guard plays the RUN cycle, yet his position holds at (22.13, 44.14, -3.44) for 3+ s - he runs in place against something on the flight (the user-visible "walking into an invisible wall"). So the nav says go and the body's move is refused: log the drone's per-tick move request vs the applied displacement and the collision contact that cancels it (the step-up on the flight's tread, a stair-rail/railing brush, the push-out against the stair's side wall, or another drone's capsule), and compare with the game's guard on the same flight.
The stall's geometry (port, the Agent real walk, frames 5250-5640): #13's route is index 1 of 4 nodes, creeping along the segment a (23.02, 43.72, -5.39) -> b (20.70, 43.72, -5.25) (the landing, y 43.72), creep step frozen at 3/5 with target (21.82, 43.72, -5.32), remaining 10.87. The guard stands at (22.13, 44.14, -3.44) - 0.4 u HIGHER and ~1.9 u off the segment in +z (up on the flight / its side), yaw 3.31, desired -2.97 - so the creep steers him through whatever separates the flight from the landing at that spot and the move is refused every tick. Questions: how did he get 1.9 u off his route line (the route was built from a node he was not near? a push-out onto the flight?), and does the game's creep re-target or give up there (Creep's arrival / NDrone2_MoveToGoalPosition's blocked arm, the +0x8EC radius)? Compare the same guard's route in PCSX2 (route+0x... for drone #13 after the flight plates).
