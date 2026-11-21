# s/drive-jungle-vehicles: the spawn vehicles' models, sheets, lists, moving parts and passes

No change moves the shared random stream. No view code changed (crates/nf-view/src untouched).

## What changed

- `nf_formats::carpose`:
  - `facing_matrix`: the camera-facing item kind 0xee, `FUN_001bfc20` [M, disassembled].
    The item turns about +y until its +z points away from the camera, stays level, and rocks
    ±0.05 turn on a triangle wave of `65535 / v` seconds. `part_matrix` now draws it; it used to
    return `Unported`.
  - `keyframe_matrix` + `KeyTrack`: the keyframe branch of `FUN_001a30c0` for kinds below 100
    [M]. `part_matrix` gives those kinds their own matrix when the `as` set has no animation
    reference, as the game does.
  - `quat_rows` (`FUN_002cc0c8`), `nlerp` (`FUN_002cc018`), `rot_z_turns` (`FUN_002cc610`).
  - `CarPose` gains `inv_fps` (`DAT_00333ff4`), `car_pos` and `camera`, the three inputs of 0xee.
- `nf_formats::carmesh`: `CarPart::anim` (the `as` payload's +0x08 word) and `set_anim`.
- docs/formats/car-model.md §11.2 (the whole kind table) and a new §11.6.

## Findings [D unless tagged]

- All 63 car files of the nine archives decode. Each `ns#0` names `<stem>.ssh`, and that sheet is
  always in the same archive. Every strip texture resolves by name in it, except `blck` on the
  three ultralight models. The helicopter uses a single 256 × 128 `$$00` atlas, and its UVs are
  already atlas coordinates.
- Run-time kinds on the disc: 0xff, 0xfe, 0xfc, 0xfb, 0xef, 0xf3 and 0xee. None is below 100, so
  the keyframe branch never runs on this data. The `as +0x08` word is an `an  #N` reference on
  648 items, all of kinds 100 and up.
- MIS01 helicopter, the player's vehicle [M FUN_00188200 + D]:
  - vd lists 1-4 hold only items 8 and 9, which have no model. Driven by the player outside
    AutoDrive, it draws nothing.
  - The sniper phase's first list turns AutoDrive on, and there is no vd#5, so all 10 items
    draw during the phase.
  - Items 1 and 2 (0xfc, v 4864 / 1792) spin 19° and 7° per tick about their own +y. Item 6
    (0xee) faces the camera. Item 7 (0xf3) keeps its own matrix.
- MIS13A jungle_truck (the same file as MISC's): 36 items, only vd#2. It draws every item but
  the hidden 24; an interior view drops the two `rearlicense` items. Pass: player land off a
  spline, path on one.
- MIS13C fodbase: one item, one 3-vertex strip spanning 0.00004 units, texture `whit`. The
  vehicle the game spawns there shows nothing. Pass: player land.
- `.atr`: every pvehicle file but `default.atr` names a model in its own archive. Boolean values
  appear only on IS_HELICOPTER, NO_SHADOW and TYRES_INVUNERABLE, which no pass reads. The full
  per-file pass table is pinned in `crates/nf-game/tests/drive_jungle_vehicles_disc.rs`.

## For the view (not made: view changes are out of this branch)

- A view that draws the helicopter must set `CarPose::camera` and `car_pos` each frame. With the
  defaults (origin), the 0xee item turns towards the world origin. The view currently loads the
  Vanquish on MIS01, so nothing changes on screen now.
- The view's hand table (`player_car_stem`) still differs from the `MSet` spawn vehicles on
  MIS01, MIS4, MIS13A and MIS13C.

## Tests

- `cargo test -p nf-formats --lib`: 182 passed. New unit tests: the 0xee facing and rock,
  pinned at frames 0 / 15 / 30 / 45 and at 50 Hz; the keyframe blend, the mirror and the clamp.
- `carpose_vehicles_disc` (new, 3 tests): passed. `carpose_disc`: passed. Its `unported`
  count is now 0; with the 0xee arm reverted it fails. `carmesh_disc` and `carshade_disc`:
  passed.
- `nf-game drive_jungle_vehicles_disc` (new, 2 tests): passed. `drive_sub_physics_disc`: passed.
- `nf-view driving_wheels_live`: passed (builds against the new `CarPose`).

## Not proven

- The `an` / `qa` records and the hinge code `FUN_001cc6b0` / `FUN_001cc738` that may read them
  are not decoded or ported, so hood, door and trunk hinge matrices stay identity.
- What the helicopter's five `$$00` descriptors with a sub-rectangle and +0x1c = 1 do on the GS,
  given strips whose v runs past 1.
- The VU's rsqrt of 0, for a camera exactly above a 0xee item.
- The unused table kinds (0x65, 0x66-0xec, 0xed, 0xf0-0xf2, 0xf4-0xf7, 0xf9, 0xfa, 0xfd) are
  not ported.
- That the MIS13C player really sees no vehicle: the model is a speck, but what the camera and
  the riders show there is not checked.
