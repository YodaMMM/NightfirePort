# Report: placement-matrix-classes

Branch `s/placement-matrix-classes`. Every placement class now builds its object matrix with the builder its game creator actually uses: the dynamic dispatcher cases through `View_RotTransMatrix` 0x1e5168, the 0x8000 static path and the pinned exceptions through `RotMatrix` 0x1e2368, yaw-only rows unchanged to within one cosine ulp.

## What the game does (the [M] catalog)

`parsemap_create_dynamic_objects` 0x1d04f0 dispatches on the placement's flags low byte. Every creator that reaches `control_create_object` 0x1344e0 passes the quaternion argument as a literal 0 — all 91 `control_create_object` call sites in ACTION.ELF do — so the object matrix is always `View_RotTransMatrix(obj+0x50 rot, obj+0x30 pos, obj+0x90)` [M 0x1344e0:26-29]. The two builders share the same six PS2Sinf3 values but compose differently: `View_RotTransMatrix` is the Y·Z·X reading, `RotMatrix` (0x1e2368) is X·Y·Z [M 0x1e5168:33-48 vs 0x1e2368:28-40]. They agree only for rx = 0 (and, to the ulp, for rx = rz = 0).

Classes that do NOT use `View_RotTransMatrix`, each pinned:
- **0x8000 create-cel rows never reach the dispatcher.** `parsemap_block_map_data_static` 0x1d1998 stores the vectors raw and `parseentity_transform_bounding_box` 0x1d0fb0:28-29 builds the world bbox with `RotMatrix` + `Mat_SetTrans` — the port's statics were already right.
- 0x26 `MP_RegisterMPObject` 0x185990:64-65 builds with `RotMatrix` + `Mat_SetTrans`.
- 0x2a `View_AddSkyObj` 0x1e79a0 stores the vectors; `View_DrawSky` 0x1e5ad0:184,194 applies `RotMatrix` at draw time.
- 0x0f `Drone_Create` 0x136be0 and 0xf1 `DroneSpawner_Create` 0x1378d0 pass NULL pos/rot — the object matrix is the identity; the placement vectors live in the ext.
- 0x34 `GunImp_Create` 0x131f20:66-69 and the 0xf2 emitter case (built inline in the dispatcher, 0x1d04f0:474-478) use `Quat_QuatToMat` 0x1e3e80 on the placement's +0x24 **quaternion**.
- 0x24/0x25/0x4e/0xe5/0xe6/0xe9/0xdf build no object matrix at all; unlisted ids fall to the dispatcher's default (no creator).
- 0xdb doors and 0x2e framed locks overwrite obj+0x90 from `Spline_Interp` path frames after creation [M 0x136100:83-91, 0x135440:53-58]; the View matrix is the pre-spline seed.

**Correction to the task brief, from the decompile:** doors are class 0xdb (`Door_Create`), ladders 0x22 (`Ladder_Create`), sky objects 0x2a (`View_AddSkyObj`) — "doors 0x2a/0x22" does not match the switch. `control_create_object`'s second arm is `Quat_QuatToMat` + `Mat_SetTrans`, not `RotMatrix` (and has no live caller).

## What changed in the port

