# floor-crack-01 — the 07000001 floor crack at (-4.703, 6.34, -5.968)

Task: on 07000001 a standing guard in STEP_AIM_RIGHT 0x74 loses the floor at x -4.703, z -5.968 and
falls forever (`Drone::feet_on_point` misses; gravity runs). `world.floor_below([-4.703, 7.3, -5.968], 40.0)`
returns None, while points 0.1 away find the floor at 5.34. Four static objects' bounds meet there, so the
triangles leave a gap at a T-junction. Work in progress — this report is updated as steps land.

## Plan

1. Dump the triangles of the static objects whose bounds contain the point; find the gap (T-junction /
   missing triangle / build or transform error / cel scoping in `build_scoped`).
2. Read `Drone_FeetOnPoint` 0x140660 and the `Collide_RayIntersect` (cel-scoped) call at :98; cite why the
   game's ray hits at that point; port that rule — not a widened tolerance unless the decompile shows one.
3. Tests: the feet probe at the point finds the floor; a census of feet probes over the nav nodes of
   07000001 finds no other cracks; un-ignore `the_3x3_hit_rate_and_damage_matrix_matches_the_game`
   (fix_guard_weapons_live) and keep it green.

## Status

- (in progress)
