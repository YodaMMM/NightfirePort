# mp-pickups-visible — MP arena weapon pickups: the match's weapon set and the weapon's own model

*The user (2026-10-19, MP from the menus): "You can't see weapon drops on the ground." Everything game-side here
cites the decompile [M] or the arena archives on disc [D]; port choices are [S].*

## 1. What the game does

**The placement.** A level's type-0xF0 dynamic-object placements become pickups through
`parsemap_create_dynamic_objects` 0x1d04f0 case 0xF0 → `Pickup_Create` 0x18dda0 (the placement's params give
kind/id/rounds/switch/sound/respawn; the placement's resolved celglist rides as `Pickup_Create`'s model argument).

**The weapon-set intercept [M].** `Pickup_Create` 0x18dda0 (after the glist ≠ 0 gate 0x18de04 and the id = −1
refusal 0x18de0c) intercepts kinds **7..11** (`kind-7 < 5` unsigned, 0x18de4c-0x18de50) and calls
`Pickup_CreateFromSet(pos, rot, kind)` 0x18f1b0, which:
- reads the match's weapon set `MPSettings+0x1B4` (0x18f1b8; the same word as the port's `MpSettings::weapon_set`);
- takes `w = PickupMatrix[set][kind-7]` — `PickupMatrix` @0x2b9100, 11 rows × 5 s16 (the row = weapon set, the
  column = the placement's kind minus 7); row 10 is runtime-filled by `Pickup_MakeRandomWeaponSet` 0x18f098;
- maps `w` to its base `weapon_data[w]+0x02` and creates a **kind-0 weapon pickup** with
  `id = weapon_data[w]+0x00`, `rounds = *(s16*)(weapon_data+base·0x10C+0x92) << 1`, sound 0xF5, rest 1,
  **respawn 3** (·10 s hidden), lifetime 0 — and **the model is the weapon's own third-person world model**,
  `hashtable_hashcode_to_celglist(*(u32*)(weapon_data+base·0x10C+0x80))` (0x18f1b0:56-61; refused when it does not
  resolve, the same null-glist gate 0x18dda0:88). **The placement's object is never drawn for kinds 7..11** — it is
  only required to resolve for the gate.
- Special case (0x18f280-0x18f300): row entry 0x1A (AIMS-20) additionally spawns a kind-1 ammo pile of weapon 0x1B
  (model 0x02007c4, +0.1/+0.1) — the port documents this as an unmodelled remainder (pickup.rs `create_from_set`);
  sets 0 and 1 contain no 0x1A, and this task leaves it as documented.

**MP-only extras [M 0x18dda0:138-144, 197-199].** In a match (`MPSettings+0x180 ≠ 0`) every created pickup spins
(`+0x38 = 1`, ~1 rad/s about +Y, applied per tick by `Pickup_Update` 0x18e148:193-201) and registers in the
`MPpickups` table (the port's `Drones::pickups`).

**Spin scope:** the per-tick spin rides `obj+0x90` in the game; the port's `Pickup.spin` already drives the same
rotation (`Pickup_Update` port path), and the draw follows the sim's matrix (`Scene::n`, scene.rs:4868).

## 2. What the arenas actually place [D]

Parsed from the ISO (`07000023.bin`/`07000024.bin`, main map, placements with `flags & 0xff == 0xf0`, params read
by index like `parsemap` does):

- **07000023 Atlantis**: 20 pickups — 15 of kinds 7..11 (kind 7 ×1, kind 8 ×5, kind 9 ×4, kind 10 ×5, kind 11 ×1)
  and 5 kind-3 armour. Every kind 7..11 placement's object is named `PICKUP` with **hashcode 0xFFFFFFFF** — a
  generic marker with **no by-hash entry** (hashcode 0xFFFFFFFF cannot be looked up).
- **07000024 Skyrail**: 28 pickups — 23 of kinds 7..11 (kind 7 ×5, kind 8 ×9, kind 9 ×5, kind 10 ×4, kind 11 ×2)
  and 5 kind-3 armour. Same `PICKUP` / 0xFFFFFFFF placement objects.
- The placement's own `id` param (0x12, 0x2c, 0xa, 0x19…) is **ignored** for kinds 7..11 (`Pickup_CreateFromSet`
  takes only the kind) — it is the SP-style default, not the MP weapon.
- **Model availability [D]**: for both arenas and both weapon sets 0 and 1, every mapped weapon's `+0x80` world
  model hash resolves **in the arena's own archive** (`level=true` for all ten rows' weapons on both; the one
  story-order miss, 0x42's 0x020008ae, is local to the arenas). Every `weapon_data` `+0x80` hash the disc defines
  resolves after the existing session merge (0 missing from arena+story on both arenas) — dropped guns (the
  `Pickup_CreateSimple` path) resolve their models too.

## 3. What the port did instead (the bug)

`Scene::build` builds placed pickups twice — the draw side (scene.rs ~5936) and the sim side (~7468):

- **Sim side** (mp-lobby-settings, correct row): kinds 7..11 → `Pickup::create_from_set_row` with the match's row
  (`weapon_row(mp)`) — the right weapon — **but then overrode `obj` with the placement's object**
  (`p2.obj = Some(obj)`), so the renderer drew the `PICKUP`/0xFFFFFFFF marker instead of the weapon's model.
- **Draw side**: kinds 7..11 went through `Pickup::create_placed` → `create_from_set`, which hardcodes
  **`PICKUP_MATRIX_ROW0`** — the wrong row whenever the lobby's weapon set ≠ 0 (the weapons the sim hands out do
  not match the models the draw side builds) — and also overrode `obj` with the placement object.
- Net effect: every set pickup on 07000023/07000024 drew the arena's generic `PICKUP` marker (hashcode 0xFFFFFFFF,
  not a weapon), instead of the weapon's own world model [M §1] — the user sees no weapon pickups on the ground.
  (Dropped guns ride `weapon_data[id]+0x80` and resolve [D §2]; the placed set pickups were the invisible ones.)

## 4. The fix (port)

One shared builder for both sides (`scene.rs::placed_pickup_from_placement`, [S] structure — the game creates once):
- kinds 7..11 with a match row → `Pickup::create_from_set_row` (the match's row, the weapon's own `+0x80` model),
  `obj` stays `None` so the renderer falls back to the model hashcode (which resolves [D]) — the game's own
  behaviour (the placement object is never drawn);
- all other kinds → `create_placed` with the placement's resolved object (unchanged, [M 0x1d04f0]);
- in a match every placed pickup spins (`+0x38 = 1`, [M 0x18dda0:138-144]);
- both builds call the same builder with the same `mp_row`, so the draw slot k and the sim's `pickups[k]` cannot
  diverge again (the per-index draw contract, scene.rs:4868).

No drawing code changes beyond the model each instance points at.

## 5. Tests (`crates/nf-view/tests/mp_pickups_visible_live.rs`, ISO-gated, numbers/ids only)

For each arena (07000023, 07000024) and weapon set 0 and 1, through the real load path (`nf_view::load` with the
match's `MpSettings`):
1. every kind 7..11 placement becomes a kind-0 pickup whose weapon id is `PICKUP_MATRIX[set][kind-7]` [M 0x2b9100
   via mp/equip.rs] and whose model hash is `weapon_data[base]+0x80` [M 0x18f1b0];
2. that model hash resolves to an object with **triangles** (`MapObject.gfx.leaves` → `Leaf::triangles()`,
   ps2gfx.rs:73) — drawable;
3. sister: an SP level (07000001) loads with no match and its pickups are unchanged — kinds 1/3/6 only, each still
   drawn from its placement's own object, none of kind 0 from a set slot.
