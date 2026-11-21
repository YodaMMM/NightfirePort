# Driving tracks: where the player vehicle starts

Code: `crates/nf-formats/src/trackstart.rs` (`map_triggers`, `ground_height`, `track_spawn`).
Test: `crates/nf-formats/tests/geoprim_disc.rs` `every_track_starts_on_a_surface_at_its_start_mission_trigger`.
Evidence tags: [M] DRIVING.ELF decompile (function + address), [D] disc data.

## 1. The chain [M]

1. Mission setup `FUN_00176658` 0x176658 prints "Init World" and calls `FUN_0022faf8`, which passes
   the `Map` window's `Trgr` record to `FUN_0022cbe0` 0x22cbe0 (a copy of the 0x40-byte trigger
   array is kept for resets). `FUN_0022cbe0` calls `FUN_0022cd30` 0x22cd30: for every trigger whose
   action list holds id **0x16** (`FUN_0022ca18(trigger, 0x16)`), it fires the trigger at once,
   `FUN_0022c890(trigger, 1, -1, 0)`.
2. `FUN_0022c890` 0x22c890 stores the trigger pointer in `DAT_0032e738` and walks its action list
   (trigger `+0x18`): `+0` count, then 16-byte entries from `+0x10`; entry `+0` = id, entry `+8` =
   offset from the entry to the arguments. The id indexes the creator table at 0x36fda0
   (`FUN_001757b8`).
3. Creator 0x16 = `FUN_0015c920` 0x15c920: bump-allocates in the event buffer (`FUN_00175730`,
   `DAT_0032e6fc`) an `EStartMission` (vtable 0x3703c0, typeinfo name "13EStartMission" 0x3728e8)
   holding the trigger and argument word `+4` (`FUN_0015c718`).
4. The handler `FUN_0015c738` 0x15c738: position = trigger `+0x00..+0x0b`, direction = trigger
   `+0x30..+0x3b`; `FUN_001fee68(0x339190, pos, dir)`; car velocity = normalised direction ×
   argument × car `+0x4c`; `FUN_00167d80()+4 = 1`.
5. `FUN_001fee68` 0x1fee68: y = ground height under (x, y, z) + 1.0 (`FUN_002151b8`; when it finds
   nothing, 0 + 1.0); x and z stay; the matrix comes from the direction (`FUN_00210540`: row 2 = the
   normalised direction, the other rows from world up; row 3 = (0, 0, 0, 1)).

The ground query (`FUN_002151b8` → `FUN_002322c8` → `FUN_002324e8`) keeps, among the triangles
gathered at the point (`FUN_00217588`), the one with the smallest `(y + 0.5) − lowest corner y`
that is > 0 (`FUN_00214338` / `FUN_00213ff8`, triangle test `FUN_002138b0`, which skips class bit
0x04); the height is that triangle's plane at (x, z) (`FUN_00231a90`).

"Spawn player car" in the same setup first places the car at (0, ground(0, 0, 0) + 1, 0) with the
direction constant (0, 0, 1, 0) at 0x376db0 (`FUN_001ff850`). The `EStartMission` event was queued
during "Init World", before that spawn, so it moves the car afterwards; the queue's dispatch loop
is not traced. The restart path `FUN_001566b0` re-spawns at (0, 0, 0).

The query in detail, as ported in `trackstart::game_ground` [M]:

1. Gather (`FUN_00217588` → `FUN_00217228`, radius 0, planar): a `CDat ci` item is kept when
   `max(|dx|, |dz|) + min(|dx|, |dz|)/4` from its centre `(−t·r0, −t·r2)` (`FUN_00212da8`) to the
   point is below its `+0x3c` radius, and (`FUN_00216ec0`) when one of its `ca` groups has its
   centre within `+0x0c u16 / 16` of the local point in xz and holds a triangle whose centroid is
   within that triangle's own `+0x2e u16 / 16`.
