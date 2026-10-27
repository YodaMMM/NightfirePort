## Your task: `level-13-guards-start-alerted-3`, the vision-bit gate with every sight-capable state

Start from `origin/s/level-13-guards-start-alerted-2` and read both reports (merge origin/master). Round 2 added the vision bit to PlayScript and the abseil ENTERs, which fixed the first eight reds, but the full suite (reproducible alone, and green on master without this branch) still has six: escort_follow_07000001_live `escort_at_walking_speed_keeps_the_games_distance_and_the_exit_opens`, walkreal_07000001_live `walk_from_the_spawn_to_the_exit_with_the_contact`, fix_npc_doors_live `the_escort_opens_the_interior_doors_on_his_route`, walkreal_07000008_live `walk_the_gondola_to_the_briefcase_and_back`, fix_pickups_ammo_live `drops_and_boxes_take_through_the_real_path_with_the_game_numbers`, drone_weapon_drop_live `killing_a_six_guard_drops_his_gun_with_the_game_rounds`. Round 2's own census says 139 state functions set `+0x4F8 |= 0x10000` [M]; port the bit on EVERY one of those ENTERs (ally/escort states, combat, cover, death/drop paths), not just the ones a red test exposes, so the gate matches the game. Get all six green plus the earlier eight and the 13 start test; run the whole AI/escort/walk family before pushing.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-13-guards-start-alerted-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

