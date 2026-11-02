# sniper-spawn-stats-2 — seed spawner stats from the load-time table without breaking 14/16/1b

Branch from origin/master; merge origin/s/sniper-range-accuracy-0c-0d in first.

That branch seeds `drones.stats_table` with the level's load-time table so spawner spawns read the shared type record [M 0x137cb0 -> 0x1c7a98/0x14c600] (and adds Drone::spawn_stats_params). On master it turns 8 tests red, all about SPAWNED drones:
- walkreal_07000014_live: the_e_use_switch_fires_the_sabotage_cascade_and_opens_the_west_door, walk_the_infiltration_to_the_delta_exit, walk_the_tripbomb_route_pick_up_place_and_shoot_it
- walkreal_07000016_live: the_chasers_punch_lands_the_record_damage, walk_the_chasers_defeat_completes_the_mission_on_real_damage
- drake_fight_live::drake_initializes_as_the_zero_g_boss
- walkreal_0700001b_live::the_astronauts_fire_at_accuracy_9_for_3_a_hit
- astronaut_guards_live::astronaut_health_is_take_x_hits_of_weapon_0x33
Several of these pin [D]/[M] values (astronaut accuracy 9, Drake's init, the chaser's punch damage) that the new table read changes.

Do:
1. Find which spawn reads differ: for each red, the record the game reads for that spawn [M the spawner → behaviour_util_get → DoModeSettingsNEW path, including any per-spawner section that overrides the load-time record] versus what the branch reads.
2. Make the spawn read exactly the game's record in every case (load-time table where the game uses it, the spawner's own section where it has one). Cite each.
3. All 8 tests green with their pins unchanged (unless a pin is proven wrong — then cite the fix), plus the 0c/0d walks' damage half. Add the new field to every Drone literal (cargo test -p nf-game --lib --no-run). Run every walkreal_*_live, then the full suite.

---
claimed-by: sandbox4 at 2026-11-02T20:50Z