2. Per kept item (`FUN_00214338`): the point in the item frame (`FUN_00212ed0` + `FUN_0025b890`)
   must lie within the `+0x0c` / `+0x2c` half extents. Per kept group, only the **first**
   triangle of the strip that holds the point in plan view counts (`FUN_002138b0`). Distance =
   `(local y + 0.5) − lowest corner`, or with item flag bit 0 `lowest corner − (local y − 0.5)`.
   The smallest distance > 0 over all items wins (`FUN_002324e8`).
3. Height = the world plane of that triangle at (x, z), normal `(v1 − v0) × (v0 − v2)` turned up,
   n.y capped at 0.9999 (`FUN_002151b8`, `FUN_00231a90`).

`track_spawn` uses it. On all 8 tracks it gives the same ground as the older all-triangle
reading (test `the_traced_ground_query_agrees_at_every_start_and_each_tracks_sky_is_listed`).

## 2. On the disc [D]

The trigger `+0x18` word is `0x656c` (`el`) + an index into the `Map` window's `el  #N` records,
whose payload is the action list. All 8 tracks: exactly one trigger carries id 0x16.

| track | trigger | marker (x, y, z) | ground | yaw (deg, 0 = +z) | argument +4 |
|---|---|---|---|---|---|
| MIS01 | 684 of 754 | (779.07, −7.60, 206.95) | −7.60 | −46 | 0 |
| MIS11 | 29 of 226 | (71.29, 17.66, 40.96) | 1.18 | 25 | 0 |
| MIS13A | 28 of 128 | (0.24, 0.00, −50.53) | 0.00 | 0 | 35 |
| MIS13B | 67 of 128 | (−95.19, −16.94, −292.87) | −185.81 | 137 | 2 |
| MIS13C | 16 of 23 | (−2265.85, −582.44, −2330.18) | −588.22 | 59 | 0 |
| MIS3 | 214 of 297 | (223.63, 10.91, −411.06) | 10.61 | 62 | 0 |
| MIS4 | 252 of 324 | (0.09, −0.97, 155.60) | −0.97 | 180 | 24 |
| RACE | 212 of 232 | (−2.48, −0.39, 168.78) | −0.39 | 178 | 0 |

On every track the decoded render geometry (geoprim.md) has a surface within 2 units under the
car as well. MIS13C's trigger 0 (the previous stand-in) lies 43 units from the start trigger.

## 3. Not proven

- The order "event queued at Init World, run after Spawn player car" (the dispatch loop is not read).
- `game_ground` tests every `ci` item instead of only those listed in the grid cells
  (`FUN_0021cf30`), and takes the world's class mask (`DAT_0033a540 + 0x2c`) as 0.
- MIS13B: the marker sits at y −16.9. Nothing lies near it: the only surface under it, in both the
  collision world and the render geometry, is the valley terrain `Vol_foothillE` at −185.8, and
  the traced query puts the car there (+1.0). A trigger 1.2 units from the marker (a sphere of
  radius 2.5 at y −17.6) holds the intro actions `EControlToSpline` (202), `EPlayCameraAnim` (63),
  `ELetterBoxOn` (104), `ESetAutoDriveCamOn` (77). A car standing on the valley floor 168 units
  below would never enter that sphere, so the start as traced cannot be the whole story. Open:
  whether `Vol_foothillE`'s collision is live at that moment (the gather skips items whose `+0x1c`
  pointer is 0), the event dispatch order, and what `EControlToSpline` does with the car. Only a
  live read of the car position in the first frames of MIS13B can settle this.
- `ground_height` tests every collision triangle instead of the grid cells `FUN_00217588`
  gathers, and ignores the instance flag bit 0 that makes the query measure upward.
- MIS13B's marker sits at y −16.9 while the ground found under it is 169 units lower (−185.8):
  the code puts the car on that ground; whether the ultralight then behaves differently is not traced.
  The trigger beside it (78, radius 2.5, 1.4 u from the marker) and the path it hands the car
  (`path_BondMaster01`, first key 1.2 u from the marker) are both at the marker's height, so a
  car on the found ground would never start the opening (driving-intros.md section 4).
- The meaning of the argument beyond "× car `+0x4c` = start speed" (the `+0x4c` field is not named).
