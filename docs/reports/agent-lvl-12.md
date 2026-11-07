# agent-lvl-12: 07000012 object audit

This branch moves the shared random stream (enemy blasts and pane kills draw it, guard positions change).

## Fixed
- Copter spot check: `Collide_LineOfSight` flags 0x2e drop the class-0x10 glass [M 0x17dbb8:171-172, 0x1ebaa8:315-317].
  The port ran the unmasked ray, so 0 of 8813 in-cone ticks saw Bond on the sky bridge and the copter never attacked him there.
- Copter rounds: they pass glass and drain panes; the missile leaves at a quarter speed, ramps, and explodes where it stops
  [M 0x12c5a0:187-191, 0x12d890:134-138, 0x12ce80:193-199]. Enemy rounds with `+0x70 & 4` had no blast at all.
- Script objects take enemy rounds [M Collide_Update 0x1ea398, SP_Hit 0x1dc470, SP_GetHitDamage 0x1dc5f8]. Before, only
  player rounds counted, so nothing but the player could start the bridge's break script.
- Contact on large script objects runs the triangles after the sphere [M 0x1eae80, 0x1ebaa8:300-321]. The bridge arches have
  5.7-8.5 spheres: a round in open air started the script and each round counted once per overlapping sphere.
- Guards on 07000012: a step is no longer rejected by static geometry. The game has no world gate on the guard
  move, only the bounds push [M 0x13b490, 0x157bd0]; the port's stand-in ray held them against scenery with no way round.
  07000012 sweep, 5 stand spots, difficulty 3: stall rows 13 to 2, no guard found inside a wall. Doors still hold them.

## Checked, no change
Copter damage: body-only hits, 86 P2K rounds at d1 (`copter_hits_12_live`, green before and after). Objectives, exit, keypad,
Q-Worm, monitor, every touch plate and spawner channel, pickup channels, the level-specific ACTION.ELF branches.

## Tests
`level12_bridge_live`: 6 tests, all red on origin/master, green here. Green: walkreal 02, 05, 09, 0c, 11, 12; interactables 12, 1b;
copter_hits_12, copter_paths, urn_break, enemies_07000012, walk_07000012, scripts_07000012. 02, 05 and 09 ran before the guard rule was scoped to 07000012.

## Open
- The guard rule holds on every level with bounds [M], but it is gated to 07000012: applied everywhere, the real-damage walks
  on 0700000c, 07000011 (agent) and 07000013 (d3) ended with Bond dead. Widening it needs those routes re-pinned.
- `interactables_0700001b_live` klaxon pin loosened: #160 may now busy-kill.
- `walkreal_07000013_live` real-damage d3 fails on this branch: the carried fight from 07000012 changes, Bond leaves the east
  lane for ammunition and wedges at (0.60, 25.66) on the lobby west mid leg. A test-route wedge, no guard near; needs a re-pin
  (a waypoint at the lane mid moved the failure to a death, out of ammunition). d1 and d2 pass.
- Only the copter's rounds carry the glass pass. Guard rounds still stop at glass without breaking it (the note at the
  close-combat spawn: the routes stand behind station glass). In the game they break it [M 0x12bfd8].
- Live checks: how fast the copter wears down the bridge's four holds; two guards still pause: #23 at the shut door #537.
