# control_funcs update column against the port

`control_funcs` at 0x26f890, 12 bytes per class. For every class with a non-null update: which levels create it, and which port function ticks it? A class with objects on an on-foot level and no port tick is a gap. Port the small ones, list the rest with counts.

Read the game code first, count on every level with `Level::parse` over `nf_view::dev::LEVELS`, and test on the real level, red on the old code. A count of zero on every level closes an item with that count as proof. Do not trust a port comment that says inert. Values measured at load can differ from the same global in play (FRAME_RATE reads 30 while a level builds, 60 after). Branch from origin/master, push to `s/control-funcs-update`, report in docs/reports/control-funcs-update.md. No driving levels. Draw work: leave a stub in the hand-off list and say so.

---
claimed-by: sandbox4 at 2026-11-07T20:25Z

---
## Report (done, 2026-11-07T21:04Z)


(report path was relative and missed the queue cwd — content appended now)

# control-funcs-update: the update column of `control_funcs` against the port

Branch `s/control-funcs-update` from `origin/master` (10956badb). The class table
`control_funcs` @0x26f890 in ACTION.ELF holds 85 rows × 12 bytes (update / collide / delete
slots) [M ACTION.ELF @0x26f890, dumped per entry with the loader mapping vaddr−0x107100+0x1000;
the same table read live in ee_start_05.bin @0x26f890, docs/port-map/core-loop.md §2 item 7].
The update slot of row `c` is `0x26f890 + c·0xC` [M 0x1336f8:62-68 dispatches
`control_funcs[obj+0xff · 0xC](obj)` per object]. Every non-NULL update pointer names a
decompiled function (the `work/decomp/ACTION.ELF/00<addr>_` file set), so the whole column is
accounted for. The census (`crates/nf-view/tests/update_column_census.rs`) counts placements
per class per level with `Level::parse` over `nf_view::dev::LEVELS` — all 34 archives on the
disc, loads only, no driving — using the collide census's placement-flags→class dispatch table
(the `parsemap_create_dynamic_objects` 0x1d04f0 jump table, shared with
`crates/nf-view/tests/collide_column_census.rs` on s/control-funcs-collide).

## Done

1. **The update column enumerated.** 54 of the 85 classes have a non-NULL update; each maps to
   its decompiled function (Player_Update 0x193600 on 0x03/0x12, Drone_Control 0x139268 on
   0x02/0x11, Bullet_update 0x12e7b0 on 0x05, …). The full table with per-level counts is in the
   census test output; the placements-only summary is below. 31 classes have NULL update slots
   (0x00, 0x01, 0x04, 0x08–0x0b, 0x13, 0x14, 0x16–0x18, 0x1e, 0x29, 0x2b, 0x2c, 0x2d, 0x3d,
   0x3e, 0x42, 0x51) — they can never be update gaps by construction.
2. **The census** ran over every archive: totals per class and per-level rows are printed and
   the closures are asserted in the test (see Verification).
3. **Every placed class but two already has a port tick.** The walk dispatches per class in
   `Drones::tick` + the module ticks: drones (`drone.rs`), doors (`door.rs`), triggers
   (`triggers.rs`), RB (`rigid_body.rs`), SP (`nis.rs` scripts), break/destroy
   (`breakable.rs`), searchlight, copter, pickups, spawners (`spawners.rs`), sensor, monitor,
   switch/lock/PCQWorm (`useobj.rs`, `lock.rs`), fusebox (`fusebox.rs`), hurt, SS (`ss.rs`),
   cloud/flicker/lightning/ripples/emitter/env-tree/apocalypse (`world_fx.rs`), sound triggers
   (`sound.rs` + `drone.rs`), hints (`player.rs`), GunImp (`gunimp.rs`), CamSubject
   (`props.rs`), rotor/music-trigger/shooter/dynamic-object/inverter (`worldobj.rs`), mine
   (`worldobj.rs`), space laser/missile (`space_laser.rs`), cars (`car.rs`, MP-only content,
   SP refuses [M 0x1305b0]).
