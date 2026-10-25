## Your task: `level-0a-rain-indoors-2`, land the rain fall speed without changing other levels

Start from `origin/s/level-0a-rain-indoors` (merge it first; the fix adds `Drops.fall` = DAT_003194f4 = -0.5 for rain [M InitDrops 0x18bf68, UpdateDrops 0x18c580]). Merged with origin/master it turns these red (each green on master):
- sight_frustum_live (2 tests: "guard placement 261 missing or not present")
- interactables_0700000a_live::armour_pickup_taken_by_walking_over_it
- hostage_killer_07000002_live::the_captor_and_hostage_conversations_loop_on_both_pairs
- fix_pickups_ammo_live::drops_and_boxes_take_through_the_real_path_with_the_game_numbers
- doublecross_transitions_live::castle_d_intro_enables_the_assassin_and_the_kill_ends_the_mission_to_the_outro
- escort_hide_findrun_live::the_hides_exit_reanchors_the_mission_route_before_the_lead_resumes
- escort_follow_07000001_live::escort_at_walking_speed_keeps_the_games_distance_and_the_exit_opens

1. Find why a rain-step change alters placements and AI on other levels (the likely suspect: the drops consume the shared random stream differently, shifting every later Rand() draw — check which RNG `Drops` uses and whether the game's UpdateDrops/AddDrop use the same generator as the AI [M]). Make the port draw from the generator the game uses, in the game's order.
2. All of the above green with rain falling at the game speed; fix_0a_rain_indoors_live green; full suite on the final tree merged with origin/master, report the total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0a-rain-indoors-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
