# Void recovery: the room lookup has gaps

`origin/s/void-recovery-held` is the recovery on master plus the `build_FindCel` retry ahead of it. Do not merge it as it is: full suite 4403-class, about 20 reds, and with the retry many still fail. Tests that place Bond at valid spots get pushed back (`use_census_07000007`, `interactables_07000002`, `interact_07000006`, `fix_death_restart`, `interact_07000007`, `fix_blast_scripts`, `urn_break`...). So `find_cel` returns None where the game finds a room, and a real player could be shoved backwards there.

Branch from it, push to `s/void-recovery-2`.
1. Compare `CollisionWorld::find_cel` with `build_FindCel` 0x1c8cb0 and `build_point_in_bounding_box` 0x1c9500. List the spots in those failing tests where the port says None, and why: cels the port does not load, box bounds, or what the game does for an object with no cel.
2. Fix the lookup so it agrees with the game. Do not narrow the recovery to dodge tests.
3. Gate: those binaries, `void_recovery_07000003_live`, the walkreal set and a count of how often recovery fires across the walks. It should be close to zero; if not, say where.

If the lookup cannot be made to match, report that and leave the fall as it is: a rare fall is better than pushing players back.
