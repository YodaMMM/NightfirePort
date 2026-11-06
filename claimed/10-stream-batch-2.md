# Stream batch: the 31 reds

Branch from `origin/s/stream-batch`, merge current `origin/master`, push to `s/stream-batch-2`. Keep every gas and muzzle draw. Read docs/reports/stream-batch.md.

Full suite on that branch merged with master 8e72343f8: 4397 passed, 31 failed, 353 ignored. This is the one re-pin for gas, guards and muzzle; do not split it.

Non-walk reds first. These are not route tuning, so each needs a reason before any change: is it a stale pin, or did the merge break something? models_misc's muzzle puff, gun_turrets' burst order and fix_grenade_types' stun are the ones to suspect.
- enemy_accuracy_live::hits_per_10s_and_damage_per_hit_are_pinned
- fix_grenade_types_live
- fix_npc_doors_live
- fix_pickups_ammo_live
- grenade_live
- guard_drops_flashbang_live
- gun_turrets_live
- hold_npcs_live
- mayhew_hold_live
- models_misc_live
- smg_close_range_live
- start_corridor_guard_07000011_live
- upward_shot_0700000c_live

Then the walks, plain before real damage before Agent before 00 Agent:
- walkreal_07000001_live
- walkreal_07000002_live
- walkreal_07000006_live
- walkreal_07000008_live::walk_the_whole_mission_on_real_input_real_damage_00agent
- walkreal_0700000b_live
- walkreal_0700000c_live
- walkreal_0700000d_live
- walkreal_07000011_live
- walkreal_07000012_live
- walkreal_07000013_live
- walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit_real_damage
- walkreal_07000015_live
- walkreal_07000016_live

Also, one review point: `DroneFunc_HandleExplosives` 0x1435f0 `break`s after the FIRST kind-2 record in radius, whether or not that record passes the visibility check, then sends at most one 0x21. Check the port's live-grenade loop does the same; it reads as walking on to later grenades. Remove the NF_EXPPROBE debug print from drone.rs.

An ignore with a measured reason is allowed for 00 Agent walks only. Do not loosen pins tagged [L] and do not change game code to suit a walk. Finish with one total line from a full run.

---
claimed-by: sandbox3 at 2026-11-06T13:56Z
