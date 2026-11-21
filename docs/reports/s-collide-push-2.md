# s/collide-push-2: Bond's wall push checked against the game, spot by spot

**This branch moves the shared random stream (it builds on s/collide-05-gaps). Keep it on the branch for the batch re-pin.**

Base: s/collide-05-gaps (6acfc5587), the game's one-sided cylinder push. This branch measures the wedge spots that turned the walks red in PCSX2, ports the parts of the push that did not match, and sorts the remaining reds.

## Method

PCSX2 (GDB build), fresh level load, god mode held. Bond was placed with a raw write to obj+0x30 and yaw obj+0x54, the left stick was held at full forward through the GDB `input_set`, and obj+0x30 was read every frame over PINE. The port ran the same placement and input headless (cel re-found, 20 idle ticks, then forward). Collision data and plane records came straight from the disc.

## Spots: game vs port

| Spot | Game [L 2026-11-21] | Port on the base branch | Port on this branch |
|---|---|---|---|
| 06 shelf, held north into the balustrade post (36.0, 46.2, −63.3) | blocked, rests at z −62.6835; creeps east to (36.4856, 46.2306, −62.6467) | blocked at z −62.6792, no creep | blocked at z −62.6835, no creep |
| 06 shelf, held east into the room corner from (30, 46.2, −63.5) | rests at (36.7021, 46.3022, −63.4152) | (36.7000, 46.3012, −63.4168) | (36.7021, 46.3022, −63.4153) |
| 0a west office door, opened with the use key, held east from (−20.5, 1.2, 4.3) | wedged between the opened leaf and the jamb at (−20.2343, 1.1237, 4.4970) | old two-sided push: (−20.2329, 1.0327, 4.4834) | (−20.2343, 1.1346, 4.4970) |
| 0a, held south down the west office block from (−20.5, 1.2, 9.0) | passes the door, slides to (−23.15, 1.13, −1.09) | not measured | passes, slides to (−23.08, 1.13, −0.96) |
| 0a lobby doorway, held west from (−8.0, 1.12, 16.29) | hall line z 16.8295 | not measured | z 16.8296 |
| 16 ring ramp, held north from (14.5, 1.7, −87.5) | climbs to the ring at y 6.744, rests at (14.5530, 7.1948, −74.6588) | not measured | same climb, rests at (14.5529, 7.1946, −74.6588) |
| 41 "1-Sided Thing", held south into its back from (−3.0, 1.2, −43.5) | walks straight through to the far wall at z −52.2468 | walks through | walks through, far wall z −52.2468 |
| 41, the same wall held north into its face from (−3.0, 1.2, −46.4) | blocked at z −45.4472 | | blocked at z −45.4472 |
| 08 stair-head approach, held at yaw 2.97 from (143.46, 49.2, −98.0) | blocked at (143.45, 49.13, −99.4483) | | blocked at (143.45, 49.13, −99.4483) |
| 0c arc ledge, held at yaw 1.0 from (8.42, 12.45, −221.58) | slides along the wall at 0.027 per frame, (9.6023, 12.395, −221.505) at frame 100 | | the same slide, (9.5382, 12.3656, −221.4166) at tick 94 |

So the wedges at the three kinds of spot the task named are the game's own:
- **Narrow gap (06 sill):** the game's Bond does not pass the post at x 36 either. The route's "sill gap" leg crossed only because the old push let a jump carry him over.
- **Open-door part (0a west office door):** the opened leaf wedges the game's Bond at the same spot to 4 decimals.
- **Stair (08 stair head):** the game blocks at the same spot, and the walk's line runs into that wall. 16's "return 6" fall is gone on this branch.

## Fixes

1. **The push reads each triangle's stored plane record** [M + L]. `Intersect_CylGeom` 0x109da0 loads the plane at the leaf's words + plane·2 (disasm 0x10a1d8-0x10a1e8) and hands it to `ASH_vecutil_Dist2Tri` 0x109770, which tests and projects against it. The head-on denominator, the crossing fraction, the flags-99 behind test, the interior projection and the `dot(v, n) > 0` gate all use that record, not the vertices' winding plane. The records sit up to ~0.004 off their own vertices. Live, the 06 post rest (z −62.6835) is exactly the record plane plus 0.55; the vertex plane gives −62.6792. `Geo` now carries a world-space plane per triangle, static and dynamic. Placements are rigid, as `RotTransMatrix` builds them.
2. **Dist2Tri's own region logic** [M 0x109770]. Edge k runs v[k−1]→v[k] and tests `(p − v[k])·(e × n) > 0`. The loop leaves once the flags reach 2. Flags 3/5/6 take the vertex outright, and segment arms clamp with the game's fall-throughs. Flags 0 project on the record plane. This is not the exact closest point, for example on an obtuse vertex. Edge or vertex flags re-project p onto the spine, as before.
3. **The no-push node bit** [M]. `Collide_Intersect` 0x1ebaa8 sets HITDATA+0x51 bit 2 for a cel whose +0x94 has bit 0x40 (disasm 0x1ebaec-0x1ebb04), or for an object whose +0xF0 has it. `Intersect_CylGeom` then skips the push arm (0x10a530-0x10a538 `andi 6` / `bnez → 0x10a738`), and the hit is only recorded. The body queries now apply this to the static pass nodes and to dynamic parts with glist bit 0x40. They also skip dynamic parts with bit 0x20, as `Collide_PickObj` 0x1eae80 does for the walk's mask 0.
4. **The loop-exit flag is ported as a proof, not code** [M + D]. The status-flag exit at 0x10a0cc-0x10a100 (`vsub.x` of box max − vertex-0 x, `cfc2`/`andi 2`, branch to the next box 0x10a7d0) leaves a box's triangle loop once a triangle's vertex 0 lies past the query box's max x. On every level on the disc (29,054 leaves, 2,245,311 triangles), each leaf's triangles run in nondecreasing vertex-0 x, and vertex 0 is each triangle's smallest x. Every skipped triangle therefore fails the box test anyway, so the exit changes nothing.

