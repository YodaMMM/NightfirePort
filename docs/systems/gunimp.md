# The mounted guns (`GunImp`, placement type 0x34, class 0x47)

Build: ACTION.ELF `e7ffdcb3…`. Tags: [M] decomp/disassembly read here, [D] disc bytes,
[L] snapshot reads, [S] suggested. Port: `crates/nf-game/src/gunimp.rs` (the hardware table +
the aim state), `crates/nf-game/src/props.rs` (the 0x34 mount), `crates/nf-game/src/drone.rs`
(`Drones::gunimp_activate` / `gunimp_tick`), `crates/nf-view/src/scene.rs` (the use pick + the
mounted tick in `step_bond`).

## What they are

`parsemap_create_dynamic_objects` 0x1d04f0 case 0x34 → `GunImp_Create` 0x131f20; the object's
class byte is 0x47 [M 0x131f20]. The turret hardware is the `TurretGuns` table [D ACTION.ELF
@0x26f520 — one LOAD segment, vaddr 0x107100 = file 0x1000; 12 entries × 0x1c bytes]: a 12-byte
muzzle offset, the gun art hash (+0x0c), the weapon id (+0x10), the fire interval as raw f32
bits (+0x14), the fire sound (+0x18). Entry **0xb** is the laser: muzzle (0, 0, 1.71), art
0x02000b50, weapon **0x67**, interval 1.0 s, sound 1307. `GunImp_Create`'s single-player arm
reads 0xb [M 0x131f20:52-55 — `MPSettings+0x180 == 0` → index 0xb], so every SP mount is the
laser hardware.

