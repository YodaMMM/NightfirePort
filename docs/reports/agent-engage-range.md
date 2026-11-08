# agent-engage-range: 07000011 / 07000014 engage ranges and the guards' stand-off

**This branch moves the shared random stream** (it sits on s/agent-lvl-0d-11, which already moves
it, and changes guard ranges, routing and goal radii on every level).

## Fixed

| What a player sees | Cause | Evidence |
|---|---|---|
| On 07000011 and 07000014 guards engaged from the class range (12-16+) instead of the level's. | `NDrone2_DefaultInit`'s level overrides were missing: 11 sets engage/combat 6/9 (5/8 for anim set 9), 14 sets engage 10 (7 for set 9, set 5 keeps its draw). | [M 0x14b300:429-457, raw 0x14bb60-0x14bc1c re-read] |
| With those ranges, the 11 start-corridor guards walked to 4.1 from an idle Bond, stopped at the AI boundary and stood there blind, never firing. | The player's AI target searched every node within 36 of Bond. The game searches with radius 0, and only among the nodes filed under Bond's own cel. Bond's start cel 156 holds no node of path 1, so the game gets no goal, the A* returns 4, and the guards cannot route to him. The port routed to fallback node 39 and on into the boundary. | [M NavNodeCache 0x155cd8 `mtc1 $zero,$f12`; BuildAITarget 0x15ac78 `mov.s` pair; NodesForPosition 0x1586a8; NodeSearchCel 0x156e18; Link2Cel 0x1d2ec8; DoAStarPath 0x1571b0:52-68] |
| Run-to guards stopped 2.0 short of their point. | The point's own accuracy (+0x14, 1.0 on both office points) is the goal radius. | [M FindRunToPoint 0x153168:42-45, SetupGoalPosition 0x15acf8, SetupRouteToPosition 0x158b10:104] [D accuracy 1.0] |
| A guard looking for a closer spot walked to 2.0 from the AI wall. | `SetGoalToCloserPosition` asks `MoveToGoalPosition(3.0, …)` and needs `build_FindCel` to place the wall point. | [M 0x155e10 disasm 0x155e48-0x155e74] |

With no route, `DroneFunc_CombatState` keeps a firing guard firing (no 0x5a exit) and sends a
moving one to CombatNoRoute 0x5f [M 0x148338 case 0x10 / case 2 default]. The guards now hold
near their run-to points, step-aim and fire. The port's distances from the idle start are 9.7-10.7
for #126 and 7.6-8.7 for #127 (one run moved #127 in to 5.0). The capture's are 8.3-8.9 and 5.8-5.9
[L docs/live/2026-11-01-07000011-start-corridor-guards.md].

## Tests

- New `engage_range_07000011_live` (4 tests). All 4 are red on the base 6f21f46d4 and green here.
  The base gave ranges of 14.3/17.8, a fallback goal of node 39, an arrival 1.99 from the point,
  and #126 within 3.62 of Bond.
- Green: start_corridor_guard_07000011 8/0, interactables_07000011 17/0, stairwell_guards_07000011
  3/0, spawner_waves 4/0.
- walkreal_07000011: 4/1. The new red is `walk_spawn_to_the_lower_floor_load_point_real_damage_agent`,
  which no longer reaches the #193 exit load. That walk was tuned to guards that chased. It needs a
  re-pin, and the failure was not traced further.
- walkreal_07000014: 7/2/1i. Both real_damage reds were already red on the base (the descent
  landing).
- Not run: the other levels' walkreal binaries. The cel-limited target applies everywhere. At the
  start, Bond's cel holds no node on 02, 08, 0b, 0c, 11, 15, 16 and 4a, so expect more fight drift
  in the batch re-pin.

## Unproven

- #126 and #127 in the port still hold ~1-2 units further from Bond than in the capture. The game's
  guards stand about 0.7-1.8 past their run-to points toward Bond (z≈17.4 and 20.0). The port's stand
  just short of the points. The cause was not found. Candidates are the step anims' root motion and
  an early CombatNoRoute closer-position move. For a live check: on a fresh d2 start, record #127's
  state stream from f150 to f250. Does it pass through 0x5f and a second 0x63?
- The port's node cels come from its `find_cel`, which reimplements `build_FindCel`. The node-to-cel
  filing was not compared with live MAPNODE+0xC.
- An unlinked player (`cel = None`) keeps the old one-cel search. In the game a null cel finds
  nothing.
- Other `NodesForPosition` users (goal routes) still search without the cel limit.
