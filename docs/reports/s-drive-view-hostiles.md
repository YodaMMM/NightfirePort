# s/drive-view-hostiles: the driving view runs the mission's enemy vehicles

No change to the on-foot code or the on-foot random stream. The hostiles draw from their own
driving generator instance (`ai::DriveRng::game_reset()`, `FUN_001feda0`'s reset): the view's
traffic cars each keep a private copy, so there was no shared instance to join.

## What a player would notice

The driving missions now have their enemy vehicles. On MIS4 the two snowmobiles the script
releases near the start chase the car, fire their guns and take health off it (about 810 of
10000 over 40 s with a straight-line throttle), then drop away once the car is past their range.
On MIS01 the four pursuers that start unheld (two henchman cars, the bomb van, the paradis car)
are on the road from the start; two of them drive. Each draws with its own car model. Gun shots
show as a short glowing tracer, rockets and missiles as a glow. The player's rockets now hit and
damage the hostiles too.

## What was added

- `crates/nf-view/src/driving.rs`:
  - `DriveHostiles` on `DrivingState::hostiles`: the `HostileWorld`, its generator, a track
    ground for heights when the view runs no land pass, one `HostilePool` of palette slots per
    hostile model, the player car's `CarDamage` (category 0), the pending script AI events,
    the last `StepOut`, tracers, counters.
  - Load: `MapAi::from_crp`, each AIEl vehicle's own pvehicle `.atr` (`VehicleAttrs`), the
    world `.atr` (`WorldAttr`), difficulty 1 (as the view's HUD), the health factor read from
    MSet f32 +0x2c4 + 4·difficulty (`FUN_002084a8` per hostile.rs), speed factor 1.0. Every
    model `models_needed` names is loaded through `load_car_into_scene` (the traffic path), one
    slot per element that names it, capped by the vehicle or helicopter slot budget and 8.
  - Step (after the mission script each tick): the script's `EActivateAIElement`,
    `EDeactivateAIElement` and `EAICommand` (references resolved through their `sr` path,
    `{AIEl}` / 0x100 and `{AICo}` / 0xa0) go to `activate` / `deactivate` / `command`; then
    `HostileWorld::step` with the player's pose, `FlatWorld` as the drive ground and the
    track's `CollisionGround` as the height surface (as the runtime report says), the sight ray
    on the collision grid. Each `HostileHit` goes through `damage::inflict` into the HUD's
    `PlayerHealth`.
  - Draw: an element keeps a slot of its model while it lives; `apply_to_palette` writes its
    matrix (rows from `draw_list`, the model placed by the player car's rest-contact rule) and
    folds every free slot away. `particle_groups` adds the hostile projectiles and gun tracers
    as additive glows. `hostile_draws()` lists each drawn element with its slot.
  - The player's projectiles also target the live hostiles and book hits through
    `HostileWorld::damage`.
- `crates/nf-view/src/lib.rs` (headless driving `--shot` only): `--drive-throttle T` holds the
  throttle for the run; the hostiles drawn are printed.
- `crates/nf-view/tests/driving_hostiles_view_live.rs` (new).
- `crates/nf-view/tests/driving_colour_scale_live.rs`: scoped to the Vanquish's own slots. It
  took every textured car draw in the scene; the hostile models (base-pass colours of 255 on
  their up faces, the same as the traffic car model already showed) outnumbered the Vanquish.

## Tests

- `driving_hostiles_view_live` (NF_ISO): MIS01 and MIS4 for 3600 ticks each with a scripted
  throttle (0.7 for 450 ticks, 0 for 150, repeated). Every tick: each drawn element's slot
  belongs to its own model's pool, carries that model's vertices, and its palette matrix sits
  at the element's position; a free slot is zero; an element that stops drawing is no longer
  live. End: MIS01 4 elements drew, 2 moved; MIS4 the script activated 2 elements, both drew
  as supersnow, moved, landed hits (health 10000 to 9190) and were removed. It does not build
  on the base commit (no `hostiles` field).
- nf-view `driving_*` / `drive_*` integration binaries: 17 / 17 pass. nf-view lib `driving`
  unit tests: 3 pass.
- nf-game `drive*` / `driving*` binaries: 35 binaries, 260 tests, all pass.

## Unproven or stand-in

- The box point of a hit on the player (`FUN_002d3950`) is the hit point clamped to the car
  box; the zone box is the car's half extents; `mission_350` is taken as clear; the push a
  kind-2 hit returns is not applied to the player car.
- Where a hostile model sits on its body uses the player car's rule (lowest vertex at the rest
  contact) [S]; helicopters draw at their body position with no rotor spin (the parts are baked
  at rest, as for traffic).
- Riders and henchman elements are not drawn (the driving view has no character draw); rider
  shots carry no damage (as the runtime).
- `EAICommand` on an element that is not live is dropped (`HostileWorld::command` acts on live
  elements; whether the game keeps it for the spawn through AIEl +0xf8 was not traced).
  `EAIElementFireOff/On`, accuracy, kill, wake range and the trigger-car form of
  `EActivateAIElement` (element 0) are not routed.
- The traffic park requests (`park_traffic`) are not applied to the view's traffic cars.
- The script's `Facts` do not carry the hostiles (element health / position rules and the
  trigger walk's AI cars), so rules that read them do not fire yet.
- The base-pass colours of the hostile and traffic models read 255 on their up faces
  (MIS01 henchman car, bomb van, road block truck); whether that is the game's light or a
  CarRender class lookup issue is not checked.
- MIS01 reads Failed at tick 424 with or without the hostiles (before any input); not looked at.
- The snow2a_mis4 `paris_hench` to `junglesnow` model swap is not applied.
