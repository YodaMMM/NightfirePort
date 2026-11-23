# s/drive-reflection-map: the car reflection map's view

Data and logic only (nf-formats). No draw change, no change to the shared random stream.

## What was traced (DRIVING.ELF, disassembled where the decompile was unclear)

- **Camera** [M `FUN_001c5eb0`, `FUN_001c1ec0`, `FUN_001c1b18`]:
  - The map is rendered once per frame from the player car's body position (`FUN_00201af0(0x339190)`,
    the first car of the sim's `+0xdcc` list), raised by `Refl HEIGHT`. Every car samples that one map.
  - forward = normalize(0.8 · main camera forward + (0, 1, 0)).
  - right = normalize((0, 0, −1) × forward), up = forward × right.
  - The view inverts the rows (right, up, −forward, eye), the same way the main camera does.
  - right × up = +forward, so the map is a mirror image.
- **Projection** [M `FUN_001c1560`, `FUN_00295910`]:
  - Symmetric perspective with full angle `Refl ANGLE` across and down, aspect 1, near 1, far 2500,
    set once.
  - The `.tun` value is in place for that call: `FUN_001426a8` reads each key at registration,
    inside `FUN_001c0930`, which runs first.
- **Scene**:
  - Track `Map`/`in` items [M `FUN_0021fa58`]. Kept when the top is more than 10 above the car, the
    item is within 50 + radius and within radius + 25 + 0.75·height, and `flags & 0x0d` is 0. The
    `+0x3c` bounds word is decoded: 10-bit radius and height, scale 0.25 or 16.
  - Then up to 4 vehicles, the ones with the smallest LOD measures [M `FUN_001c2720`, `FUN_001c2508`,
    `FUN_001c85f8`].
  - No sky dome. Fog is pushed out to 0..24560 for the draw.
- **`cref`** [M `FUN_001c1358`, `FUN_001a9600` + D]:
  - Key 4 is the modules' `sn` sheets, and each track names its own sheet.
  - paris, uw, snow1a and snow2a have a `cref` in their track sheet. The three jungle tracks fall
    back to `ext.ssh`.
- **Defaults and disc values**:
  - Constructor defaults: ANGLE 120, HEIGHT 0, `Sky ?` on, `Desat ?` off, blend 24, bright 223.
  - All 8 archives set bright 127 and Desat 0x80919191 [D].

## Code

`crates/nf-formats/src/rendertun.rs`:
- `CarRender::reflection_view(car_pos, main_forward) -> ReflectionView`, which holds eye, right, up,
  forward, `ReflectionProjection` (`clip`, `map_texel`), `ReflectionItemQuery::keeps`, the vehicle
  count, the fog and the steps.
- `item_bounds`.

Docs: `docs/formats/car-model.md` §10.6.3 / §10.6.4 / §10.7.

## Tests

- `cargo test -p nf-formats --lib rendertun`: 9 passed. That includes 4 new tests on the axes,
  projection, bounds decoding and item filter.
  - Mutations each turned a test red: swapped cross order, rise 5 instead of 10, height = radius,
    un-negated y, flag mask 0x0f.
- `--test carshade_disc`: 5 passed. The new test `every_tracks_reflection_map_inputs_decode` covers
  the globals, the `sn` sheet per track, the sheets that hold `cref`, the item counts, no zero
  radius, and the flag drops per track.
- `--test rendertun_disc`: 1 passed.

## For the view (not done here: crates/nf-view is out of scope)

- The port needs a 256 × 256 render target drawn with `reflection_view`, mirrored as described.
- The scene is only items that rise over the car, plus nearby vehicles, between the `cref` quads.

## Unproven

- The clear value.
- The GS viewport step: the texel formula is inferred.
- The fog words' meaning, and `DAT_00339090 + 0x2024`.
- Whether the player's own car is among the 4 vehicles, and what subclasses put in vtable `+0x24`.
- When `FUN_001c27a8` runs.
- The item LOD measure under the reflection camera.
- `DAT_003349e0 + 0x30`.
- `+0x10` on the 0x40-byte body table.