## Tests

- New `collide_push_live` (NF_ISO): the post rest, the shelf corner, the opened door wedge, the ring ramp, the hall line ([L] each) and the disc-wide vertex-0 order census ([D]). All 6 pass. On the base branch the post (−62.6792) and the corner (36.7000, −63.4168) fail; the door, ramp and hall cases were not run on the base. Under `NF_CYL_OLD=1` all four hold tests fail.
- New world unit tests (nf-game lib): `dist2tri_takes_the_vertex_on_flags_3_like_the_game` and `a_pass_flagged_node_never_pushes_the_cylinder` [M], plus `the_push_rests_on_the_stored_plane`. nf-game lib: 1011 passed before the new tests; world::tests pass (the NF_ISO one needs the env).
- `walkreal_07000041_wall_live`: the north test asserted that the wall blocks from behind. The game does not, so the test now pins the walk-through [L]. 2/2 green.
- `walkreal_0700000a_live`: the old `the_west_hall_glide_height_matches_the_game` pinned y 1.1285. The live hold read 1.1244 today and the port reads 1.11-1.13. That height is the fall and ground pass, which this branch does not touch. The test is renamed `the_west_hall_walk_crosses_the_doorway`, and its hall line moved to `collide_push_live`. The ignored corner test's reason now records that the door wedge is the game's.

## Remaining reds (all walkreal binaries the base turned red, run on this branch)

Green now: 07 (6/6), 41_wall (2/2), and 16's stuck "return 6" leg (16 went from 14 red to 4).

**Route reds.** The movement matches the game; the scripted walk relied on the old push and must change.
- 06: 3 walks stop at "through the sill gap". The game blocks there too.
- 08: 4 walks stop at "the stair head" and "the stair foot". The game blocks at (143.45, 49.13, −99.4483) too.
- 0c: 2 walks wedge on the arc ledge at (9.30, 12.34, −221.09), where the game slides the same way. The walker's recovery then drops Bond off the ledge, so wires #283 and #284 are never reached.
- 12: real-damage only, stuck at the lift doors (2.53, 1.13, 14.08). Not live-checked.
- 09: `the_alarm_switch_fails_objective_1`, the switch reach is missed. Not live-checked.
- 0d: `walk_arrival_to_the_c5_door` hangs at the pulley at (179.72, 14.74, −296.02). Not live-checked.

**Fight-luck or recording re-rolls.** Leave these for the batch re-pin.
- 02: 2 real-health walks (the geisha group enable, the ch 0x81 doorway sensor).
- 06: the 00agent death.
- 0a: 2 keyboard-hatch walks, where the committed recording's tick count changed.
- 0b: the recording tick count, plus 2 deaths.
- 0d: 2 deaths.
- 11: the 00agent death.
- 15: 2 deaths, and the mine never exploded under fire.
- 16: 3 real-damage deaths, and the chaser's idle-tick punch.
- 1b: 3 walks, "console 1 latched by the real shot".

## Unproven / open

1. **Rest height.** On flat floor the game rests Bond at floor + 1.1 (06 shelf 46.1238, 0a 1.1238 / 1.1244, 41 1.1238 standing / 1.1211 walking). The port rests 0.01 higher or lower (46.1347, 1.1347, 1.1118). That comes from the fall and band pass (`Player_Collision` 0x194df0 gravity gate, `Player_FeetOnPoint` 0x194188), not the push, and is not changed here.
2. **The 06 post creep.** The game's Bond creeps east along the post (x 36.0 → 36.49 in ~100 frames, with y rising 46.20 → 46.23). The port stays put. The floor there slopes (record n (−0.193, 0.981, 0)). The cause is not found.
3. **The 16 ring west leg.** Walking west along the north wall, the game rides up onto something at x ≈ 11 → −2 (y 7.11, z −74.66). The port walks the floor at y 6.88, z −74.96. No static triangle sits there, so a game object may be missing from the port. Not settled.
4. **Triangle order.** The game walks the box tree's leaf order and the `Collide_Pick` node list. The port walks its own run order. Pushes are sequential, so a multi-face corner can differ. None of the measured spots showed it.
5. **Not live-checked:** the 09 switch reach, the 0d pulley hang, the 12 lift doors, the 1b console shot.
