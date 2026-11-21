# s/drive-hostile-runtime: the hostile runtime the driving view can own

No change to the shared random stream: every draw is the driving generator (`ai::DriveRng`),
passed in by the caller, and no existing function changed its draws. Nothing on foot is touched.

## What a player would notice

Nothing until the driving view calls it (crates/nf-view/src was out of scope). Once wired, the
driving missions get their enemy vehicles: spawned and removed at the game's 200 / 220 ranges and
slot budgets, built from their own car files, given their mission orders, chasing on the road or
straight at the player, firing guns and rockets that take health off the player's car, and
window and sunroof gunners firing on their own timing.

## What was added

- `nf-game` `driving/hostiles.rs`: `HostileWorld` (built from the track's `MapAi`, the mission's
  pvehicle `.atr` set and the world `.atr`), `step(net, ground, surface, player, rng, ray)`
  returning the database actions, `damage::Hit` calls against the player, traffic park requests
  and shot / launch / impact / rider-shot / blast events; `draw_list()` (model, matrix, wheels,
  rotor), `projectiles`, `activate` / `deactivate` / `command` / `damage` for the script and the
  player's weapons; `VehicleAttrs` (the `.atr` keys the game tests) and `models_needed(map,
  set)`.
- `nf-formats` `driveai.rs`: AICo +0x93 / +0x94 (AI +0x130 aim skill, +0x131 damage modifier)
  [M 0x145ff8].
- docs/systems/driving-ai.md §13: the step order, what was read for this branch (0x145ff8 tail,
  0x1476b8, 0x14b510, 0x12e468, 0x182978), every stand-in, the disc results.

## How the view uses it

Per mission: `models_needed` to preload models; `HostileWorld::new`. Per tick: `step`, then
`damage::inflict` for each hit on the player's `CarDamage`, park the traffic in
`park_traffic`, draw `draw_list()` and `projectiles`. Script effects `AiActivate` /
`AiCommand` map to `activate` / `command`. Today the view should pass `FlatWorld` as `ground`
and the track's `CollisionGround` as `surface` (see below).

## Tests

- `nf-game --test driving_hostiles_runtime` (new, 12): pass, NF_ISO set. 11 unit tests (the
  `.atr` keys, the model list, the channel → weapon map, spawn inside 200 / despawn beyond 220,
  the slot cap and priority, HELD / activate / deactivate, the spawn command driving a pursuer
  and a kind-4 car standing, guns and rockets landing kind-2 damage calls of the table's damage,
  no weapon without the `.atr` declaration, the sunroof rider's cadence and the seat stop, a
  helicopter in the heli pool gunning on approach with the exact draw count, the draw list).
  The new module did not exist before, so every test fails to build on master.
- The disc test runs MIS01 and MIS4 for 3600 ticks with a scripted player on the roads: every
  spawn within 200 (or persistent), every despawn beyond 220 / lost slot / held, the slot caps
  held every tick. MIS01: 5 spawns, cars within 1.5 of their road distance, 113 rider shots, no
  vehicle weapons on its cars. MIS4: 8 spawns, 4 despawns, worst stray 15.9, 494 shells, 240
  damage calls.
- `driving_ai_hostiles` (20), `driving_ai_patterns` (21), `driving_ai_heli` (16), `nf-formats`
  `driveai` unit tests (4): pass. `nf-view` builds (cargo check) against the change.

## Unproven / not done

- On the land physics (`CollisionGround`) a MIS01 pursuer hits a kerb about 4 s in, rolls over
  and falls through the ground mesh; a MIS4 car strays 140 from the road. The runtime's disc
  test therefore asserts on the rail stand-in and only reports the land run. This is a
  `Car::step` land-model issue under AI inputs and needs its own look.
- Not traced: the shell's flight (a shot hits when exact or on line), the explosion's radius
  damage (a projectile reaching the player books its full damage), the rider's round (rider
  shots carry no damage, so MIS01 lands no damage calls), the launcher busy test, the helicopter
  flight model, the EAGL path playback (pattern 5 cars and path helicopters stand still), the
  road under the target, the element targets of AICo +0x50, the nearest-road grid query.
- The disc run releases each element when the player first comes within 200 of it; the mission
  triggers that really release them were not run. With that, neither mission reached its
  vehicle slot budget; the cap and priority paths are covered by the unit tests.
- No rockets were launched on the disc runs: the MIS4 commands reached carry no rocket byte.
