# `DroneWeap_GetWeaponInfo` 0x17a5a0 — the held weapon's datum, sound and muzzle transform

Evidence tags: [M] = decompile (address = filename stem,
`:NN` = line in the .c), [S] = port choice, [D] = disc.

## What the game does

`DroneWeap_GetWeaponInfo(Drone*, int* datum_out, uint* mesh_out, ushort* sound_out, VECTOR* muzzle_out,
VECTOR* dir_out)` 0x17a5a0 (316 bytes) resolves the drone's held weapon into (a) the weapon's model
datum id, (b) its shot sound id, (c) the weapon bone's world position and (d) the weapon bone's world
−X axis:

1. `AnimDatumGetWeaponInfo(obj, 0, mesh_out, sound_out)` 0x1c7900: reads the weapon byte at the anim
   datum `+0x62` (the port's `Drone.weapon`); `≥ 0x73` fails [M 0x1c7900:11]; writes
   `weapon_data[id]+0x4C` (the muzzle-flash mesh) to `mesh_out`, `weapon_data[id]+0x60` (u16, the
   shot sound) to `sound_out`, and returns `weapon_data[id]+0x50` (the model datum byte, `0xFF` →
   `0xFFFFFFFF`) [M 0x1c7900:14-26].
2. GetWeaponInfo stores the return into `datum_out`; negative → return 0 [M 0x17a5a0
   0x17a5e8-0x17a5f0]. It re-checks the anim datum is alive and its `+0xC8` bit 0 is set (the
   posed/animation system is live; `AnimGetBoneWorldTrans` re-checks the same gate [M 0x1c3120:71])
   [M 0x17a5a0 0x17a5f8-0x17a614].
3. `sound_out == 0` → `0xFFFF` [M 0x17a5a0 0x17a618-0x17a630].
4. `AnimGetBoneWorldTrans(obj, datum, 1, muzzle_out, &M)` 0x1c3120: builds the blended matrix palette
   (`psiBuildMatrixPalette` [M 0x1c3120:72]) and looks up the bone whose id equals
   `datum + 1` (the bone key = `(index & 0x7fffffff) + extra` [M 0x1c3120:120]); the record's local
   quaternion/translation goes through `Quat_QuatTransToMat` + `Concat` with the palette entry
   [M 0x1c3120:127-131]; `muzzle_out` = the world matrix's translation row [M 0x1c3120:132-136].
5. The VU0 block transforms `(−1, 0, 0, 1)` by `M` (the row-scaled `_vmulabc/_vmaddabc/_vmaddbc`
   accumulation; the final `_vmaddbc` broadcasts vf00's hardwired `w = 1`, so the stack vector's
   uninitialised 4th slot is never read [M 0x17a5a0 0x17a664-0x17a68c]) and subtracts `muzzle_out`
   — `dir_out = −(M row 0)`, the muzzle bone's world −X axis [M 0x17a5a0 0x17a68c-0x17a6b8].

Failure returns 0 with all outputs either untouched or the position zeroed by
`AnimGetBoneWorldTrans`'s fallthrough [M 0x1c3120:168-171].

## Reachability (why it is on the shipped path)

`Drone_InitComms` 0x139520 runs `DroneVision_ProcessDroneSight` 0x176a28 every tick (the port folds
it in `Drones::tick`'s vision pass) → `DroneVision_SeekOpponentLOS` 0x175cb8:78 (per drone with the
`0x4F8 & 0x10000` want-sight flag, an opponent and visibility ≥ 0.7) → `DroneVision_GunThroughWall`
0x178520:40 → `DroneWeap_GetWeaponInfo` 0x17a5a0. [M] every link; the chain runs whenever any guard
sees the player, on every on-foot level.

## The port

`crates/nf-game/src/drone.rs`:

- `Drone::weapon_info(&d) -> Option<WeaponInfo>` — steps 1-5. `WeaponInfo { datum, sound, muzzle, dir }`.
  The `+0xC8 & 1` live-pose gate is the port's `palette()` None [S — the same substitution the
  muzzle/casing ports document]; the weapon byte is `Drone.weapon`; the mesh output (`+0x4C`) is not
  carried (the only shipped caller discards it — `d.weapons.muzzle_flash` reads the same field).
