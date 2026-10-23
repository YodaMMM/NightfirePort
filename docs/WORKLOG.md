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

Work log:
- (18:10) The disc probes (scratch examples, not committed): the west-region placements, the
  two-hit floor map of the collision world, and the 1-2-unit fine maps of the ladder balcony,
  the lane, the tower descent, the cable ledge and the wall's west face.
- (18:30) THE BLOCKER ANSWERED [D]: the round-1 lane was wrong. The ladder balcony is a closed
  perch — the game's own parapet stops a real walk at x ≈ 57.7-58.2 (westward rays from
  (61.6, 34.6, 89.2): hits at (57.74, z 89.2) and (58.16, z 86)) and at z ≈ 85-86 southward
  (rays along x 58-60 hit z 85.1-86.2); no walkable floor at x 52-62 south of z 84 down to
  y 20. Not a port collision bug — the disc's own tris.
- (18:40) The REAL west lane mapped [D the two-hit floor map + the fine maps]: the courtyard's
  west strip (x 28-56, z 92, y 26.3-30.3) → south along the wall (x 24-48, z 44-92, y 28-36;
  the guard cover nodes #75-#78 sit on it) → the tower top (x 28-32, z 44-62, y 36.0) → the
  west slope (x 20-24, z 52-64). The "Wall Hug" placements are CREEPWALLS (class 0x3F →
  `CreepWall_Create` 0x196fc8, the crawl substate 7 — the port's creepwall.rs): "Wall Hug
  Short" #299 (25.79, 31.78, 67.76) with the type-6 crawl-prompt icons #256/#257, "Wall Hug
  Window" #35 (−34.10, 37.28, −31.65). The wire #251 crosses an 8-unit NOTCH in the cable
  ledge (the ledge y 30.8-33.3 at x −25..−23 spans z −10..46 except z ≈ 4-10 where the floor
  dips to y 17-25); the grab icons #277/#300/#310 mark the notch's edges (the wire's r-4
  sphere reaches a standing head).
- (18:50) The walk attempt on the new anchors: "courtyard west" and "the lane north end" walk
  green (real input, (53.2, 27.7, 96.2) → (44.4, 29.7, 91.1)); the third leg's direct line cut
  the corner over the void (Bond fell to y 0.3 at (23.0, 92.8) — the strip narrows to x 24-36
  at z 80-86) — the anchors tightened.
- (19:12) The test rewritten: the corrected doc (the answered blocker + the found lane + the
  open crawl/cable legs), the updated anchors, the creepwall-E leg. STILL #[ignore] — the
  crawl legs + the notch grab need round-3 harness verification; the ignore reason now states
  the answered blocker instead of the stale open question.

Stage summary:
- Round 2's deliverable: the wrong-lane proof [D], the real lane's walkable map [D], the
  wall-hug = creepwall identification [D+M], the notch/wire grab geometry [D], the walkable
  legs verified green through the lane's north half. The crossing itself is round 3: the
  crawl through #299/#35, the lane→ledge link, the jump/E grab, the ride.
