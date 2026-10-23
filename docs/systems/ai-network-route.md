# AI navigation helpers — the call-tree remainder

The call-tree check (`tools/calltree/calltree.py --all-mentioned-dstates --list`) flagged 8
AI-navigation helpers reachable from the ported guard states that no Rust file named. Each one
is the port's own work under another name; the citations now carry the game name and address.
The check now lists none of them. The port code itself is unchanged (comment-only
diff) — everything here was already wired and tested; this note is the map.

Evidence tags per `docs/fields/README.md`: [M] decompile (`work/decomp/ACTION.ELF/`), [S]
suggested / port choice.

## The node search

### `AINetwork_NodeSearchCel` 0x156e18 — the cel-filtered node scan
The candidates: every node of one AI path inside the box `±√(r²)` (15.0 when r² = 0) in x/z,
`±2.0` in y [M 0x156e18:13-23], `Vec_Dist2D` ≤ r² against the query point, nearest first
(the game inserts into a sorted list as it walks the cel's node chain [M 0x156e18:36-60]).
The callers drive the port's scan: `AINetwork_NodesForPosition` 0x1586a8 / `AINetwork_DoAStarPath`
0x1571b0 / `AINetwork_EmitPath` 0x158cf8 / `AINetwork_NodeSearch` 0x15aa88. The game walks the
per-cel node lists (`AINodeSearch+2` = the cel); the port is one cel [S deviation — the
`Network::build` header note].
- Port: `Network::candidates` (nav.rs) — used by `target_nodes`, `astar`, `node_from`, `emit`.

## The boundaries

### `AINetwork_LineIntersectsBoundry` 0x157990 — the boundary wall intersection
2D line-line intersection of the segment with the boundary link's posts (`BLINK+8/+0xA` = the
nodes, the post heights `+8`/`+0x14` interpolated along the wall [M 0x157990:60-77]); hit when
the segment's height at the crossing is within `[wall − 2.4, wall + height]`; the `adjust`
argument (2.0 over) lowers both post heights to the segment's height − 0.5 [M 0x157990:82-86].
Callers: `AINetwork_BoundsTest` 0x157d80 (the port's `bounds_test`), `InitPassableBoundries`
0x15a5c8 (the build-time pass → `Network::build`), `AINetwork_FurthestPosition` 0x15a8d8 (the
port's `first_wall_hit`), `AINetwork_ModIntersectedLinkFlags_Bounds` 0x15b6f0.
- Port: `intersects` / `intersect_adjust` (nav.rs) — the intersection math, the −2.4/+height
  band, and the adjust write, as written.

## The routes

### `AINetwork_ResetRoute` 0x15aa00 — the AIRoute teardown
Clears the old route's links (the `+0x41` count loop — the decompiler dropped the body), the
mode byte to 0xb, the node/link indices to 0xffff, the stamps to the new object, the flags [M
0x15aa00:6-31]. Callers: `NDrone2_DefaultInit` 0x14b300 (fresh drones), `NDrone2_ReFindMissionPath`
0x153a30, `DroneMove_FindSafetyFromScaryPosition` 0x152250 (with `InvalidateRoute`), `MiniSub_Update`
0x181f40.
- Port: the port drops the whole `AIRoute` (`croute = None`) and builds a fresh `Route` — the
  constructor's zero fields ARE the reset [S fold]. Cited at `Route::new` (nav.rs) and
  `find_safety_from_scary_object` (drone.rs, with the 0x152250:128-129 anchor that was already
  there).

### `AINetwork_UpdateLinksUsedCount` 0x15a490 — the crowd counter
From `AINetwork_FollowRoute` 0x15b110: walks the route's remaining node chain (from the current
node − 1), finds each consecutive pair's `MAPLINK` object and increments its `+6` used counter
[M 0x15a490:14-46] — so the next drone's A* costs a crowded link `len × (used + 1)`
[M `DoAStarPath` edge cost]. The port keeps the link list on the `Route` and marks from
`index − 1` [S fold].
- Port: `Route::mark_used` (nav.rs), called from the route ticks (drone.rs 4092 / 7614) into
  `Drones.links_used` — which `to_position`/`astar` read back as the `used` costs.

### `LinkCreep_CalcToRouteEnd` 0x159cc8 — the creep setup
From `AINetwork_CalcRoute` 0x156ba8: `d = route+0x40 − route+0x20` stored (`+0xc0`), `|d| > 0.4`
→ `+0xb2` = `trunc(trunc(|d|) × 2.5)` steps of `0.4/|d| × d`, else one step [M 0x159cc8:20-35];
then `LinkCreep_Dest(0, +0x60)` picks the first step point.
- Port: `Creep::new` (nav.rs) — the same 0.4 / `trunc × 2.5` math (as written), shared with
  `LinkCreep_ForNodes` 0x159810.

### `LinkCreep_Dest` 0x15b420 — the step point
`vec × (step + k)` from the route start, stored through `build_FindCel` [M 0x15b420:8-16].
Callers: `ReFindMissionPath` 0x153a30, `AssignAIPath` 0x154190, `MoveTarget_IsRouteNode`
0x1550c8, `SetupNextNode` 0x159668, `LinkCreep_ForNodes` 0x159810, `CalcToRouteEnd` 0x159cc8,
`MiniSubState_Attack` 0x181b00.
- Port: `Creep::point` (nav.rs); the `build_FindCel` half is folded — this port is one cel
  [S deviation, the `Network::build` header note].

## The vision ray and the flee pick

### `AINetwork_TestRayCels` 0x15ab78 — the AI line of sight
The segment against every boundary of the crossed AI cels (walls first, then the rest) [M
0x15ab78]. Callers: `DroneVision_CanSeeObjectFrom` 0x176f18, `DroneVision_LineOfSightToObject`
0x178390, `DroneVision_CanSeePosition` 0x178438 — every guard-vision ray.
- Port: the static collision world's `line_of_sight` replaces it — the same deviation the nav
  `move_test` documents [S]. Cited at the per-tick drone-vision LOS pass (`Drones::tick`,
  drone.rs — the `dr.los` round-robin) and at `check_unsurrender`'s LOS.

### `AINetwork_Emitter_GetNodeAtDistance` 0x159268 — the flee node
From `DroneMove_FindSafetyFromScaryPosition` 0x152250: clears the 0x40000000 visit bits over the
path's nodes, marks the nearest node visited, then walks the link lists outward, returning the
first node at or past the requested run distance [M 0x159268:60-140].
- Port: `find_safety_from_scary_object` (drone.rs) runs the straight opposite-ray goal + the
  shared MoveTest-gated route build instead [S fold, documented at the fn]. The route reset half
  is the game's `AINetwork_ResetRoute` 0x15aa00 + `InvalidateRoute` [M 0x152250:128-129].

## Tests

No behaviour changed (comment-only diff): the port code for all 8 was already on the tree and
pinned by the nav unit tests (`astar_takes_the_short_way_and_used_links_cost_more` — the used
cost the counters feed; `scary_links_flag_inside_the_radius_and_the_cycle_clears_them` +
`astar_still_walks_a_scary_flagged_link` — the link-flag system; `walls_block_moves_and_the_creep_walks_a_loop`
— the boundary intersection band and the creep's full-loop walk) and the guard AI live tests.
The full four-crate suite on this tree: **2683 passed / 11 failed / 144 ignored**, the same 11
pre-existing environmental failures as the untouched base (the disc-identity group and the
nf-view pylon probes — the image used there was not the golden build).