- `Drone::weapon_bone_world(&d, datum)` — step 4's core, shared with `Drone::muzzle()` (the draw-side
  stand-in maps a `0xFF` datum byte to datum 0; the AI path fails instead).
- `Drone::gun_through_wall(&d) -> bool` — `DroneVision_GunThroughWall` 0x178520: armed
  (`NDrone2_ObjectIsArmed` 0x178768: the weapon byte 0x6A–0x71, or 2–0x46 with 0x47 excluded [M
  0x178768:14-20]), then the ray. `muzzle_through_wall(world, pos, muzzle)` + the free
  `clamp_to_collision_cylinder` — `DroneFunc_ClampPosToCollisionCylinder` 0x1499d0: pull the x/z into
  the 0.4-radius cylinder around `obj+0x30` (normalise then ×0.4, y kept, only beyond 0.4) [M
  0x1499d0:30-43], then the segment clamped-start → raw muzzle; any hit = buried. The game's
  `Collide_RayIntersect` mask 0x927 picks its own collision sets; the port probes its standard
  segment sets (static + missile cage + dynamic) like its other vision probes [S — the
  `line_of_sight` note in `visibility()`].

## `Drone+0x1e` — the gun-through-wall flag

`DroneVision_SeekOpponentLOS` 0x175cb8:74-82: on every successful sight tick, `+0x1e = GunThroughWall()`
while the drone is aiming (`+0x22`, set by `NDrone2_DoTracking` 0x1479d8:49 when `AimTest` passes and
an opponent exists), else 0; ticks without sight leave the old value. The combat states gate every
fire request on `+0x3b = !+0x1e` before `DroneWeap_Fire`/`PlayFiringAnim`:

| State | Gate site |
|---|---|
| CombatNoRoute 0x164c08 | :63-73 |
| CombatWait 0x164960 | :40-46 |
| GoToGoalPosition 0x165640 | :101-107 |
| RunForCover 0x169150 | :88-94 |
| AimStandFire 0x166328 | :135-147 (the ENTER fire :61-66 is not gated) |
| AimCrouchFire 0x167060 | :72-99 |
| ProneFire 0x166a28 | :46-52 (the port has no prone fire state yet — not folded) |

The port stores the flag as `Drone.gun_through_wall`, refreshes it in the vision pass next to the
`dr.los` probe (the sight-tick fold), and gates `face_and_aim`'s fire request plus the
`AIM_STAND_FIRE`/`AIM_CROUCH_FIRE`/`COMBAT_WAIT` UPDATE fire calls.

## Tests

- `crates/nf-game/src/drone.rs` `weapon_dir_is_the_negated_world_row0` — the direction idiom on a
  hand-built matrix (unit row, the whole `M·(−1,0,0,1) − pos` form); sisters flip the axis.
- `crates/nf-game/src/drone.rs` `gun_ray_clamps_and_buries_in_a_wall` — the 0.4 clamp (within /
  beyond, y kept, per-bearing) and the wall ray on a synthetic `CollisionWorld`; sisters: the open
  ground is clear.
- `crates/nf-game/tests/weapon_info_live.rs` (ISO-gated) — the disc's weapon_data census: datum-less
  records exist and fail [D + M 0x1c7900:23-25]; `≥ 0x73` fails [M 0x1c7900:11]; an armed 07000005
  guard resolves datum/sound (with the 0 → 0xFFFF fixup)/muzzle (posed, within 2.0 of the guard, not
  the fallback)/unit `dir`; the open-air guard's through-wall probe is false.
