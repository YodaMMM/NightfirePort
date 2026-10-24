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

- (22:49-23:20 UTC) Both walks ran un-ignored on master: the whole route works (the fire path,
  guard #438, doors #47/#59/#71, the courtyards) and the real wedge is the TUNNEL: a straight
  shot at (4.0, 49, −62.5) grinds in the shaft's south-east corner (jump apex 46.6 < the 48.8
  upper-ramp slab face [D floor-map probes]). Dense floor/column maps (floor_below stacks,
  x 2.0..7.0 × z −50.5..−65.5) show the shaft is a SWITCHBACK: low ramp 42.0→44.8 (z −51.5..−58.5),
  an east lane x 6.25-7.0 rising 44.6→46.8 northward, the upper ramp 47.2→48.8 (z −55.5..−59.25)
  back south to the 49.0 landing (z −59.75..). Rewrote the climb as seven legs following those
  surfaces (climb_stairs for the low ramp + corner, tight arrive radii so each leg reaches its
  mark); the upper ramp + the landing seam walk through a new shared no-jump helper
  walker.rs::walk_slope — an airborne seam crossing re-links Bond onto the ramp cel's plane
  under the landing slab and the south shaft void swallows him (the seam probe: grounded the
  whole crossing at root ~50.0; jump-pulsed legs fell past y 47). Both tests un-ignored;
  walk_the_castle_interior_to_the_upper_gallery + stopping_before_the_tunnel_never_ends_the_castle
  pass together (2 passed, 26.9 s, no falls); the sister's negative asserts (no ch 0x83, no
  objective fold, no exit) unchanged. Level doc's walk-status paragraph rewritten. Suite next.
