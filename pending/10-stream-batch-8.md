# Stream batch: the list on the rule pins

Branch from `origin/s/stream-batch-7b`, merge current `origin/master`, push to `s/stream-batch-8`. Full suite on it: 4398 passed, 46 failed, 353 ignored. Read docs/reports/stream-batch-7.md. Same rules as round 7. Also merge `origin/s/bullet-pass-water`, and `origin/s/level-gates-world` without its Player_Move x0.9 change.

Failing:
- abseil_elevator_live
- ai_live
- escort_follow_07000001_live
- fix_npc_doors_live
- fix_portonly_review_live
- goggles_draws_live
- guard_barks_live
- interactables_07000001_live
- mayhew_hold_live
- start_corridor_guard_07000011_live
- tase_silent_live
- upward_shot_0700000c_live
- walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact_real_damage_agent
- walkreal_07000002_live
- walkreal_07000003_live::walk_the_full_route_and_complete_every_objective_real_damage
- walkreal_07000003_live::walk_the_full_route_and_complete_every_objective_real_damage_agent
- walkreal_07000004_live
- walkreal_07000005_live
- walkreal_07000006_live
- walkreal_07000008_live::walk_the_whole_mission_on_real_input_real_damage_00agent
- walkreal_0700000a_live
- walkreal_0700000b_live
- walkreal_0700000c_live
- walkreal_0700000d_live
- walkreal_07000011_live
- walkreal_07000012_live
- walkreal_07000013_live
- walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit_real_damage
- walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit_real_damage_agent
- walkreal_07000014_live::walk_the_tripbomb_route_pick_up_place_and_shoot_it
- walkreal_07000015_live
- walkreal_07000016_live
- walkreal_0700001b_live

Non-walk first, a named cause each. Then plain walks, real damage, Agent, 00 Agent. An ignore with a measured reason is allowed for 00 Agent walks only. Finish with what is still red by name.
