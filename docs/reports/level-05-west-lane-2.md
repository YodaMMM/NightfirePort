# level-05-west-lane-2 — 07000005: the west lane and the cable crossing (round 2)

Branch `s/level-05-west-lane-2` (from origin/master a67f1a27e). No port source changed; the
deliverable is the answered blocker, the mapped real lane, and the re-anchored test.

## What's done

1. **The round-1 blocker is ANSWERED — it was the wrong lane.** The test's old question
   ("every yaw sweep stalls at x ≈ 57.9-58.0 — the game's collision, the port's, or the wrong
   lane?") is settled: the ladder balcony is a closed PERCH, stopped by the game's own
   geometry. [D] The 07000005 collision world: westward rays from the ladder top (61.6, 34.6,
   89.2) hit x 57.74 (z 89.2) / 58.16 (z 86); southward rays along x 58-60 hit z 85.1-86.2;
   there is no walkable floor at x 52-62 south of z 84 down to y 20 (the courtyard below). No
   Object_Tower_Gate/BlockingPoly static stands there — it is the map's own parapet tris. The
   port's collision matches the disc; nothing to fix in the port.

2. **The REAL west lane is mapped [D]** (the two-hit floor map + 1-2-unit fine maps of the
   disc collision, reproduced in the test's doc comment):
   - the lane runs from the courtyard's west strip (x 28-56, z 92, y 26.3-30.3) SOUTH along
     the castle wall (x 24-48, z 44-92, y 28-36 — the guard cover nodes #75-#78 sit on it
     [D placements]), over the tower top (x 28-32, z 44-62, y 36.0) and down its west slope
     (x 20-24, z 52-64, y 31-34);
   - the "Wall Hug" placements are **CREEPWALLS** — class 0x3F → `CreepWall_Create` 0x196fc8,
     the crawl substate 7 [M 0x196fc8 + 0x1a5170:66-97, the port's `creepwall.rs`]: "Wall Hug
     Short" #299 (25.79, 31.78, 67.76) with the type-6 crawl-prompt third-icons #256/#257
     (24.2, 31.9, 60-70.6) [D], and "Wall Hug Window" #35 (−34.10, 37.28, −31.65) on the
     window ledge. The narrow sections the plain walker cannot thread are the game's own
     crawl sections;
   - the wire #251 "Wire_Cable" (−23.49, 34.47, 6.26 [D]) crosses an **8-unit notch** in the
     cable ledge: the ledge (y 30.8-33.3 at x −25..−23) spans z −10..46 EXCEPT z ≈ 4-10,
     where the floor dips to y 17-25 [D the fine map]. The 3rd-person grab icons #277/#300
     (−23.56, 32.24, 0.2-2.65) and the big icon #310 (−23.80, 32.74, 12.33) mark the notch's
     edges — the wire's r-4.0 sphere reaches a standing Bond's head there, so the action-edge
     grab [M Player_HandleJump 0x1916b0:126-147] does not need a jump from the correct spots
     (round 1's "needs the jump-grab" note came from the wrong anchor standing 4.5+ away).

3. **The test is re-anchored on the real lane.** The first two legs walked GREEN through the
   real input path (courtyard west → (53.2, 27.7, 96.2); the lane north end → (44.4, 29.7,
   91.1)); the strip's micro-navigation iterations are documented (the z 90 strip is only
   x 28-40 wide; the 29.3→30.3 lip at x 42-44 wedges the strict walker — the legs run lenient
   so the breadcrumbs always land). The creepwall-E leg is in place for round 3.

## What's left (round 3)

- The lane's strip walk south of z 92 (anchors are in the test; the wedging lip at x 42-44
  needs either a hop or a sidestep-heavy approach), the crawl through #299 and the lane→ledge
  link, the notch grab at (−24.6, ~31, 10-12), the ride south, the window-ledge crawl #35 and
  the exit leg. The fine map in the test's doc is the anchor source.
- A live check would settle the game's own choreography: on a FRESH boot into 07000005, trace
  Bond's position while walking to the cable (the main agent's `tools/trace/gdbwatch.py` on
  the player object's position triple, or a `--bond` render at (44, 30, 91), (30, 33, 80),
  (25.8, 32, 67.8) — the lane, and (−24, 31, 12) — the notch's north edge, which should show
  the E-grab prompt icon #310's spot).

## Verification

- The probes were scratch examples (not committed): the west placements dump, the two-hit
  floor map, the 1-2-unit fine maps, all against the disc's parsed level and the port's
  CollisionWorld built from it [D].
- The test compiles; the ignored test's legs were run twice with `--ignored --nocapture`
  (courtyard + lane north end green; the fall/wedge iterations logged in the WORKLOG). The
  other walkreal_07000005 test (`walk_to_the_gatehouse_and_ride_the_truck`) was not touched.
- The full suite (`tools/test/suite.py`) was started in the background at 17:53 UTC on this
  branch's base tree; its build-verify plus early tests produced no new failure classes
  before this report (the window closes before the total line — the next message reads
  `tail suite.out` / `~/suite.log`). No crate source changed, so the suite's risk is nil; the
  touched crate's test binary (`walkreal_07000005_live`) was built and run directly.

## Usage

not visible to me
