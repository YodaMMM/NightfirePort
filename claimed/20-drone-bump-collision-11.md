## Your task: `drone-bump-collision-11`, land the bot-capsule arming without breaking the other levels

`origin/s/drone-bump-collision-10` (its report is on the branch) merged with master makes these tests RED, reproducibly (all green on master without it): walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit, walkreal_07000002_live::walk_the_castle_all_objectives_to_the_exit, escort_follow_07000001_live::escort_at_walking_speed_keeps_the_games_distance_and_the_exit_opens, fix_party_mission_live::the_07000008_safe_chain_works_in_play, playtest_spots_live::spots_07000008_safe_bolts_in_reach, interactables_0700000a_live::armour_pickup_taken_by_walking_over_it, fix_drone3_live::the_truck_driver_reaches_his_idle_state_and_never_twists, pickups08_live (4 tests), fix_safe08_live::the_safe_pickups_take_after_the_safe_opens, speech_facial_fallback_live::a_present_script_talker_plays_through_the_facial_and_advances, fix_06_driver_live::the_scared_cower_keeps_him_in_place_on_the_floor, rook_door_06_walk_live::driver_walks_his_mission_route_after_the_sit_script. Continue from -10 (merge `origin/master` first). For each red, find which of -10's changes moves it (the route-walk boundary clamp replacing probe_move, the push in every stance, the kind-word arming, the door-zone opponent gate) and what the game does at that step [M/D/L]. A test calibrated on old port behaviour is re-derived only with evidence; a real regression is fixed in the port. Run the FULL suite on the final tree and quote its total line - it must have no new reds.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/drone-bump-collision-11`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-29T12:46Z
