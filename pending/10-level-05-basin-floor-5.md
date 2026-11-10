## Your task: `level-05-basin-floor-5`, the solid hurt planes keep every level's hurt volumes working

Branch from origin/s/level-05-basin-floor-4 and merge current origin/master. Round 4 fixed level 05 (no marker draw, damage through the object hit [M Player_DealWithObjHit 0x194458:94-103, Player_CollisionHandler 0x1949a8:139-145]) but a full suite is 4883 / 9 failed; master passes all of these. Read every message:
- interactables_0700001b_live::the_missile_flame_hurts_on_contact ("the volume's contact witness records the hit")
- interactables_0700000a_live::lift_shaft_hurt_volume_deals_the_fall_damage ("the shaft volume's 1000 damage kills", health stays 100)
- walkreal_07000016_live: the_real_order_launch_chain_matches_the_teleported_one, the_pit_pulse_kills_an_idle_bond_about_450_ticks_later_like_the_game, the_walked_mission_fails_at_the_launch_blast_when_bond_stays_on_the_pit_floor (the 1000 pit pulse no longer kills)
- blast_room_shelter_16_live: the_shuttle_launch_plane_is_solid_only_while_ch_243_reads_high (the plane stays solid after ch 243 drops: the every-tick law 0x17e5d0:26-37 must clear it), the_west_blast_room_shelters_bond_through_the_pulse_and_the_open_floor_does_not (hp 100, hits 0)
- hurt_07000046_live::the_shot_icicle_flies_its_path_at_the_game_pace_and_hurts_bond_in_its_way (the moving icicle's 105 contact no longer lands)
- net_match_end_clients_live::three_machines_reach_the_debriefing_with_identical_boards (tick 765 desync): first check whether it fails on master; if not, find what the fold changed in the MP sim.
The likely root: volumes Bond stood INSIDE or passed through (pit floors, shafts, moving hurts) used to deal damage through the per-tick volume contact; now that they are solid or take the object-hit path, the volume contact no longer fires. In the game both paths exist: the solid planes via the object hit, the volumes via Hurt_Update's own contact. Keep both, with each hurt object on the path the game uses for it [M Hurt_Update 0x17e5d0 + Player_CollisionHandler]. Run every binary named above, all walkreal_*/interactables_*/interact_*/hurt_* binaries one per tool call, and the net_* binaries.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-05-basin-floor-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