- `nf-formats/src/placement.rs`: new `object_matrix_for_class(class, creates_cel, rot, trans)` — the per-class predicate with the full per-case catalog in its doc comment. The pre-existing `view_rot_trans_matrix` (from level-14-vent-grates) and `object_matrix` are unchanged.
- `nf-formats/src/level.rs`: `instances()` replaces the 0xd9-only switch with the predicate; `sky_instances()` keeps `RotMatrix` for 0x2a and uses View for 0xd4 clouds [M 0x18d620:13]. `character_instances()` (0xf drones) keeps the RotMatrix reading — documented as a port reconstruction (the game builds identity there; the class's disc rows are all yaw-only).
- `nf-game`: every class-specific matrix site switched to View with its [M] cite — triggers.rs (Trigger_Create), breakable.rs + props.rs `bounds` (Break_Create), sensor.rs (Sensor_Create), useobj.rs (PCQ sphere), worldobj.rs (Shooter_Create, DynamicObject_Create), drone.rs (Mine_Create, the fusebox/switch/GT script-player mounts), copter.rs, searchlight.rs (Searchlight_Create main matrix), pickup.rs (Pickup_Create — the `Mat_Copy` alternate is dead from the dispatcher [M 0x1d04f0:459-462]), hurt.rs, door.rs (pre-spline seed).
- Known gap left open: the 0xf2 emitter's quaternion path (the port still reads Euler, `world_fx.rs`, noted inline with the [M] cite; 16/52 of its disc rows carry pitch/roll). GunImp 0x34 is the same quaternion path; its 37 disc rows are all yaw-only.

## Census [D]

`crates/nf-formats/tests/placement_rot_census.rs` walks every level `.bin` on the disc (99 levels, 2,230 maps) and counts rows with non-zero rotation-x / rotation-z per class. Reach of the switch (rx≠0/rz≠0 counts): sensors 0x28 29/25 of 62, coronas 0xfc 57/36 of 113, pickups 0xf0 41/42 of 570, SS 0xe2 20/21 of 110, grapple 0x3a 19/19 of 57, dynobj 0x3b 23/23 of 75, emitters 0xf2 16/52 of 69, wire 0x3e 4/7, rotors 0x71 3/2, touches 0xea 12/7, mines 0xfe 7/8, fusebox 0x31 4/4 of 10,430, statics 0x19 42/42 of 4,766 (staying on RotMatrix), doors 0xdb 0 of 447 (all yaw-only), drones 0x0f 0 of 588.

## Verification

- Unit tests (`placement.rs`): the View rows match the decompile's twelve stores entry-for-entry against a closed form built from the game's own `sinf` [M 0x1e5168:37-48]; `object_matrix` matches RotMatrix's nine stores [M 0x1e2368:32-40]; the two differ for a pitched+yawed+rolled rotation; the class predicate pins 19 View classes and 12 RotMatrix/exception classes; a creates_cel row of a View class stays on RotMatrix.
- Yaw-only rows: bit-equal on every matrix entry except [0][2]/[2][0] (one cosine ulp: 1.0000001 vs 1.0000002 factor) and [1][0] (+0.0 vs −0.0); transformed geometry moves ≤ 10⁻⁷ per unit — the scale the vent-grates swap already accepted battery-wide. Pinned in `yaw_only_rows_agree_between_the_builders_to_the_cosine_ulp`.
- `nf-game`: 95 test binaries, **933 passed / 0 failed / 8 ignored** (`cargo test --offline --release -p nf-game -- --test-threads=1`).
- `nf-formats`: 91 lib tests + census, all green (bigf's `disc_golden` failure is one of the 3 allowed on the slim ISO).
- 47 placement-sensitive `nf-view` binaries via `tools/test/suite.py`: **96 passed / 0 failed / 34 ignored** (probe14 grates/mouths/grapple, doors, sensors, pickups, dynobjs, searchlight, worldobj, baked statics, census_0700004a, models_world, portals, rotors, copters, ladders, hurt, wire, sky, gunimp).
- Full-suite total line: the sandbox restarted twice mid-run and the 855-binary suite needs more wall-clock than the remaining window allowed; the per-binary chunks that ran are the ones above (all green). The next session should run `tools/test/suite.py` once on this tree for the total line.

## What's left

- One full-suite total line (see above).
- The 0xf2 (and 0x34) `Quat_QuatToMat` path — build the matrix from the placement's +0x24 quaternion [M 0x1d04f0:474-478, 0x131f20:66-69] instead of the Euler fallback.
- Door/lock final matrices come from `Spline_Interp` over path frames; the port models doors' keys separately — a cross-check of the pre-spline seed vs the spline result on a pitched door row is impossible on this disc (0xdb is all yaw-only).

## Live checks for the main agent

1. A pitched sensor (class 0x28, e.g. 07000014's ceiling sensor rows with rx≈±π/2 [D census]) should sweep its beam in the wall plane, not through it — the View builder's Y·Z·X order is what the game's Sensor_Create installs.
2. Corona sprites (0xfc, 57/36 pitched rows) should sit where the wall/ceiling lights are; under the old RotMatrix reading a pitched corona drifts by up to the model radius.
3. Placed pickups on sloped surfaces (0xf0, 41/42) should keep their orientation; the switch changes their item-sphere centres by ~1e-7 units — invisible, but the collision tris for grapple anchors (0x3a, 19/19) move by up to a unit and should be spot-checked against the wall.
4. Yaw-only scenes must render pixel-identical to before (all 0xdb doors, 588 drone rows, 4,724 of 4,766 statics) — the ulp shifts are below the projection quantization.

## Usage

not visible to me
