## Your task: `level-13-guards-start-alerted-4`, the full vision-bit census without the seven reds

Start from `origin/s/level-13-guards-start-alerted-3` and read its report (merge origin/master). The full census of ENTER arms is right per the decompile, but the full suite (reproducible alone; green on master) has seven reds with it: walkreal_07000001_live `walk_from_the_spawn_to_the_exit_with_the_contact` (obj0 through the real monitor [M 0x1827d8 case 3]), drake_fight_live `the_missile_launch_sequence_reaches_each_phase_and_disarms` (objective 1 'Disable the missiles' never completes), fix_pickups_ammo_live `drops_and_boxes_take_through_the_real_path_with_the_game_numbers` (the second drop is None, must be ammo), drone_weapon_drop_live `killing_a_six_guard_drops_his_gun_with_the_game_rounds` (the 06 guard never dies under P2K fire), hostage_killer_07000002_live `the_captor_and_hostage_alternate_mission_lines` (the opener is not the captor's line [M 0x144340:34-40]), bathhouse_voices_live and bathhouse_shout_mixer_live (the bathhouse floor guards never shout). With sight now live in more states, guards see/react earlier than these tests' walks expect, or some downstream system (drops, shouts, mission lines) depended on the old blindness. For each, find what the game does at that moment [M] and fix the port, or re-derive a test that leaned on the old behaviour with the cite. All seven green plus everything rounds 1-3 fixed; run the AI/escort/walk/voice/pickup families before pushing.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-13-guards-start-alerted-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-28T00:34Z
