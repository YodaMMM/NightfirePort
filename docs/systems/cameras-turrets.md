# Cameras & turrets: the census over the on-foot single-player levels

Scope: every placement whose creator makes a security camera, a wall/ceiling turret, a gun
emplacement or an auto-gun, on the campaign's on-foot levels (the `CAMPAIGN` table,
crates/nf-view/src/menu.rs — every mission's parts). Evidence tags: [M] decompile (function +
address), [D] disc bytes, [L] live EE read, [S] suggested.

## 1. The creator families [M]

`parsemap_create_dynamic_objects` 0x1d04f0 dispatches on the placement kind; every dynamic
object case is gated by the same spawn flag (param_7 — the port skips `flags & 0xa000`).
Five cases build the camera/turret family; the class byte then dispatches the per-tick update
through `control_funcs` (`control_movement_object_handler` 0x1336f8):

| case | creator | class | update (control_funcs) | what it does | what it draws |
|---|---|---|---|---|---|
| 0x27 | `GT_Create` 0x17ffa0 | 0x36 | `GT_Update` 0x180498 (`GT_Delete` 0x181678) | the auto-gun turret: 8-state machine (DORMANT/ARMING/RESTORING/FINISHED/COMBAT/DISABLED_SWEEP/DESTROYED/MOUNTED), two looping script anims per turret (`Script_Load` of params 2/3), `GT_Track` 0x17f9c8 target scan (range 50 SP, front-cone dot ≥ 0.01 vs the create-time `ext+0x80` direction, one `Collide_LineOfSight(…, 0x72f)` per candidate at 0x17f9c8:258,271-278), fires weapon record 96 every 0.1 s with `Bullet_init` owner −1, `GT_ApplyDamage` 0x1816d8 (health 100.0, or 0xffff invulnerable when param 0 == 0), `GT_Disable` 0x1816a8 | the placement's own glist mesh (wall gun 0x200064c, minigun 0x2000359), hidden until deployed (obj+0xf0 bit 0x10), the spinning barrel (`GT_Register` 0x181538's 0xC4 entry, ext+0xd4/d8), the muzzle flash + wreck smoke `Gas_Init` (update case 6) |
| 0x28 | `Sensor_Create` 0x1af010 | 0x37 | `Sensor_Update` 0x1af618 | the security camera: sweeps its 0x19 key-list poses, watches a cone (`Sensor_InCone` 0x1af4f0, player-only), after 31 in-cone ticks tracks (`Vec_Track` 0.05) and after 2 s of sight writes **1 to its OUT channel (param 3)** — the alarm signal the door/script systems and the drone AI (`DroneFunc_CheckAlarmRaised` 0x1453a8) listen for; the disable channel (param 4) turns it off; `Sensor_ApplyDamage` 0x1b08a8 is an empty stub — cameras take no damage | the lens (param 0) and mount (param 7) gfx proxy entities with their hashtable meshes, the destroyed-glist swap (obj+0xD8), `Sensor_SetBeam` 0x1af398's sight beam; the 0x4AA hum / 0x4DD servo / 0x4A9 beeps |
| 0x30 | `Monitor_Create` 0x1b08b0 | 0x38 | `Monitor_Update` 0x1afcc8 | the camera-console: use key → `Monitor_Activate` 0x1b0058 picks a CameraList sensor of the same group (monitor p0 == sensor p6) and views it, writing the sensor's monitor-view channel (p8) every tick; action cycles, exit restores; `Monitor_ApplyDamage` 0x1b0968: health (1.0 when p1 == 0, else immune) below 0 → state 3 + `Explode_Create(…, script 0x6000027, 200, 200)` | the console's glist mesh; the viewed camera's picture is the draw side's work |
| 0x34 | `GunImp_Create` 0x131f20 | 0x47 | `GunImp_Update` 0x1320d8 | the use-mounted gun emplacement: activate → cam mode 0xe, the imp owns the tick (aim from the stick, pitch clamped [−π/2, +π/4], fire action 9 through the `TurretGuns` table [D ACTION.ELF @0x26f520] cooldown/heat); the SP arm mounts entry 0xb — the laser (weapon 0x67) [M 0x131f20:52-63]; the round is `Bullet_init(player, imp, weapon_data + 0x67·0x10c, …)` | the base glist mesh (the placement's own object) + the **gun child**: `GunImp_Update` 0x1320d8:162-172 composes base·RotY(yaw_s)·RotX(pitch_s) on the `TurretGuns` art (SP: 0x02000b50) |
| 0xd2 | `Shooter_Create` 0x17d030 | 0x4e | `Shooter_Update` 0x17d0c0 | the one-shot fixed auto-gun: while its arm channel (p0, obj+0xF4) is set it fires ONE `Bullet_init(0, 0, obj, weapon_data + p1·0x10c, obj+0x30, dir)` (dir = the object matrix's forward, or straight down (0,−1,0) when the weapon is the 0x3b grenade) and sets obj+0xFE bit 0 (self-delete) | nothing — `control_create_object(0, …)` builds an object with no glist; the game's shooter is invisible |

Adjacent but out of scope: the searchlight (case 0xde, `Searchlight_Create`, its own system —
crates/nf-game/src/searchlight.rs, docs/atlas/searchlight.md); the drone placements (case 0x0f)
create the character AI, not cameras — see §2.

## 2. "Drone templates that are cameras or turrets" — there are none [M]

One might expect camera/turret templates among `Drone_Create` 0x136be0 / the NDrone2
classes. The decompile says otherwise: `Drone_Create` 0x136be0 copies the placement's 0x84
bytes of character data into a `DCVars` AI record (`NDrone2_CreateObj`), and every DMODE
initializer (`NDrone2_init_DMODE_*`, 0x14d528..0x14e568 — Defaults, Normal, Guard, Retreater,
Sniper, Stealth, Attacker, RunToPoint, Assassin, HostageKiller, Hostage, HostageTied,
JustStand4Demo, DeleteMe, Civilian, CivilianScared, MissionFailer, Mayhew, Ninja, AlarmRaiser,
SearchLight, Ambush, Zoe, PartyGirl, CivilianGuard, Interogator, CivDoorGuard, TruckDriver,
CastleChatGuard1/2, SniperAlert, PartyGirlLooker, Bot) configures a **character** behaviour
(patrol/attack/talk); the `SearchLight` DMODE is the AI of a guard **manning** a searchlight
object, not a camera. No NDrone2 class draws a sweep cone or fires a turret weapon. The
cameras and turrets are exactly the five placement kinds of §1.

## 3. The disc census [D]

Read from this ISO's level bins (main map placements, kind = `flags & 0xff`, spawn-gated,
placement indices from the parse order). Count rows are `docs`-safe repeats of
`crates/nf-game/tests/cameras_turrets_census_live.rs`, which re-derives them on every run.

| mission (parts) | bin | GT 0x27 | sensor 0x28 | monitor 0x30 | gunimp 0x34 | shooter 0xd2 |
|---|---|---|---|---|---|---|
| Double Cross 1 | 07000001 | – | 1 | 1 | – | 1 |
| Double Cross 2 | 07000002 | – | 2 | 1 | – | – |
| Double Cross 3/4 | 07000003/04 | – | – | – | – | – |
| The Exchange 1 | 07000005 | – | – | – | – | – |
| The Exchange 2 | 07000006 | – | – | – | – | – |
| The Exchange 3/4 | 07000007/08 | – | – | – | – | – |
| Night Shift 1 | 07000009 | – | – | – | – | – |
| Night Shift 2 (Tower B) | 0700000a | 6 | 2 | 1 | – | – |
| Night Shift 3 (Tower C) | 0700000b | 5 | 2 | – | – | – |
| Chain Reaction 1 | 0700000c | – | – | – | – | – |
| Chain Reaction 2 | 0700000d | – | – | – | 1 | – |
| Phoenix Fire 1 | 07000011 | – | – | – | – | – |
| Phoenix Fire 2 | 07000012 | 3 | 5 | 1 | – | – |
| Phoenix Fire 3 (elevator) | 0700004a | – | – | – | – | 2 |
| Phoenix Fire 4 | 07000013 | – | – | – | – | – |
| Countdown 1 | 07000014 | – | 14 | 12 | – | – |
| Countdown 2 (silo) | 07000015 | – | – | – | – | 1 |
| Countdown 3 | 07000016 | – | – | – | – | – |
| Equinox | 0700001b | – | – | – | – | – |
| Deep Descent part | 07000041 | – | – | – | – | 1 |
| MP arenas | 07000023/24/25/26/27/29/4b | – | – | – | 2 each | – |
| MP/other | 07000046 | – | 3 | 1 | – | – |
| other MP/unused bins (07000042-07000049, 0700004c+) | scanned — none carry camera-family kinds | | | | |

Notable rows (placement #, hash, position, params):

- **Night Shift 2's six wall guns** (#636-#641, glist 0x200064c, "GunSamuri-Placement"):
  all facing 1 (point **straight down** — the create-time `ext+0x80` = −up of the placement
  matrix [M 0017ffa0:141-147]), HP 0xffff (param 0 = 1 → invulnerable), enable channels
  0x12/0x11/0x10/0xf, scripts 0x06000041/0x06000042 (the looping deploy/fold anims).
- **Night Shift 3's five**: #174/#175 (y 49.10, enable ch 0xe) and #291-#293 (exec floor,
  enable ch 0x1c, **disable ch 0x4e** — the office switch kills them).
- **Phoenix Fire 2's three miniguns** (#654/#655/#681, glist 0x2000359, facing 0 = forward,
  HP 100, scripts 0x06000045/0x06000044, no enable channel — armed at load).
