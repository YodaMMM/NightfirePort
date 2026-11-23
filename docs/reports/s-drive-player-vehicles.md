# s/drive-player-vehicles: the player's vehicle per mission and phase, the helicopter's flight

No change to the shared random stream (nothing here draws from it). No view change.

## What was done

- `crates/nf-formats/src/drive_vehicle.rs` (new): the game's own player-vehicle choice.
  - `mset_vehicle` / `spawn_vehicle`: the `MSet` name, the `db5` / `z8` unlock swap and the
    `jungle_truck` low-memory override [M `FUN_00204d18`, `FUN_002051f8`, `FUN_00176658`].
  - `secondary_type`: the `.atr` `SECONDARY_TYPE` the transition rebuilds the car as [M
    `FUN_001862c0` :133, `FUN_001932d0`].
  - `vehicle_event`, `PlayerVehicle::apply`, `vehicle_lists`, `mission_vehicles`: the
    per-phase state as the script runs: `EControlToSpline` / `EEndControlToSpline` (who
    drives) and `ECustomTransition("paris")`, the only swap [M `FUN_0014ae60`, `FUN_0014c110`,
    `FUN_00131508`, `FUN_00131590`, `FUN_0014b020` → vtable +0x8c `FUN_00185e50`].
- `crates/nf-game/src/driving/path_drive.rs` (new): the helicopter's flight model.
  - `path_driven` / `player_pass`: the `FUN_00188200` path test (driver spline controller,
    health > 0) in front of the existing `drive_pass`.
  - `place_on_path`: `FUN_00131620` + `FUN_001f7880` (the body takes the path pose, the
    follower's velocity and velocity × mass momentum; angular terms zeroed).
  - `PathDrive::step`: the path pass `FUN_00192498` (speed, wheel spin, roll phases, skid and
    slip flags, steer, per-wheel ground depth and spring length), with `surface_bump`
    (`FUN_00213618`) and `wheel_depth` (`FUN_001f6ae0`'s out w).
  - `HeliFlight`: follower update → pose copy → path pass, one tick.
- Docs: docs/systems/drive-player-vehicles.md (new); driving-mis01-opening.md §4.5 / §11 (the
  swap is now [M]); drive-suspension.md and suspension.rs (the vtable +0x1d4 getter is
  `FUN_00193170`, the attr block: was [S]).

## Findings

- Spawn vehicles [M + D]: MIS01 helicopter, MIS3 supersnow, MIS4 vanquishalps, MIS11
  vanquishsub, MIS13A jungle_truck, MIS13B ultralight, MIS13C fodbase, RACE cobra_player.
- Only MIS01 changes the player's vehicle: lists el742 and el789 hold `ECustomTransition("paris")`,
  and only `helicopter.atr` has a `SECONDARY_TYPE` (vanquish). MIS13A and MIS13C never move
  the player into another vehicle; their spline lists (13A el46 / el47, 13C el18) only hand the
  driving to a path.
- The helicopter has no flight physics of its own. Its `.atr` is a land car's tuning plus
  `IS_HELICOPTER`, which the player's car never reads. On the spline it takes the path pass and
  its pose is the keyed track; off the spline it would take the player land pass
  (`physics::Car::step`). There is no free flight.
- A spline-driven car has driver index 1 (`FUN_00131508` → `FUN_00193000`), so at zero health
  on a spline it would fall to the AI land pass, not the player one [M].
- The view's hand table differs from the game on MIS01, MIS4, MIS13A and MIS13C (not changed:
  view code is out of scope here).

## Tests

- `cargo test -p nf-game --test drive_path_drive`: 7 passed.
- `cargo test -p nf-game --test drive_player_vehicles_disc` (NF_ISO): 2 passed. Every mission's
  spawn vehicle, vehicle set, spline / transition lists and pass; the MIS01 helicopter on
  `pt 0001` at throttle 0.71 for 600 ticks (body on the follower each tick, momentum = v × 1660,
  speed 23.93 m/s, 13 skid ticks, six pinned positions and steer values), then list 789 → the
  Vanquish under the player land pass.
- `cargo test -p nf-formats --lib drive_vehicle`: 5 passed.
- Unchanged neighbours: `drive_sub_physics` 12 passed, `driving_sniper_live` 9 passed.
- Mutation check: 16 rules or constants changed one at a time (the 6.32 divisor, both skid
  thresholds, the 0.01 spin floor, the health gate, the 0.015 bump, the boat +10, the position
  row, the `<= 0` slip clear, the snowmobile slip zeroing, the `paris` check, the same-name
  check, the end-spline player check, the `z8` swap): all turn a binary red. The front-skid
  threshold and the end-spline player check stayed green at first; tests were added (74 / 76
  degree headings, an AI-element event) and both re-run red.

## Not proven

- The writers of `DAT_0033a010..020` and 0x1009a1 bit 0 (spawn overrides): assumed off.
- That the run-time string skips the `sr` `CHAR::` prefix (stripped, as `pov.rs` does).
- The car rebuild `FUN_00185e50` is not ported (only the new name is given).
- The helicopter's body box (LOD-0 geometry stands in for the model header box), the port's
  ground grid versus `FUN_002322c8`, and the query's retry arm: spring lengths over the river
  are not pinned.
- The order of follower update, pose copy and path pass in a frame; the quaternion round trip.
- What reads the helicopter's wheel spin and roll phases.