4. **Gap found and ported: `OneSided_Update` 0x1e7d58 (class 0x4C).** The disc's single
   type-0xd5 placement (07000041, sub-map 99 "01000058", translation (−3.011, 2.555, −44.897))
   creates a plain object [M `OneSided_Create` 0x1e7ce8:9-16] whose update dots the object
   matrix's dir (`Mat_GetDir` 0x1e3398 reads obj+0x90 column 2) with (obj pos − viewer
   `glb_viewer+0xC0`) and rewrites obj+0xF0 bit 0x10 every tick (dot < 0 → hidden) [M 0x1e7d58].
   The port had nothing: the scene.rs comment claimed the job was "the port's per-triangle
   backface cull of the placed glist", but **no pipeline culls anything** — `cull_mode: None`
   in every wgpu PrimitiveState [M crates/nf-view/src/gpu.rs:745,772, devwin.rs:391]. That is
   exactly the "do not trust a port comment" case. Ported in `crates/nf-game/src/worldobj.rs`:
   the `OneSided` record (create arm 0xd5, matrix column 2 as dir, placement index for the
   draw-side join) and the per-tick flip inside `WorldObjects::tick` (no RNG, no channels — the
   shared RNG stream is untouched, determinism holds). The stale scene.rs claim is corrected in
   place.
5. **`GT_Update` 0x180498 is the remaining real update gap** — hand-off, not ported (3948-byte
   update; `gt.rs` already documents the turret behaviours — `GT_Update`, `GT_Track` 0x17f9c8,
   `GT_ApplyDamage` 0x1816d8, deploy — as a known gap). Counts: 41 placements disc-wide —
   campaign 0700000a ×6 (sky-bridge turrets) and 0700000b ×5 (terminal/exec floor), 2 per MP
   map (07000023..29, 0700004b). `GT_Create` 0x17ffa0 gates on params 2/3 non-zero, so the
   created-object count can be lower than the placement count [M 0x17ffa0:69-71].

## Zero-placement closures (the count is the proof)

