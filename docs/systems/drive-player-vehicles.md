# Driving missions: the player's vehicle, and the helicopter's flight

Which vehicle the player is in, per mission and per scripted phase, and what moves the MIS01
helicopter. Evidence: [M] DRIVING.ELF decompile (EE addresses), [D] disc bytes, [S] suggested.
Port: `nf_formats::drive_vehicle` (the choice), `nf_game::driving::path_drive` (the pass
choice, the spline-driven body and the path pass). Tests: `crates/nf-game/tests/drive_path_drive.rs`
(unit), `crates/nf-game/tests/drive_player_vehicles_disc.rs` (needs `NF_ISO`).

## 1. The spawn vehicle

- `FUN_0022faf8` hands the `MSet` record to `FUN_00205220` (payload pointer at mission+0x1c);
  `FUN_00204d18` copies it to mission+0x1b44 as the vehicle name (the payload opens with a C
  string). Only `db5` / `z8` are replaced, by the first set flag of `DAT_0033a010..020`:
  lotus, playervan, playerlimo, playertaxi, playersmart [M]. No DRIVING.ELF code writes those
  flags.
- "Spawn player car" `FUN_00176658` asks `FUN_002051f8`: mission+0x1b44, or `jungle_truck`
  when low-memory byte 0x1009a1 bit 0 is set [M]. That byte's writer (outside DRIVING.ELF) is
  not traced.
- The car ctor `FUN_001862c0` would take `DAT_0033196c` instead for the player; nothing writes
  it [M].

On the disc [D]:

| archive | track | spawn vehicle | pass off a spline |
|---|---|---|---|
| MIS01 | paris_mis01 | helicopter | player land (`FUN_0018cef8`) |
| MIS3 | snow1a_mis3 | supersnow | snowmobile (`FUN_00190510`) |
| MIS4 | snow2a_mis4 | vanquishalps | player land |
| MIS11 | uw_mis11 | vanquishsub | SUB_PHYSICS (`FUN_00190f88`) |
| MIS13A | junglea_mis13a | jungle_truck | player land |
| MIS13B | jungleb_mis13b | ultralight | SUB_PHYSICS |
| MIS13C | junglec_mis13c | fodbase | player land |
| RACE | snow2a_race | cobra_player | player land |

The view's hand table (`player_car_stem`, crates/nf-view/src/driving.rs) differs on MIS01
(vanquish), MIS4 (vanquish), MIS13A (ultralight) and MIS13C (patrolboat); RACE picks
paradis_car, which is cobra_player's `RENDER_FILENAME` (the same model under another name);
MIS3, MIS11 and MIS13B match. Switching the view is a view change and is not made here.

## 2. Phases: who drives, and the one swap

- `EControlToSpline` (202, `FUN_0014ae60`, word 1 == 0 = the player): the player car's driver
  (car+0x2e4, vtable +0x5c `FUN_00192fe8`) gets the spline through `FUN_00131508`: car+0x2c8
  (the driver index) = 1, inputs cleared (`FUN_001872d8`), driver state 3, a spline controller
  at driver+0x50 [M].
- `EEndControlToSpline` (239, `FUN_0014c110`): `FUN_00131590`: car+0x2c8 = 0, inputs cleared,
  driver state 1, the controller freed (+0x50 = 0) [M].
- `ECustomTransition` (240, `FUN_0014b020`) acts only on the string `paris` (0x36f2d0): vtable
  +0x294 = `FUN_001932d0` returns car+0x450, which the ctor filled from the `.atr` key
  `SECONDARY_TYPE` (0x379a88, `FUN_001862c0` :133); vtable +0x8c = `FUN_00185e50` rebuilds the
  car in place under that name (attrs, model, engine sound, body) when it differs from car+0x2cc
  [M]. In the whole decompile this is the only call of that slot on the player car [M grep];
  `EControlToCPU` / `EControlToPlayer` change the driver, not the vehicle.

Per mission [D]: the lists holding a player-vehicle event.

| archive | to spline | end spline | `ECustomTransition` | vehicles |
|---|---|---|---|---|
| MIS01 | el686 (AISp 0) | el742, el776, el789 | el742, el789 (`paris`) | helicopter → vanquish |
| MIS3 | el222, el256, el270, el288 | el108, el119, el259, el289, el296 | none | supersnow |
| MIS4 | el268 | none | none | vanquishalps |
| MIS11 | el34 | none | none | vanquishsub |
| MIS13A | el46 | el47 | none | jungle_truck |
| MIS13B | el79 | none | none | ultralight |
| MIS13C | el18 | none | none | fodbase |
| RACE | el225 | none | none | cobra_player |

Only `helicopter.atr` carries a `SECONDARY_TYPE` among the archives' pvehicle files [D]. So
13A and 13C never move the player into another vehicle; their spline lists only hand the
driving to a path and back.

`nf_formats::drive_vehicle::{mset_vehicle, spawn_vehicle, vehicle_event, PlayerVehicle,
vehicle_lists, mission_vehicles}` is the call the view can make: the spawn name from the
`MSet` chunk, then `PlayerVehicle::apply` per event as the script runs.

