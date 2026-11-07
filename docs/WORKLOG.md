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

## recovery-loop-07

Branch `s/recovery-loop-07` from `origin/s/void-recovery-2` (670687740). Task: on 07000007 the
recovery loop fires at (94.84, ~43.1, -72.19): cel 15's box is y 42.75..54.25, and the walk's
push-out puts Bond outside it. Find what pushes him out and compare with the game's collision at
that spot; report whether recovery can merge; count recovery fires across every walk.

Outcome (this session): the loop was not a push-out — the census's raw `bond.pos`
stand-ins buried Bond half a unit UNDER the corridor floor (door 4's pos y sits ON
the floor), no floor ray or capsule contact sees him, he fell out of cel 15's box
and the recovery yanked him back every 2 ticks (2416 fires). Fix: `place_bond` now
takes the game's placement snap (`Player_StandAtNewPosition` 0x190d88:104-116 +
`build_PointOnFloor` 0x1c95a8 — hit: y=hit+1.1, miss: y=+1.2, unconditional) and the
census places through it. use_census_07000007 green, 20/20 census rows green,
void_recovery_07000003_live + fix_death_restart_live green, interact_07000007 7/7,
interactables_07000002 18/18; walkreals 01/04/05 green with 0 recovery fires. TWO
gate tests went red (fix_blast_scripts type-0 smash, interact_07000006 well band) —
their bands were calibrated on the no-snap stand-in; live checks requested in the
report before re-tuning. See docs/reports/recovery-loop-07.md.