0700000d carries one: #225 `PowerStationGunImpBase` (137.27, 9.33, −324.66, rotation identity
[D placements]) — the laser prototype's mount, the "Destroy door C5 with mounted laser"
objective. Placement params: p0 = 0x02000565 (an art hash also carried by TurretGuns entries
7–10), p1 = 103 (0x67) [D] — neither is read by `GunImp_Create`'s C [M]; the disassembly was
not re-checked this session [*S* — the SP arm's fixed 0xb makes both inert for 0d].

## The mounted mode

- **Use** (`Player_Activate` 0x1a5170's class walk reaching 0x47 → `GunImp_Activate` 0x132f00):
  `Player_ChangeSubState(player, 0xc)`, the player's BLData+0x878 = the imp, cam mode 0xe,
  `imp+0xf4 = 1`, the player slot into `imp+0xf6`, `Player_Disable(player, 1)`,
  `Input_SetAction(0xe)` and `Player_WeaponNone` [M 0x132f00]. Only from state 0; a player whose
  BLData+0xf6 ≠ 4 is refused (the decompile's guard) [M].
- **Update** (`GunImp_Update` 0x1320d8, state 1):
  - dismount: `Input_Action(slot, 0xe, 4)` → `GunImp_Deactivate` 0x132c18 (the saved substate
    back, cam mode 0, `Player_Enable`, the spherical aim handed to BLData+0x50) [M].
  - aim: yaw `ext+0x74` −= stick-x·FRAME_RATE_MUL·0.015, pitch `ext+0x70` −= stick-y·…·0.015,
    pitch clamped to [−π/2, +π/4] [M 0x1320d8:124-135]; the smoothed mirrors `ext+0x80`/`+0x84`
    ease 0.15 per tick [M 0x1320d8:136-161]. FRAME_RATE_MUL = 1.0 [L script clocks].
  - fire (action 9) when the cooldown `ext+0x94` has drained ≤ 0 [M 0x1320d8:271]: the cooldown
    refills from `TurretGuns[0xb]+0x14` (= 1.0 s), the heat `ext+0x90` += 2.0 (200 saturates the
    hot byte `ext+0x9c`), and **`Bullet_init(slot, player, imp, weapon_data + 0x67·0x10c,
    muzzle, dir, 0xffff)`** [M 0x1320d8:325-331]. The muzzle is the barrel matrix
    (placement transform — identity rotation on 0d — composed RotY(yaw_s)·RotX(pitch_s)) applied
    to the table's (0, 0, 1.71) offset [M 0x1320d8:196-211]; the direction is its +Z. The barrel
    also draws the beam quad 0x20005cd `PositionBeam(muzzle → ray hit at 1000 units)`, the
    0x200026e impact flare, a light, and `Explode_Propagate` at the hit for 0x67 [M 0x1320d8:355-437]
    — presentation the port leaves to the draw code (getters `muzzle`/`barrel_dir`/`heat`/`hot`).
  - the direct `Bullet_init` never runs `Player_WeaponInitBullet`'s pellet loop, so
    **no `PlrStat_LogShotFired`** [M 0x1a4bf0:258 sits in that loop] and no fire sound from the
    weapon's own path (the looped fire sound id 1307 is the imp's [M 0x1320d8:284-291]).

## The round and the door

Weapon 0x67 [L ee_start_05.bin weapon_data + 0x67·0x10c]: base 103, damage +0x0c = **50.0**,
range +0x1c = 500.0, 1 pellet, speed +0x20 = 16.667, spread +0x24 = 0.0, +0x68/+0x6C = 0,
ammo type 0. `Bullet_init`'s internal cone runs unconditionally (3 shared-RNG draws per shot,
spread 0) [M 0x12c5a0:171-180].

The C5 leaves `bigdoor01`/`bigdoor02` (#93/#96, 0xE1 `Destroy`, healths 1000/10000 = param 3,
out channel 0x1f = param 0 [D placements]) take every class-5 bullet contact:
`Destroy_CollisionHandler` 0x12bd48 subtracts the hit's damage from tag+4 and, below 0, writes
`switch_channels[param 0]` = ch 0x1f and smashes [M 0x12bd48:31-49]. Weapon 0x67 only adds a
burn-flash decay bonus there [M 0x12bd48:32-34] — **any** bullet drains the pool; the mounted
laser is simply the only practical 50-damage-per-second source on the route. Objective 6
(reveal 0x1d, complete 0x1f [L record 12]) then joins the AND #434 [0x1e & 0x1f & 0x28] →
ch 100 → the mission end to 0x730000d [M 0x182c60:102-103].

## The port

- `Props::create` mounts one `GunImp` per main-map 0x34 placement (placement order, the game's
  dynamic-object walk) with the placement transform as the base [M 0x131f20's base matrix].
- `Drones::gunimp_activate` / `gunimp_tick` carry the state machine above; the tick takes the
  raw stick axes, the fire bit and the use key, spawns the round through the player-bullet
  flight (the game's owner is the player [M 0x1320d8:327-331]) so the existing class-5 contact
  pass (`update_player_bullets` → `Props::bullet_hit`) damages the leaves.
- `scene.rs`'s use block picks the nearest idle gunimp under the head ray within the 2.2 reach
  stand-in (the SS arm's shape — the game's pick is one head-ray class walk [M 0x1a5170]);
  `step_bond` runs the mounted tick and zeroes Bond's input while a gunimp holds him
  (`Player_Disable`).
- The aim-integration constants (0.015/tick, the 0.15 ease, the clamp) are [M]; the stick-axis
  SIGNS against the port's `Input.turn/look` are the port's mapping [*S* unverified — the walk
  test drives the aim by feedback (turn = (yaw−want)/0.015), which is sign-agnostic].

## Tests

`crates/nf-view/tests/walkreal_0700000d_live.rs` walks to the mount, presses use
(`GunImp_Activate`), aims the barrel at bigdoor01 by the tick equations above and holds fire:
the door's health drains 50/shot and its death writes ch 0x1f — objective 6 completes, the AND
fires ch 100, the mission status reaches 6. The pre-fix tree has no gunimp: the use does
nothing, ch 0x1f stays off — the sister that fails.

## Live checks for the main agent

1. On 0700000d, stand at the laser prototype (≈(137, 8, −322)), press use: expect the mounted
   camera (cam mode 0xe) and no held weapon (`Player_WeaponNone`). gdbwatch `BLData+0x878`
   (expect the imp's obj handle) and `imp+0xf4` (= 1).
2. Hold fire on the C5 door for ~25 s: gdbwatch the door tag's health float (ext+4) — expect
   −50.0 per shot — and `switch_channels[0x1f]` flipping to 1 when it crosses 0.
3. The stick signs: pitch the aim up and read `ext+0x70`'s sign vs the stick (the port assumes
   stick-y negative raises the barrel).
