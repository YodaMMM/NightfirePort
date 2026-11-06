# WORKLOG

## void-recovery-2

Branch `s/void-recovery-2` from `origin/s/void-recovery-held` (d51212da2, which is master
ab80ac6ed + 9cff44e0c recovery + d51212da2 build_FindCel retry). Task: the room lookup
(`CollisionWorld::find_cel`) returns None where the game finds a room; tests that place Bond
at valid spots get pushed back (use_census_07000007, interactables_07000002, interact_07000006,
fix_death_restart, interact_07000007, fix_blast_scripts, urn_break...). Compare with
build_FindCel 0x1c8cb0 and build_point_in_bounding_box 0x1c9500, fix the lookup to agree with
the game, gate on the failing binaries + void_recovery_07000003_live + walkreal set + a
recovery-fire count across the walks.
