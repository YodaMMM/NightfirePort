## Your task: `drone-bump-collision-2`, the guard push in every stance without breaking the escort and party routes

Start from `origin/s/drone-bump-collision` (merge origin/master) and read its report: the player-side push from armed drone capsules now runs in every stance [M Player_Collision 0x194df0, Collide_Jointy 0x1ec180:229-266] with `Drone::bump_capsule` per NDrone2_Collision 0x140e90:59-124. Alone on master it turns these red: mayhew_hold_live `the_escort_holds_at_the_ally_wait_node_until_the_ambush_clears`, party_door_lock_07000007_live (both tests), walkreal_07000007_live `walk_the_party_start_to_finish`, probe_07_route `probe_the_route`, cutscene_timer_pause_live `the_control_scene_freezes_the_mission_clock_until_the_hand_back`, walkreal_07000011_live `walk_spawn_to_the_lower_floor_load_point`. Find which push the game does not do there - the pick-level gate (which drones are armed for the player's class-3 query: +0x464 = 0x800 under NDrone2_DoCollision 0x149ec8:11-34, the 0x20/0x200 flag arms, NearPlayerCollision 0x14b190), allies/escorts/civilians in scripted states, the radius per state - and port exactly that. All seven green plus drone_bump_live; full suite green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/drone-bump-collision-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-27T23:49Z
