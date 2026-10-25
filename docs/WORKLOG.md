
### level-0c-display-hatch (started 04:28 UTC, 2026-10-25)

Task: 0700000c — two `fix_0c_hatch_live` tests are ignored. (1) `bond_crouches_through_the_open_
vent_to_the_display_ground`: leg 1, the ascent to the duct top, unidentified — the game's climb is
ladder-only [M 0x196118] and the vent has no ladder [D]; find how the player reaches the duct top
in the game [D + M — crates/ledges/jumps, script or lift, the Q-Claw grapple points on 0c] and
walk it with real input. (2) `from_the_duct_top_the_open_hole_carries_bond_down`: legs 2-3 work
(the open hole drops the crouched capsule onto the inner ledge 14.72); leg 4, the descent into the
display case (floor ~7.3 at (5.4, -113) under the 11.27 mid floor) is not mapped — map and walk
it, fix any port divergence [M]. Un-ignore both tests as they go green.

Plan: branch s/level-0c-display-hatch from origin/master e6e3c899; work-slim + ISO downloads;
fast suite runner started early; census 0c floors/crates/lifts/grapple placements [D] + the
climb/ladder/jump decompiles [M]; real-input walks through the real path; un-ignore as green;
report + finish.
