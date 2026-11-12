## Your task: `level-0a-civilian-slide-cels-5`, the 0a civilian slide fix lands with every walk green

Branch from origin/s/level-0a-civilian-slide-cels-4 and merge origin/master; read docs/reports/level-0a-civilian-slide-cels-4.md and -3.md there. The 0a civilian slide fix (the player-visible part, from a user report) is done on this line; it cannot reach master while walks that pass on master are red or ignored. Open, in this order: (1) walkreal_07000001_live's two difficulty-1 walks the -4 branch #[ignore]d (green isolated, red in the whole-binary order - the shared RNG draw order shifts the hall fights): make them green in the WHOLE binary run and remove both ignores; (2) 08 walk_the_whole_mission_on_real_input_real_damage_agent - the live-wire hurt volume at (112.98, 13.47, -177.50): why the route changed; (3) 11 walkreal_07000011_live base real-damage - the airborne wedge at (21.51, 48.03, 15.89) near door (20.0, 46.8, 16.6): name the arm; (4) the 05 d3 / 13 / 15 agent / 11 sister re-tunes in the -3 ledger. Real input only, never lower damage or health, no top-ups, no new ignores. Run each touched walkreal binary WHOLE (not --exact) and the full suite.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-0a-civilian-slide-cels-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: main agent at 2026-11-12T11:33:37Z

---
## Main agent (2026-11-12, released back)
Start from origin/s/level-0a-civilian-slide-cels-5-base (= master 729f9694e + the -4 line squashed; the one drone.rs conflict resolved: master's chat_channel_on line kept above the -4 cels drawn gate). The full suite on that tree: 4889 passed / 41 failed / 367 ignored. The drawn gate (room-visibility instead of the ray) reaches every level's AI, not only 0a - the reds:
- walkreal_07000014_live::walk_the_tripbomb_route_pick_up_place_and_shoot_it
- walkreal_07000016_live::the_chasers_punch_lands_the_record_damage
- walkreal_07000016_live::walk_the_chasers_defeat_completes_the_mission_00agent
- walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch
- walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch_real_damage
- walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch_real_damage_00agent
- walkreal_0700000a_live::walk_office_floor_qworm_keyboard_hatch_real_damage_agent
- walkreal_07000005_live::the_door_band_exit_chains_to_07000006_walked_real_damage_00agent
- walkreal_07000011_live::walk_spawn_to_the_lower_floor_load_point_real_damage
- walkreal_07000011_live::walk_spawn_to_the_lower_floor_load_point_real_damage_00agent
- walkreal_07000011_live::walk_spawn_to_the_lower_floor_load_point_real_damage_agent
- walkreal_07000015_live::walk_spawn_to_the_omega_sector_exit_real_damage_agent
- walkreal_07000013_live::walk_the_warm_chain_to_the_front_entrance_and_out_real_damage
- zoe_gondola_realpath_live::zoe_rides_the_carriage_and_never_targets_bond
- fix_mission_live::the_07000001_exit_works_end_to_end
- walkreal_07000012_live::walk_spawn_to_the_shaft_exit_real_damage
- npc_doors_live::the_06_guard_chasing_bond_opens_the_closed_door
- start_corridor_guard_07000011_live::at_difficulty_3_the_guards_walk_their_disc_owner_points
- walkreal_07000006_live::stopping_before_the_tunnel_never_ends_the_castle
- walkreal_07000006_live::the_castle_chat_pair_stands_chatting_and_breaks_off_when_alerted
- walkreal_07000006_live::the_lamps_light_at_the_ground_mouth
- walkreal_07000006_live::the_wellhead_seals_the_shaft_on_real_input
- walkreal_07000006_live::walk_the_castle_interior_to_the_party
- walkreal_07000006_live::walk_the_castle_interior_to_the_party_real_damage
- walkreal_07000006_live::walk_the_castle_interior_to_the_upper_gallery
- fix_party_mission_live::the_guards_reappear_at_their_places_after_the_kimiko_scene
- walkreal_07000008_live::walk_the_whole_mission_on_real_input_real_damage_agent::walk_the_whole_mission_on_real_input_real_damage_agent
- interactables_07000008_live::glass_pane_breaks_on_the_first_bullet
- walkreal_07000009_live::the_alarm_switch_fails_objective_1
- ai_traces_live::ai_05_truck
- engage_range_07000011_live::the_corridor_guards_hold_off_the_idle_bond
- engage_range_07000011_live::the_guards_run_to_their_point_within_its_accuracy
- fix_npc_doors_live::the_escort_opens_the_interior_doors_on_his_route
- fix_06_fire_live::the_searchlight_courtyard_fight_fires_every_press
- replay_parity_0700000a::replay_the_operative_walk_headless_to_the_hatch_exit
- rook06_realpath_live::the_switch16_guard_fights_through_the_door47_doorway
- fix_cutscenes2_live::the_drake_flythrough_hides_the_live_party_and_restores_it
- walkreal_07000003_live::walk_the_alpine_route_to_the_exit::walk_the_alpine_route_to_the_exit
- rappel_vanish_0700004a_live::a_shot_rope_guard_falls_instead_of_vanishing
- sight_screens_07000001_live::released_guards_see_and_fire_at_bond_beyond_the_screens
- fix_06_driver_live::the_scared_cower_keeps_him_in_place_on_the_floor
First decide whether the drawn gate itself is right for non-civilians (which drone code reads `drawn` - sight? animation? physics?) against the decompile, before re-tuning any walk.
- Main agent, decompile answer to "is the gate right for non-civilians": YES. `View_AddObjects` 0x1e62d8 queues only objects linked into the visible cels (the VisCelList walk) and writes obj+0xFC = 2 for each queued object; obj+0xFC is read for EVERY drone by NDrone2_DoGravity 0x141e58, DroneAnim_SetDAnimInternal 0x13aee8, NDrone2_BondTalk 0x1446b0, Bullet_heatseekable 0x12f248 (homing only seeks drawn objects), DroneSpawner_Control 0x137e68 and control_movement_object_handler 0x1336f8. So the cel-visibility drawn gate is the game's rule for all drones; the 41 reds are its consequences - re-derive the walks/pins, do not narrow the gate to civilians.

---
claimed-by: sandbox5 at 2026-11-12T16:22Z
