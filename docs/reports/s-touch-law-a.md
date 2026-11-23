# s/touch-law-a

**Shipped random stream: unchanged.** `drone::TOUCH_LAW_LIVE` is false by default, and with it
false `human_touchers` returns the same stand-in set as before. **With it true the stream moves.**
The 01 and 02 walks part from the box runs in their first fights. For example, 02's garden pursuer
reads 8.8 u under the law and 8.9 u under the box at the same walk line.

Based on `s/touch-ref-point-p`. This branch covers levels 01, 02, 05, 06 and 07. Another branch
takes the rest.

## The switch

`human_touchers` returns `game_touchers` (the walk HITTEST segment under `Intersect_CylGeom`
0x109da0, with no box and no containment) when `TOUCH_LAW_LIVE` is true. Otherwise it returns
`box_touchers`. No test adds the box back.

## Per test (red under the law before this branch)

| Test | Why it was red | Now |
|---|---|---|
| mayhew_handover_live x2, fix_truck_live x2, walk_07000005_live, walkreal_07000005_live (05 gatehouse ch 0x5C) | The box fired the hand-off quad (y 27.05) while Bond stood. The game's standing segment top sits 0.56 under it, outside r 0.55. The reference trace stood there 212 frames with ch 0x5C at 0, jumped, and ch 0x5C rose [L tests/traces/05-truck.trace 33244-33464]. | Bond jumps in place while the plate is up. Under the law a standing fire is an assert failure (mayhew). |
| walk_07000006_live, walkreal_07000006_live (06 LoadPoint #746) | Bond was dropped inside the volume. The game has no containment contact [M Trigger_Hit 0x1b1fc8 reads only the pick's list]. | Bond lands past the sill and walks north into a wall of the volume. The walkreal poll also accepts ch 0x02, the scene's own closing write, because the Touch channel drops once Bond is off the wall. |
| interactables_07000007_live roof music #443 | The plate faces up, 1.3 over the step top where the test stood Bond. | Under the law it asserts that standing and jumping on the step do not fire it. It then crouches on the rail top under the plate, which fires it. |
| reveal07 helper (Drake's reveal #356, used by the 07 cut-scene tests) | Bond was dropped at the centre of a thin slab. | Bond walks north into the slab from 2 u south. |
| walkreal_07000001 exit fade | Under the law the contact died at the stairwell guards. | Lever 2000/30 to 2000/60. Of 12 values, 5 complete under the law. 2000/60 also completes under the box. |
| walkreal_07000002 00 Agent | Under the law Bond wedged at (-33.5, 9.6) coming in from the east and never reached the ch-0x17 plate. | Lever under the law 4000/90, the only one of 12 that completes. It fails under the box, so the stop is picked on the const. |
| walkreal_07000002 Agent | Under the law the run lost a captor kill. | Lever 3000/60 to 1500/45, which completes under both. |
| walkreal_07000002 to the exit | Under the law the level exit fired during the geisha-1 captor approach. | A 2000/30 stop, which completes under both. Under the law this walk crosses every plate itself, with no driven raise. |

## Tests run (both values of the const unless noted)

- walkreal_07000001_live: 6 pass, 1 ignored. following_mayhew_down_completes_the_level loops on a
  door at (-14.1, 5.3, 13.1) for over 10 minutes under BOTH sets. That predates this branch and
  is not judged here.
- walkreal_07000002_live: 5/5 with the const off and on.
- walkreal_07000005_live 17/17, walkreal_07000006_live 8 pass + 1 ignored, walkreal_07000007_live
  6/6.
- walk_07000005/06/07 (2 each), fix_truck_live 2, mayhew_handover_live 3,
  interactables_07000007_live 4, interact_07000007_live 8.
- Law only: walk_07000001/02, interactables_07000001/02/05/06, fix_party_mission_live. Each
  passes, except fix_party_mission's 08 safe chain, which is the 08 cable-car floor #236 from
  the other set.

## Unproven

- **#443 route.** No route to #443's rail top is pinned. The rail crouch is a teleport transit.
  Whether the game reaches the plate another way is not traced.
- **02 Agent nondeterminism.** Under the law, 02's Agent walk with its old 3000/60 stop passed
  once alone and failed alone on a later build. That build differed only in other tests' stops.
  The cause is not found.
- **02 ch-0x17 plate wall.** The walk to the ch-0x17 plate (to the exit test) still stops at the
  wall end west of the quad, at (-36.8, 5.2), in both sets. The box fired the plate through that
  wall. Under the law the 2000/30 run reaches the plate. A waypoint round the wall end would make
  the leg exact.
