# s/heli-rotors: every drawn vehicle poses its own moving parts

No change to the random streams (on foot or driving): the posing reads the cars' state and
writes palette matrices only.

## What a player would notice

- Enemy helicopters (MIS3 openheli, MIS4 subcopter, MIS13A jungleheli) now show their main rotor
  disc and it turns; the tail rotor turns too. Before, the main rotor blades were not drawn at
  all and the tail rotor stood still.
- Traffic cars and the hostile cars and snowmobiles roll and steer their wheels as they drive.
- The submarines' and ultralights' propellers (0xfb) turn.

## What changed

`crates/nf-view/src/driving.rs`:

- `part_is_live`: the parts the view poses per frame are now the `Skel` items (wheels, hubs),
  the rotors (0xfc / 0xfb, `R(v/256 · frame °)` [M FUN_001bf3d8 / FUN_001bf458]) and the
  camera-facing item (0xee [M FUN_001bfc20]), the last three only without flag 0x10.
- Traffic cars (`DrivingState::ai_rigs`, parallel to `ai_cars`) and each hostile palette slot
  (`HostilePool::rigs`, parallel to `slots`) load with `posed` and get their own `CarRig`
  (`vehicle_rig`). `CarRig::tick_ai` ticks the rig from that car: spin from its forward speed,
  steer from its +0x28c angle over `max(maxsteering, 0.2)`, springs from its spring state when
  it runs the land pass on the track, else at the springs' rest length (the hostile cars run
  the flat model at their rest height).
- `CarRig::write_parts` writes every posed part (`part-local · car matrix`) with no allocation:
  `Skel` parts from the rig's pose, rotors at the sim tick count, the 0xee item turned towards
  the camera from the car's row 3. The player car uses the same call.
- `apply_to_palette` takes the chase camera as the 0xee eye; `apply_to_palette_eye` takes one
  (the headless `--cam`). A free hostile slot folds its parts away with its body.
- The hostile body offset (`HostilePool::centre`) is the lowest vertex of the body and its
  posed parts at rest (wheels at the springs' rest length), as the bake was before.
- Strips of a spun part (0xfc / 0xfb) triangulate without the 3.5-unit edge cut: the rotor
  blades reach 5.57 from the hub [D openheli.crp], so every blade triangle was being dropped.
  The decoded strip list already splits the strips (nf_formats::carmesh).

`crates/nf-view/src/lib.rs` (headless driving `--shot` only): `--drive-near-element NAME`
releases the first AIEl element of that vehicle name and holds the car 40 units beside its
spawn point; the 0xee eye is the `--cam` position.

## Tests

- `crates/nf-view/tests/driving_vehicle_parts_live.rs` (new, NF_ISO):
  - `driving_vehicle_parts_live`: all eight driving tracks for 2400 ticks with a scripted
    throttle, then MIS3 / MIS4 / MIS11 / MIS13A staged (the first helicopter element released,
    the car held beside it, 300 ticks). Every tick a drawn hostile's rotor turns by exactly
    `s16(v)/256` degrees about its axis between consecutive ticks (openheli, subcopter,
    jungleheli 897 checks each, alpha_sub 1497, ultralightbig 4970); a moving traffic car's
    wheel matrices change (MIS01 3723 ... RACE 8458 car-ticks); moving hostile cars' wheels turn
    (MIS4 supersnow, MIS01 paris_henchbigbox, RACE); the openheli / subcopter / jungleheli rotor
    meshes carry triangles longer than 5 units; each posed part has its own slot with vertices.
  - `driving_vehicle_parts_live_camera_facing_item`: the MIS01 helicopter's one 0xee item, posed
    through `CarRig::write_parts` on an unrotated car, turns +z from the camera through the item,
    level, for four camera positions, and stays at its origin.
  - Neither builds on the base commit (no `ai_rigs`, `HostilePool::rigs`, `write_parts`).
- nf-view `driving_*` / `drive_*` integration binaries: 20 binaries, 23 tests, all pass (the new
  one included). nf-view lib `driving` unit tests: 3 pass. nf-game `drive*` / `driving*`
  binaries: 39 binaries, 286 tests, all pass (no nf-game change).

Shots (local, not in the repo): before/after of the MIS13A jungleheli and the MIS3 openheli at
ticks 240 and 242 (`--drive-near-element`, `--cam`). Before: hub only, no main rotor. After: the
rotor disc draws and its blade pattern turns between the two ticks.

## Unproven or stand-in

- Whether the game runs the wheel rig for AI vehicles: `FUN_001d2cb0` runs only when the
  vehicle's +0x40 word is 1, and the writer of that word is not traced. The view runs it for
  every car.
- The AI steer input to the rig (the car's angle over its steer limit) and the hostiles' spring
  compression (rest length) are stand-ins.
- The rotor frame is the view's tick count from load (`DAT_00334000` is the sim tick counter;
  its start value on a driving load is not traced).
- MIS11's helimine elements did not spawn in the staged run, so their rotor was not checked.
- No 0xee item is drawn by any traffic or hostile model; the only one on the disc is on the MIS01
  helicopter, which the view does not use as a vehicle. It is checked through `write_parts`.
- The rotor disc's look (its texture and blending) was not compared with the game.
- The MIS4 supersnow's front ski struts draw well below its body in a close shot (wheels at the
  springs' rest length, the rig's `WHEEL_RADIUS + REST - off - REST` offset); whether the game
  draws them there was not compared.