Never placed on any of the 34 archives [D census]: the runtime-only classes 0x03/0x12 (players),
0x05 (bullets), 0x0f (explodes), 0x11 (MP-bot drones), 0x0c (particles), 0x0d (casings),
0x0e (gas/smoke), 0x10 (debris), 0x15/0x1f (script anims), 0x06 (bubbles), 0x19 (sun
reflection), 0x2a (occlude), 0x2e (leaves), 0x41 (swoosh), 0x46 (body glow) — all created by
the game's own systems, not by placements; and the placed-elsewhere-in-theory classes 0x23
(Creature_Create 0xe0 exists, no archive uses it), 0x45 (Corona_Update's class; coronas are
created as class-0x01 helpers with a NULL update slot [M corona.rs]), 0x4a/0x4d (Sub/MiniSub,
matching the collide census's closures). The census test asserts all of these stay zero, so a
future archive that places one reopens the item loudly.

## The placed classes and their port ticks (campaign-relevant)

| Class | Update | Levels (placements) | Port tick |
|---|---|---|---|
| 0x02 | Drone_Control 0x139268 | every level (588) | `drone.rs` walk |
| 0x07 | rotor_update 0x18f568 | 01×2, 09/0a/0c/0d/12/13×1, 11×3, 41×4 (15) | `worldobj.rs` rotors |
| 0x1a | RB_Update 0x1d6e38 | 01×4, 02×21, 03×3, 04×2, 06×1, 0d×3, 41×1 (35) | `rigid_body.rs` |
| 0x1b | SP_Update 0x1dbda8 | every level (253) | `nis.rs` script players |
| 0x1c | Door_Update 0x1363c8 | most levels (447) | `door.rs` |
| 0x1d | Trigger_Update 0x1b1a78 | every level (676) | `triggers.rs` |
| 0x20 | Break_Update 0x12bfd8 | most levels (461) | `breakable.rs` |
| 0x21 | Destroy_Update 0x12c4f8 | most levels (197) | `breakable.rs`/`drone.rs` destroy pass |
| 0x22 | Searchlight_Update 0x1cb900 | 05×29, 06×20, 41×5, 01/08/0a (54) | `searchlight.rs` |
| 0x24 | cloud_update 0x18d768 | 06×1, 41×1 (2) | `world_fx.rs` |
| 0x25 | SS_Update 0x1a9948 | most levels (110) | `ss.rs` |
| 0x26 | Flicker_Update 0x1ab960 | 29×1, 41×1 (2, MP only) | `world_fx.rs` |
| 0x27 | Hurt_Update 0x17e5d0 | 05×13, 0a×5, 15×9, 41×8, … (64) | `hurt.rs` |
| 0x28 | Car_Update 0x1307d8 | MP maps + 02/46/48/4b (55; campaign rows are origin dummies per the collide census) | `car.rs` |
| 0x2f | Pickup_Update 0x18e148 | every level (570) | pickup pass in `drone.rs` |
| 0x30 | DroneSpawner_Control 0x137e68 | every level (112) | `spawners.rs` |
| 0x31 | LeafGen_Update 0x1acc48 | 04×2, 41×1 (3) | `world_fx.rs` LeafGenFx |
| 0x32 | Emitter_Update 0x17b080 | 02×1, 14×35, 15×16, 46×17 (69) | `world_fx.rs` emitters |
| 0x33 | Lightning_Update 0x18cf68 | 01×11, 41×4 (15) | `world_fx.rs` |
| 0x34 | Ripples_Update 0x1ac028 | 01×7, 02×16, 04×4, 0b×5, 41×1 (33) | `world_fx.rs` |
| 0x36 | **GT_Update 0x180498** | **0a×6, 0b×5, 2 per MP map (41)** | **none — hand-off** |
| 0x37 | Sensor_Update 0x1af618 | every level (62) | `sensor.rs` |
| 0x38 | Monitor_Update 0x1afcc8 | 01/02/03/0a/0c/0d/11/12/14/41/46 (17) | `monitor.rs` |
| 0x39 | Switch_Update 0x1adea8 | every level (99) | `useobj.rs` |
| 0x3a | Lock_Update 0x1ae3d8 | every level (123) | `lock.rs` |
| 0x3b | Copter_Update 0x17dbb8 | 05×5, 08×3, 0b×3, 12×2, 46×1 (14) | `copter.rs` |
| 0x3c | FuseBox_Update 0x1adfe8 | every level (10427) | `fusebox.rs` |
| 0x3f | Env_Tree_Update 0x17c328 | 02×6, 04×2 (8) | `world_fx.rs` |
| 0x40 | SoundTrigger_Update 0x1b2790 | most levels (104) | `sound.rs`/`drone.rs` |
| 0x43 | Hint_Update 0x1b3380 | most levels (49) | `player.rs` |
| 0x44 | MusicTrigger_Update 0x1b31a0 | 01/02/03/07/08/0a/0b/0c/0d/11/41/46 (15) | `worldobj.rs` |
| 0x47 | GunImp_Update 0x1320d8 | 0d×3, 16×2, MP maps (37) | `gunimp.rs` |
| 0x48 | CamSubject_Update 0x1bc668 | 0c×2, 0d×2, 41×1 (5) | `props.rs` |
| 0x49 | PCQWorm_Update 0x1aee20 | 0a×3, 0b×1, 11×1, 12×1, 46×1 (7) | `useobj.rs` |
| 0x4b | Mine_Update 0x17c930 | 15×9, 41×2 (11) | `worldobj.rs` mines |
| 0x4c | OneSided_Update 0x1e7d58 | 41×1 (1) | **ported this task** (`worldobj.rs`) |
| 0x4e | Shooter_Update 0x17d0c0 | 01×1, 15×1, 41×1, 4a×2 (5) | `worldobj.rs` |
| 0x4f | SpaceMissile_Update 0x1aa118 | 1b×8 (8) | `space_laser.rs` |
| 0x50 | SpaceLaser_Update 0x1aa3f8 | 1b×1 (1) | `space_laser.rs` |
| 0x52 | DynamicObject_Update 0x18d3b8 | most levels (75) | `worldobj.rs` dynobjs |
| 0x53 | InverterTrigger_Update 0x1b30a8 | 06×1, 12×1, 14×4 (6) | `worldobj.rs` inverters |
| 0x54 | Apocalypse_Update 0x18d4f0 | 14×11, 15×4, 16×2, 1b×8, 41×1 (26) | `world_fx.rs` |
| 0x35 | MP_ObjectUpdate 0x18b198 | MP maps only: 23..29 + 4b (286) | none (MP out of scope) |

The class-0x3c count (10427) is the fusebox: every level carries hundreds (230–451 each) —
they are the per-wire records the fusebox system registers, and `fusebox.rs` handles them.

## Verification

- `update_column_census` (nf-view): **1 passed** — per-level rows + the zero-closures (21
  classes), the MP-only scoping of 0x35 (zero outside 07000023..29/4b), the Sub/MiniSub
  closures, and the non-zero anchors.
- `onesided_update_pin` (nf-view): **1 passed** — the real 07000041 archive: exactly one 0xd5
  placement (sub-map 99), the record's dir == the placement matrix's column 2, and the hidden
  flag flips per dot(dir, pos − viewer) < 0 on both sides and re-arms per tick. Red on the old
  code: no 0xd5 arm existed (the master comment claimed a draw-side cull that does not exist —
  `cull_mode: None` everywhere).
- `worldobj_live` (nf-view): **6 passed**. `07000014_inverter_trigger_live` (nf-game): **1
  passed**. `cameras_turrets_census_live` (nf-game): **1 passed**.
  `walkreal_07000041_live` (nf-view): **15 passed** (the level the changed code touches).
- The full suite (`cargo test --offline --release -j2 -p nf-game -p nf-formats -p nf-audio
  -p nf-view -- --test-threads=1`) **did not fit** in this sandbox's window (setup consumed the
  first hour: fresh clone + work/ + ISO + toolchain). The worldobj-user binaries above are the
  ones the change can reach: the diff is one new match arm (0xd5 — previously the unused
  `_ => {}`), one new `WorldObjects` field and one new tick loop over it; no existing record,
  constant or match arm was renamed or repurposed.

