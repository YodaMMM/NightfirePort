# WORKLOG

### level-06-castle-walks (started 22:49 UTC, 2026-10-24)

Task: `walkreal_07000006_live::walk_the_castle_interior_to_the_upper_gallery` and
`stopping_before_the_tunnel_never_ends_the_castle` are still ignored. The old blocker (the fire
fault: one round per 12 presses, the view turning on the press tick) does not reproduce on master
— fix_06_fire_live pins every press firing and no body turn. Run both walks un-ignored on master;
fix what stops them with real input and the game's rules [M] (the courtyard fight with the alarm's
reinforcements, the doors, the route to the upper gallery); un-ignore both when they walk; the
sister must still be able to fail.

Plan: fast suite in background; run the two ignored walks to see the current wedge; read
common/walker.rs climb_stairs/walk_to; attack the tunnel ramp (y 42 -> 49) and whatever else
stops the walk; un-ignore; full suite; report.

- (22:49 UTC) Sandbox kept repo/work/ISO from the previous session (ai-dstates-uncited, pushed
  complete): skipped setup, branched s/level-06-castle-walks from origin/master at 7dbc9cb1,
  pushing this checkpoint. Suite start next.
