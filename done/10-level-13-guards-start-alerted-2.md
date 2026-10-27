## Your task: `level-13-guards-start-alerted-2`, the vision-bit gate without breaking Double Cross

Start from `origin/s/level-13-guards-start-alerted` and read its report (merge origin/master). Its two AI fixes (the sight pipeline only with the vision bit +0x4F8 & 0x10000, no alertness bypass [M DroneVision_ProcessDroneSight 0x176a28]; the EnemyRunToPoint Tower resume keeps Attack [M 0x168910:22-36]) turned eight tests red on the full suite: exit_aim_01_live `the_exit_resolves_to_the_gate_plate_and_e_is_refused_there` (Bond ends at (-13.49, 6.46, 10.40) instead of the plate), interact_07000001_live `the_armour_pickup_sets_armour_and_writes_channel_13_and_the_sister_refuses` and `the_e_key_opens_the_entry_sliding_door_and_the_sister_keeps_it_shut` (the door opens with no E press), fix_npc_doors_live `the_escort_opens_the_interior_doors_on_his_route` (escort settles at (-4.6, 6.3, 14.1)), walkreal_07000001_live `walk_from_the_spawn_to_the_exit_with_the_contact`, fix_mission_live `the_07000001_exit_works_end_to_end`, guard_first_shot_01_live `a_starting_guard_finishes_his_start_script_before_any_shot`, rappel_vanish_0700004a_live `a_shot_rope_guard_falls_instead_of_vanishing`. They fail within seconds, so the change alters something at level start on 01 and 4a. Find which part does it and what the game does there [M] (which states set the vision bit on 01's starting guards and the escort; whether a test leaned on the bypass), fix the port or re-derive a wrong test with the cite, and get all eight green plus the 13 start test.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-13-guards-start-alerted-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-27T17:30Z

---
## Report (done, 2026-10-27T18:56Z)

