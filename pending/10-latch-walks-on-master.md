# Guards turn off Bond on contact: repair the walks it moves

Branch `s/latch-on-master` is current master plus the drone contact-turn rule (DroneAnim_PreTransCallback 0x13b490:104-116: while a drone is latched on a body contact, the part of its root motion that points into the contact is removed). The full suite on it fails six tests that pass on master:
- walkreal_07000011_live: walk_spawn_to_the_lower_floor_load_point_real_damage, _real_damage_agent, _real_damage_00agent
- walkreal_07000008_live: walk_the_whole_mission_on_real_input
- fix_mission_live: the_07000001_exit_works_end_to_end

For each, find where the run departs from master (log Bond and the nearest guards per tick on both), decide whether the contact rule is applied wrong in the port (compare with the decompile, including which drones latch and when the latch clears) or whether the walk lost its margin, and fix the port or re-route the walk as a player would. Never lower damage or health, never loosen pinned game values, no new ignores.

Branch from origin/s/latch-on-master, push to `s/latch-walks-on-master`, report in docs/reports/latch-walks-on-master.md. Run each touched binary alone, plus walkreal_07000001, 07000007, 07000009, 07000011, 07000008. No draw changes.