- **Countdown 1's security centre**: 14 cameras + 12 monitors, the monitor groups p0 = 1/2
  matching the cameras' p6, camera p7 mount 0x2000652, cone 15° range 5, out channels
  0x3d..0x4d, disable 0x78 (+#314/#782 pair with disable 0x53).
- **The 0700000a/0b monitor pairs**: each console (p0 = 0xa) watches the two Tower
  cameras (p6 = 0xa, out 0x1d/0x1e on 0a; #164/#172 out 0x28/0x27 on 0b).
- **Chain Reaction 2's laser mount** #225 (hash 0x2000b51, p0 0x2000565 p1 0x67): the
  power-station mounted laser, the port's GunImp fixture.
- **The MP gun emplacements** ×2 per arena (hash 0x2000566, p0 0x2000a3b p1 0x60 = the
  TurretGuns[1] 0x60 weapon).
- **Shooters**: Double Cross 1 #329 (arm ch 0xd, weapon 0x36), 07000041 #111 (arm ch 0x1,
  weapon 0x36), Phoenix Fire 3's elevator pair #286/#287 (arm ch 0x22/0x4f, weapon 0x3d — the
  grenade type: the update's straight-down branch [M 0x17d0c0]), Countdown 2 #389 (arm ch
  0xfb, weapon 0x34).