## 3. The helicopter's flight model

- `helicopter.atr` [D MIS01.VIV] is a land car's tuning (the Vanquish's with small changes:
  MASS 1660, MAXACC 2.65, MAXSTEERING 0.077, WHEEL_RADIUS 0.315, SPRING_REST_LENGTH 0.35,
  CAR_WHEEL_X/ZF/ZR 0.13 / -0.8 / 1.17) plus `SECONDARY_TYPE.s=vanquish`, `NO_SHADOW.b=true`,
  `IS_HELICOPTER.b=true`. No `SUB_PHYSICS`, no `IS_SNOWMOBILE`.
- `IS_HELICOPTER` (0x379aa0) is read only in the car ctor for AI cars (param 2 != 0 → car+0x37c
  = 0) [M `FUN_001862c0` :188-207]; the player's is never read. The names `helicopter` /
  `helivanquish` pick the engine sound object (`FUN_00186878`) [M].
- **The pass** (`FUN_00188200` 0x188b70-0x188c60) [M]: when the driver's spline controller is
  set and the health (`FUN_00195a30`: *car+0x54, 1.0 without one) is above 0, the path pass
  `FUN_00192498`; otherwise the attr pass: SUB_PHYSICS, IS_SNOWMOBILE (through vtable +0x1d4 =
  `FUN_00193170` = car+0x23c, the attr block), then car+0x2c8 == 0 → `FUN_0018cef8`, else
  `FUN_0018ecb8`. The helicopter on its spline takes the path pass; off it, the player land
  pass (there is no free-flight model). Because `EControlToSpline` leaves car+0x2c8 = 1, a
  spline-driven player car at zero health would get the AI land pass.
- **The pose** [M]: the driver update `FUN_0012e350`, state 3, runs `FUN_00131620`: the body
  is placed by `FUN_001f7880` (velocities and momenta zeroed, position = the follower's target
  row 3, orientation = the target's rotation × the controller's offset matrix +0x70 through a
  quaternion), then velocity (+0x20) = the follower's +0x88 and momentum (+0x40) = that × mass.
  The path itself is `flight::PathFollower` (driving-mis01-opening.md §6).
- **The path pass** `FUN_00192498` [M]: returns at once for SUB_PHYSICS. Speed +0x288 = |v|;
  wheel spin +0x274 = |v.xz| / (WHEEL_RADIUS · 6.32 · fps), 0 under 0.01, at most 1; roll phases
  +0x230 / +0x234 += spin, wrapped into [0, 1]; c = forward row · v̂: below 0.707 the skid
  flag +0x43e and the rear slip flags (+0x228, +0x22c) = 1, below 0.25 also the front ones;
  steer +0x28c = clamp(right row · v̂, ±1). Unless IS_SNOWMOBILE, per wheel: the world point
  (the body rows × the wheel point + position), the ground query `FUN_001f6ae0` (depth
  `(v0 − p) · n` + the surface bump `FUN_00213618` + 10 for IS_BOAT; no triangle: normal up,
  depth 0), spring +0x210 = clamp(depth + SPRING_REST_LENGTH, 0, SPRING_COMPRESSION_LIMIT), a
  0 spring clearing that wheel's slip flag. IS_SNOWMOBILE zeroes springs and slip flags.
- The surface bump [M `FUN_00213618`]: surfaces 2/3/5/10 `0.01 · (sin(4π z) + cos(4π x))`,
  4/7 the same at 0.015, 6 a flat −10, others 0 (`FUN_002bfc38` = sinf, `FUN_002bfa48` = cosf:
  the fdlibm quadrant switch onto `__kernel_sin` / `__kernel_cos`).

The disc run (`the_mis01_helicopter_flies_its_opening_path`): list 686 puts the helicopter on
`pt 0001` at throttle 0.71; over 600 ticks the body sits exactly on the follower's pose, carries
its velocity and 1660 × that momentum; the speed holds about 23.93 m/s, the wheel spin 0.2003,
the heading leaves the velocity by more than 45 degrees on 13 ticks; list 789 then leaves the
player in the Vanquish under the player land pass.

## 4. Not proven / not ported

- The writers of `DAT_0033a010..020` and of 0x1009a1 bit 0 (the spawn overrides): all off is
  assumed for a normal run.
- That the `sr` string's `CHAR::` prefix is skipped by the run-time pointer (the reference
  loader is not traced): the port strips it, as `pov.rs` does.
- The car rebuild `FUN_00185e50` itself (model, sound, body re-creation) is not ported; the
  port only names the new vehicle.
- The helicopter's body box: the game reads the model header (`FUN_001c9778`); the disc test
  uses the LOD-0 geometry box [S]. The ground query is the port's `Ground` grid, not the game's
  `FUN_002322c8` cell walk, and the query's retry arm (x + 0.1, twice) is not ported, so the
  spring lengths over the river are not pinned.
- The order of the follower update, the pose copy and the path pass inside a frame [S]; the
  quaternion round trip of the orientation is taken as exact.
- What reads +0x274 / +0x230 / +0x234 on the helicopter (a rotor or wheel animation) is not
  traced.
