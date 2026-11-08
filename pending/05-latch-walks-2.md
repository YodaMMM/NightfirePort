# Guards turn off Bond on contact: the last three reds

Branch `s/latch-batch` is current master plus the drone contact-turn rule and its first walk repairs (read docs/reports/latch-walks-on-master.md on it: sections "Failure 2 sisters", "Failure 4", "Failure 5" and "What's left"). Its full suite fails three tests that pass on master:
- walkreal_07000011_live::walk_spawn_to_the_lower_floor_load_point_real_damage_00agent (and the Agent sister if red): guard #33 now parks at contact distance and fires point-blank on the 23.0 strip.
- walkreal_07000008_live::walk_the_whole_mission_on_real_input: not yet diagnosed.
- fix_mission_live::the_07000001_exit_works_end_to_end: the courtyard shoji ping-pongs 0x8/0x10 and never opens (NDrone2_DoorIsOpen 0x14aaa0). Log the shoji state and every drone activating it per tick on master and on the branch; this one may be a port bug in who re-activates the door.

For each: decide port bug vs lost walk margin with evidence, fix the port (test red on old code) or re-route as a player would. Never lower damage or health, no loosened pins, no new ignores. Branch from origin/s/latch-batch, push to `s/latch-walks-2`, report in docs/reports/latch-walks-2.md. Run each touched binary alone, then walkreal_07000001, 07, 08, 09, 11.