## 4. The port side (all rows verified against real level data)

| family | created | updates | alerts/fires | damage/destruction | mesh to draw lists | pin |
|---|---|---|---|---|---|---|
| GT turret | yes (`drone.rs` kind-0x27 walk → `gt::from_placement`) | yes (`GT_Update` port, `gt.rs::update`, the script parks/rewinds) | yes — `GTEvent::Fire` spawns the weapon-96 bullet (owner −1) + the muzzle sound; `GT_Track` port incl. the front cone and, since this task, the per-candidate `Collide_LineOfSight` 0x72f through `d.world` (was stubbed `los_ok = true` — the tracked gap) | yes (`apply_damage`, `disable`, delete; the 0xffff HP selector) | the draw gate honours the hidden bit 0x10 (scene.rs); the barrel spin + smoke/burst events reach the renderer as `GTEvent`s | `crates/nf-view/tests/gun_turrets_live.rs` (DORMANT at load, ch 18 → COMBAT, ammo + Bond damage), **`crates/nf-view/tests/gt_los_live.rs` (new: a wall-blocked candidate holds the gun DORMANT — fails on the pre-fix tree)**, `gt.rs` 13 unit tests |
| security camera (sensor) | yes (`sensor.rs::create`, kind 0x28) | yes (sweep/track/return/disable states) | yes — the OUT channel after 2 s of sight, the alarm consumers read it | n/a — the game's `Sensor_ApplyDamage` 0x1b08a8 is a stub; cameras are indestructible [M] | **no** — the lens/mount proxies and the beam are unported (draw hand-off §5) | `sensor_radius_live.rs` + `sensor_live.rs` (0700000b real cone/range gates) |
| monitor console | yes (`monitor.rs::create`) | yes (`tick` — the watched sensor's channel + the cooldown states) | yes — activate/cycle/exit through the real use path (`drone.rs:14028`) | yes (`bullet_hit` → state 3, wired at `drone.rs:13130`) | **no** — the console mesh + the viewed picture are draw-side | `monitor.rs` unit tests (the 07000001 group-1 fixture) |
| GunImp | yes (`props.rs` kind 0x34) | yes (`gunimp_tick` — aim/heat/cooldown, the mounted pose) | yes — action 9 fires the weapon-0x67 round through the player-bullet path | the game's imp is indestructible (no damage path in 0x1320d8) [M] | base yes; **the gun child (TurretGuns art) no** — draw hand-off §5 | `props.rs` + `drone.rs` gunimp unit tests, `0700000d` live fixture |
| shooter | yes (`worldobj.rs` kind 0xd2) | yes (channel-armed one-shot + self-delete) | yes — `Bullet_init` with p1's weapon, the 0x3b straight-down branch | self-deletes after firing (obj+0xFE bit 0) [M 0x17d0c0] | n/a — the game's shooter has no glist [M 0x17d030 arg 0] | `worldobj.rs` unit tests |

Census pin (all 11 census levels): `crates/nf-game/tests/cameras_turrets_census_live.rs` —
the port's five creators produce exactly the [D] placement counts above, per level.

## 5. Draw hand-off (not this task's — for the renderer)

- `// draw hand-off:` GunImp gun child — `crates/nf-game/src/gunimp.rs` `barrel_matrix()`
  already returns the exact matrix the game composes (base·RotY(yaw_s)·RotX(pitch_s),
  [M 0x1320d8:162-172]); draw the `TurretGuns[turret].art` glist (SP 0x02000b50 laser; the
  MP entries' 0x02000a3b/0x02000b05/0x02000565 per the table) at that matrix, plus the base
  glist the placement's own object provides.
- `// draw hand-off:` Sensor lens/mount/beam — `crates/nf-game/src/sensor.rs` (header block):
  param-0 lens (0x2000360) and param-7 mount (0x2000652) proxy entities with their hashtable
  meshes, the destroyed-glist swap, and `Sensor_SetBeam`'s sight beam; the sweep angles are
  the port's `matrix`/`quat` fields.
- `// draw hand-off:` GT barrel spin + wreck — `crates/nf-game/src/drone.rs`'s
  `GTEvent::Smoke`/`DestroyedBurst` arm (currently `{}`): `Gas_Init(0.05, 0.25, glist
  0x2000039, 7, 0x48403080, 1)` [M 0x180498 case 6], and the registered barrel object's
  ext+0xd4/d8 spin angle for the renderer.

## 6. Live checks (optional confirmation)

1. `gdbwatch` on `Collide_LineOfSight` 0x1edf50 during 0700000a with ch 18 raised and Bond
   behind the bridge structure in gun #636's cone: expect the call with filter 0x72f to
   return 0 while he is walled (no deploy), and non-zero on the open deck below the mount.
   The port's `gt_los_live` pins the same behaviour through the static world.
