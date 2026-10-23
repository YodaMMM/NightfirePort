# Work journal

## level-05-west-lane-2 (branch s/level-05-west-lane-2, started 2026-10-23 17:52 UTC)

Task: un-ignore `walkreal_07000005_live::walk_the_west_ledge_and_cross_the_cable` — the west
lane and the cable crossing (round 2). The test's doc comment names the blocker: the ledge
west of ladder #151's top dead-ends at x ≈ 57.9-58.0 (every yaw sweep stalls; the yaw-5.89
walk drops off the ledge's south edge to y 28); the wire #251 grab spot (−23.5, 34.5, 6.3) is
unreachable from there. Open: the game's own collision (the tower/BlockingPoly statics), the
port's collision of it, or the wrong lane. The level doc adds: the west anchors predate the
ledge's real walk heights (the cable-mouth anchor (−20, 34.5, 8) stands at y 28.0); the
wall-hug window ledge (plates #311-#313) walks at y ≈ 35.3. Round 1
(origin/s/level-05-west-ledge) pushed nothing but the marker.

Plan:
1. Disc probes (scratch, not committed): the west-region placements (triggers #301/#305-#315,
   wire #251, ladder #151, the statics) with positions — the plates trace the game's real
   lane; the collision tris around the stall (x 55-63, y 32-37, z 84-94) — what stops Bond.
2. Decide: a port collision bug (fix it with [M]/[D]) vs the wrong lane (re-anchor with
   [D]-backed waypoints; the walk heights from the disc meshes).
3. Real-input only (walker: movement/aim/fire/use/ladder/wire); un-ignore with a sister that
   can fail; suite before push.

Work log:
- (17:52) branch created from origin/master a67f1a27e, WORKLOG started. Round 1's branch
  carries no work; the findings above are the test's doc + docs/levels/07000005.md.
