# s/drive-projectile-gfx: what driving projectiles, explosions and bullet streaks draw (data + logic)

No change to the shared random stream (the ribbon jitter takes a `DriveRng` the caller passes; no
existing caller changed). No draw or view changes (`crates/nf-view/src` untouched): the view still
draws its stand-in glow and none of this is wired in yet.

## What was found [M DRIVING.ELF, D disc]
- **Model**: a projectile is a copy of one item of the shooter's own car model (`FUN_001c7b60`
  from the fire event `FUN_0014ca60`), drawn with an identity local matrix. The weapon is the one
  whose barrel list holds that item's action byte (`as + 0x1a`, `+0x1b` mirrored; `FUN_001c9328`,
  `FUN_0020e678`). On the disc the barrel items carry an `ef` entry whose `+0x15` names the item to
  copy, e.g. vanquish rockets 39/40 → item 39, missiles 47/48 → 47 and 30/31 → 30, Q-Pulse item 14,
  vanquishsub torpedoes 22, remote torpedoes 12 (14 cars carry barrel models).
- **Body matrix**: rows up × f, f × x, f (`FUN_00210540`).
- **Smoke ribbon**: per-kind type (Snowmobile Rocket 1, torpedoes 2, else 0), half-width 0.3575 /
  0.1 / 0.1, jitter 0.2 / 0 / 0, 0x60 vertices, a 40-vertex window with an alpha ramp, texture
  `trai`; not drawn for Missile and FOD Rocket; the Q-Pulse has none.
- **Lights**: per-kind glow (Lighting tun `Colour/Size{Missile,Rocket,SnowmobileRocket}`, 2 slots,
  a size that flashes at launch then grows after one second), the Q-Pulse's own blue light, the
  explosion light by damage class with `Decay`, at most 4 lights a frame.
- **Explosion**: the Giotto effect choice (FOD, snow, water, the `Un_Expl*` sizes, `Ex_SmallD*`,
  `Ex_SmB0*` on the player, `Ex_Small*`), the scorch decal record 5, the camera shake range.
  `FODHitEffect`, `rocketsplash` and `Dummy` are in no effects gallery, so they never show.
- **Bullet streaks**: list, 60-tick purge, head = start + dir·(t·Speed + Length), the camera-facing
  quad (Width, Length, halved colour, `wtrc/ytrc/otrc/btrc`, additive).

## Code
- `crates/nf-formats/src/driveproj.rs`: `ProjectileLighting` (tun + ELF defaults), ribbon/streak
  constants and textures, `action_items` (car items with action byte, LODs, launch effects).
- `crates/nf-game/src/driving/projectile_gfx.rs`: `body_matrix`, `barrel_models`, `Trail`,
  `GlowSlots`, `emp_light`, `BlastLights`, `frame_lights`, `explosion_effect`, `blast_shake`,
  `ProjectileFx::tick` (the per-tick draw description), `Streaks`.
- Doc: docs/systems/driving-projectile-gfx.md (driving-weapons.md points to it).

## Tests (NF_SILENT=1, NF_ISO set)
- `cargo test -p nf-formats --lib driveproj`: 3 passed.
- `cargo test -p nf-game --lib projectile_gfx`: 12 passed.
- `cargo test -p nf-game --test drive_projectile_gfx_disc`: 3 passed (all 8 tracks' lighting
  values, common.ssh textures, gallery gaps, every car file in the 8 track archives and MISC).
- Mutation check: reading the action byte without the mirrored offset fails the car test.

## Unproven
- Which `ef` entry the fire event carries comes from the event dispatcher (`DAT_0032e710..`),
  which was not traced; the rule above is from the data plus `FUN_0014ca60`'s `+0x15` override.
- Whether the copied item's launch effect plays during flight; the car guns' muzzle flash
  (`GIOT::Dummy` is in no gallery); which view-matrix row the ribbon's side vector is; the meaning
  of `DAT_00339f44 == 3`, light mode 1, and the player object's `+0xbc` / `+0x50`; the streak
  bucket reset and VU quad expansion. Only a capture from the real game can settle the look.