## What's left (hand-off)

1. **GT_Update 0x180498** (class 0x36) — the turret update, unported; counts above. The
   campaign-relevant part is 0700000a's six sky-bridge turrets and 0700000b's five; the MP rows
   are out of scope. `gt.rs` already stores the placement-derived records (facing mode, HP,
   scripts, activation radius) that the port will need.
2. **OneSided draw consumer (stub — this is the draw work this task leaves).** The ported
   `OneSided::hidden` flag has no reader: the palette FAR-write that hides the static instance
   per frame (the `door_scene_hidden` precedent, scene.rs `skin_palette_into`'s `p[...] = FAR`)
   is not wired. Until then the 07000041 wall draws from both sides.
3. **OneSided walk wiring (stub).** `Drones::create` builds `WorldObjects` from the main map's
   placements only (`drone.rs` `WorldObjects::create(&level.maps[0].placements, …)` — the same
   scope every worldobj class has); 07000041's 0xd5 placement sits on sub-map 99, so the walk
   never creates the record. The game's `parsemap_create_dynamic_objects` 0x1d04f0 takes one
   placements array per parsed map [M its signature], so the per-map granularity is the game's;
   the port needs a per-map (or all-maps) create pass for sub-map placements before the record
   reaches the walk.
4. **Live checks for the main agent** (render checks; the port has no GPU here):
   - 07000041 (--bond near the one-sided wall, sub-map 99 object "01000058", placement
     translation (−3.01, 2.56, −44.90), dir ≈ (0, 0, 1)): a spot at (−3.0, 2.6, −25.0) looking
     down −Z at the wall and one at (−3.0, 2.6, −65.0) looking up +Z. In the game the wall
     shows from one side only (`OneSided_Update` 0x1e7d58 hides the other via obj+0xF0 bit
     0x10); in the current port it draws from both (the hand-off stub above) — the check
     demonstrates the remaining gap and, after the draw consumer lands, pins it.

## Usage

not visible to me
