# Stream batch: the joined tree

Branch from `origin/s/stream-joined` (stream-batch-5, the bullet water fix and master merged, 1b walks taken from master). Push to `s/stream-batch-6`. Full suite on it: 4383 passed, 59 failed, 353 ignored. Read docs/reports/stream-batch-5.md. Do not merge any `s/void-recovery*`.

Failing now:
- abseil_elevator_live
- ai_live
- anim_events_live
- escort_follow_07000001_live
- fix_npc_doors_live
- fix_portonly_review_live
- goggles_draws_live
- guard_barks_live
- guard_reaction_live
- guard_side_gates_live
- interactables_07000001_live
- mayhew_hold_live
- spawner_kill
- start_corridor_guard_07000011_live
- tase_silent_live
- upward_shot_0700000c_live
- walkreal_07000003_live::walk_the_full_route_and_complete_every_objective_real_damage
- walkreal_07000003_live::walk_the_full_route_and_complete_every_objective_real_damage_agent
- walkreal_07000004_live
- walkreal_07000006_live
- walkreal_07000008_live::walk_the_whole_mission_on_real_input_real_damage_00agent
- walkreal_0700000b_live
- walkreal_0700000c_live
- walkreal_0700000d_live
- walkreal_07000012_live
- walkreal_07000013_live
- walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit_real_damage
- walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit_real_damage_agent
- walkreal_07000015_live

The count doubled from 28 on stream-batch-3 and the new reds are not walks, so bisect them before anything else: `ai_live`, `guard_reaction_live`, `guard_side_gates_live`, `guard_barks_live` ("the guard never reached AIM_STAND_FIRE"), `spawner_kill` ("second spawn, got None"), `anim_events_live`, `tase_silent_live`. Suspects: stream-batch-5's removal of the re-acquire block and exit, and the spawn-pick merge. A guard that never fires is a regression, not drift. Say which commit did it.

Then the 0x5c follow wedge from that report: the d2 guards' stall. Then non-walk reds, naming a cause before touching a test. Then plain walks, real damage, Agent, 00 Agent. A walk_to cap needs a look at what stands there. An ignore with a measured reason is allowed for 00 Agent walks only. Do not loosen pins tagged [L] and do not change game code to suit a walk. Finish with one total line.
