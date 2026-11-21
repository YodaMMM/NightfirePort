# Pickups: dropped weapons, ammo, Bond's inventory

Build: ACTION.ELF `e7ffdcb3…`. Tags: [M] read in our decomp export (work/decomp/ACTION.ELF) or the disassembly
(scratchpad `dis.py` over the ELF), [L] read in the level-start snapshot `work/ee_start_05.bin` (07000005),
[S] suggested.

## Who creates pickups [M]

- `DroneWeap_DropWeapon` (0x17a2a0) from `Death_Anim` 0x44's update (each tick while the firing flag `Drone+0x3B` is 0
  or the anim set `+0xDA` is 4 / 9), from `NDrone2_SetAsDead` (Dead's enter) and from the other death / surrender /
  stun states. It runs once: it clears the weapon byte `anim+0x62` (the drone draws no gun afterwards).
  - No weapon byte (≤ 0) or no third-person model (`weapon_data[w]+0x80` = 0): only the byte is cleared.
  - Matrix: `AnimGetBoneWorldTrans(obj, datum, 0, &v, &M)` with the datum of `AnimDatumGetWeaponInfo` (the weapon's
    datum, `+0x50`, 0xFF → 0; datum 0 when it reports none): the held gun's world matrix.
  - `Collide_RayIntersect(obj+0x30 → M.translation, mask 0x125)`: when anything is in the way the translation becomes
    the drone's position `obj+0x30` (the gun never spawns behind a wall).
  - Rounds: n = `weapon_data[w]+0x92` (the clip size); f = 0.5 + FRand(0.5) when n < 10, else 0.25 + FRand(0.5);
    rounds = (short)(int)(n · f). **One `Rand_FRand` per drop** (two for weapon 0x1A) — it advances the shared RNG.
  - `Pickup_CreateSimple(M, weapon_data[w]+0x02 (the base weapon), rounds, type 0 = weapon)`.
  - Weapon 0x1A also drops 0x1B (its grenade launcher) as type 1 (ammo), rounds `weapon_data[0x1B]+0x92` ·
    (0.25 + FRand(0.5)), with `obj+0xF0 |= 0x10` (see U-036).
- `DroneFunc_OnInitDeath` (0x145bd0): a captain (`Drone+0x19` ≠ 0) drops grenades at its own matrix (`obj+0x90`) by
  `DroneFunc_DropGrenade` → `Pickup_CreateSimple(…, id, n, 0)`: 07000001–08 → weapon 0x35 with 2; 07000009–0B,
  11–16, 4A → weapon 0x34 with 1 (`a2` = 1, read in the disassembly at 0x145de8).
- `Player_CheckForDeath` (0x190810:42-56), multiplayer only (`MPSettings+0x180` set): a human whose health runs out
  drops the gun in hand before `Player_WeaponNone` (:61). Gates: `weapon_data[held]+0x80` ≠ 0 (the held record has a
  world model) and `BLData+0x7E8` ≠ 0 (the view model). `Pickup_CreateSimple(view model +0x90, weapon_data[held]+0x02
  (the base), BLData+0x1B8 + base·0xC (the rounds in the base's clip slot), type 0)`: the drop carries the clip only,
  not the carried reserve, so an empty clip drops nothing; no RNG draw. Base 0x1A also drops 0x1B as type 1 with the
  rounds of `weapon_data[0x1B].id`'s clip slot, hidden (`obj+0xF0 |= 0x10`). No scenario word is read on this arm.
  With the MP lifetime below the drop lasts 1800 ticks (30 s) and spins. Anyone alive takes it: the players' loop
  (the dropper too once respawned), then the bots.
- MP bots die through `NDrone2_DSTATE_BotDeathAnim` 0x175508 (update case 3) / `BotDeathByExplosion` 0x1755a8, which
  call `DroneWeap_DropWeapon` like the guards (with its `Rand_FRand` draws). A round's or a punch's death enters 0xF2
  (`NDrone2_DealWithObjHit` 0x13fff0 case 0x27's MP arm, `NDrone2_PunchImpact`/`BulletImpact`), and its UPDATE drops;
  a blast death enters 0xF3 (`NDrone2_ExplosiveImpact` 0x146480's MP arm, `param_3 = 0xf3`), whose ENTER drops right
  after `MP_PlayerKilled`. `Drone_SM_SetState` 0x1758c0 only queues the state; the route applies it (EXIT, ENTER).
  The held byte is `anim+0x62`, written with `Drone+0x768` by `BOTWEAP_changeWeapon` 0x12a028. Ported (mp-bot-drops):
  `Sim::mp_bot_death_drop`; the shot death drops on the next tick's bot pass, the blast death on the kill; the gun's
  matrix is the bot body's resting gun matrix (the human drop's stand-in), not the posed datum.
- Level placements (`parsemap_create_dynamic_objects`), scripts (`Script_CreateEntity`), `GT_Update`,
  `Pickup_CreateFromSet` (types 7–11): see the port's loaders.

## `Pickup_CreateSimple` (0x18f348) [M, disassembly]

- Rounds 0 → nothing. Id remap: 0x6A / 0x6E → 0x33; none for 1, 0x45, 0x47, 0x50, 0x51, 0x53 or id − 1 > 0x70.
- `Pickup_Create(NULL, NULL, M, glist(weapon_data[id]+0x80), type, weapon_data[id]+0x00, rounds, switch 0,
  sound 0xF5, rest 0 (drop to the floor), respawn 0, lifetime FRAME_RATE_INT · (MP ? 30 : 120), label 0)` — the stack
  arguments read at 0x18f43c–0x18f488. FRAME_RATE_INT = 60 in the snapshot [L], so a dropped gun lasts **7200 ticks**
  (2 minutes) [L+M]. The MP test is `MPSettings+0x184` (0x2a4924, the master copy `MP_Start` writes): in a match
  the lifetime is **1800 ticks** (30 s) [M 0x18f440-0x18f47c].

## `Pickup_Create` (0x18dda0) [M]

- No glist, id −1, or type 6 (Bond bonus) without the gold medal → nothing. Types 7–11 → `Pickup_CreateFromSet`.
- `control_create_object(0x40)`; `obj+0x90` = M; PICKUPINFO at `obj+0xE0`: `+0x00` velocity (zero), `+0x10` offset
  translation − sphere centre, `+0x20` state (0 falling, 1 resting, 2 waiting to respawn), `+0x22` type, `+0x24` id,
  `+0x26` rounds, `+0x28` switch channel, `+0x2A` sound (0xFFFF none), `+0x2C` respawn, `+0x2E` lifetime, `+0x30` −1,
  `+0x34` label, `+0x38` spin (type 6 or multiplayer).
- `obj+0xF0` = glist flags & ~0x2000 | 0x840; `obj+0xFA |= 0x44`; `obj+0xFF` = 0x2F.
- Position: `build_PointOnFloor(1.0, cel, …)` on the translation → `obj+0x30` / `+0x40`.
- Rest 0 (drops): the matrix is turned upright (`Mat_Align2Up(M, (0,1,0), Mat_GetDir(M))`: it keeps its heading),
  pushed out of overlapping geometry (`Collide_SphereIntersect`, mask 0x50D), lowered by radius − |glist centre y|,
  state 0. Rest ≠ 0: state 1.

## `Pickup_Update` (0x18e148), every tick [M]

- State 0 (falling): velocity += `WldGravity` (0, −9.8, 0) [L] · `REC_FRAME_RATE`² (1/60 [L]); translation +=
  velocity. Ray from `obj+0x40` to translation − (0, |glist centre y|, 0) (mask 0x50D); when it hits and the
  translation is below the hit: position = hit + `HIT+0x30` · (−|centre y|) + info `+0x10` (see U-036), velocity
  zeroed, state 1. Hits on moving objects (types other than 2, 5, 0x11, 0x1A) give the pickup their velocity
  (`Control_InheritVelocity`, 3.0) and keep it falling.
- State 2 (taken, respawning): it returns while `GameState+0x34 − obj+0xEC ≤ FRAME_RATE_INT · respawn · 10`, so it
  shows again (`obj+0xF0 &= ~0x10`, state 1) on the tick after the limit: limit + 1 ticks after the take [M
  0x18e148:183-189]. Nothing else that tick.
- Spin (`+0x38`): about +Y by `REC_FRAME_RATE` radians per tick.
- Touch: for each player (`glb_players`, alive `obj+0xF4` = 1, not type 0x12): squared distance from the pickup's
  sphere centre (`obj+0x80`) to the player's position (`obj+0x30`) < (radius `obj+0x8C` + 1.1)² and
  `Collide_LineOfSight(player obj+0x80 → pickup obj+0x80, mask 0x721)` → `Pickup_Handler`, and nothing else this tick.
  (Multiplayer bots likewise.)
- Lifetime (`+0x2E`): when non-zero, −1 per tick; at 0 the pickup is deleted (`obj+0xFE |= 1`).

## `Pickup_Handler` (0x18e650), Bond [M]

- Id: `Upgrade_Weapon(id)` in single player (6 → the upgraded handgun, 0x1E / 0x24 snipers, 0x43, 0x4A, 0x4E, 0x56 by
  the upgrade bytes `0x2BEEAA…`; 10 → 0xC with `GameState+0x52`; others unchanged).
- In a match there is no `Upgrade_Weapon` [M 0x18e650:141-160]: when the taker's character row
  (`BOT_getDefaultStats(MPSettings[slot·0x30+0x24])`, byte 9) is 1, a 10 becomes 0xD if the taker already owns 10
  (`BLData+0x232`); otherwise a 0xD becomes 10; then `Upgrade_MPWeapon(slot, id)` (2/4 → 2, 6/8 → 6, the snipers to
  their base, 0x50). So a P99 (6) pickup stays record 6 and a finished game does not turn 10 into 0xC.
- Type 0 (weapon): Bond without it (`BLData+0x1BA + id·0xC` = 0) → `Player_EquipWeapon`; with it →
  `Player_EquipAmmo` (message as ammo). Type 1 (ammo) → `Player_EquipAmmo`. Either returning 0 leaves the pickup
  where it is. Type 2 → `Player_EquipKey`; 3 → armour 50 (`+0x8B0`, only when below 50); 5, 6: special.
- Message: weapon "%s" (label `0x200000D` with `weapon_data+0x38`), ammo "%d × %s" (`0x200000E`, `ammo_data+8` of
  `weapon_data+0x90`), shown 180 ticks (`Text_AddMsg`). Weapon / ammo pickups get sound 0xF5.
- Switch channel `+0x28` → `switch_channels` = 1 with the time.
- `Sound_Play3D(sound, at the translation, 100.0, −1, −1)` — a sound the drones may hear (`Sound_Alertness`).
- Respawn 0 → deleted; −1 → stays; else hidden, state 2, stamp `obj+0xEC`.

## Bond's inventory (BLData = player `obj+0xE0`) [M, L]

- `+0x170 + 2·t`: rounds carried per ammo type t (`weapon_data[w]+0x90`); cap `ammo_data[t]+0x02` (12-byte records
  at 0x2C69B8). 07000005 start [L]: type 1 = 41, all others 0.
- `+0x1B8 + 0xC·w`: s16 rounds in weapon w's clip; `+0x1BA`: owned. 07000005 start [L]: owned 1, 2 (P2K, clip 7),
  0x4A, 0x4E, 0x50, 0x54 (gadgets).
- `Player_EquipWeapon` (0x1a8a58): needs `weapon_data[id]+0xDC` loaded (`hashtable_getitem`, the first-person model;
  else refused). Not owned (by the base `+0x02`): owned, clip = `+0x92` (a full clip), carried += rounds + old clip −
  clip size when positive. Owned: refused when carried ≥ cap or rounds = 0, else carried += rounds. Carried clamped
  to the cap. With auto-switch on (`0x2A38D2`) and `Player_IsBetterWeapon`: `anim+0x63` = id (switch request).
- `Player_EquipAmmo` (0x1a8d18): carried < cap and rounds ≠ 0 → carried += rounds (flag `+0x68 & 0x10` weapons fill
  an empty clip first), clamped; returns the rounds taken (0 = refused).
- 07000005 [L]: guards hold SG5 (0x16: base 0x16, clip 30, ammo type 5, cap 90, model 0x02000B44); the sniper holds
  0x24 (clip 5, ammo type 7, cap 30). Bond's P2K uses type 1 (cap 70).

## Weapon upgrades (`Upgrade_Weapon` 0x1a8f30, `Menu_SetBonus` 0x1ff898) [M, L, D]

- `Upgrades` (0x2beea8, 4 players x 9 bytes). `Menu_SetBonus` rebuilds a player's row whenever the bonus word is
  written (save load, results award, codename/cheat paths): `ResetUpgrade`, then for each of the 12 `sp_level`
  hashes and counters 1..4, a kind-4 (gold) reward the player holds sets `Upgrades[bit / 3]` = the set count of its
  reward group [M 0x1ff898:73-98, `PlrStarts_ProcessRewardCounter` 0x199210, `Set_Upgrade` 0x1a98a8].
- `Upgrade_Weapon` (single player, row 0) by its jump table at 0x2fc9a0 [M disasm 0x1a8f5c-0x1a9038]:
  6 → `UpgradedHandguns[row[0]]` = [2,4,6,8] (gold group bits 0..2: 07000005, 07000009, 07000011);
  0x1e / 0x24 → snipers [0x1e,0x20,0x22,0x22] / [0x24,0x26,0x28,0x28] by row[3] (bits 9..10: 07000001,
  0700001B); 0x43 → [0x43,0x44,..] by row[4] (bit 12: 0700000C); 0x56 → [0x56,0x57,..] by row[5] (bit 15:
  09000002); 0x4a → [0x4a,0x4c,..] by row[6] (bit 18: 09000003); 0x4e → [0x4e,0x4f,..] by row[7] (bit 21:
  07000014); 0x54 keeps its id and writes records 0x54/0x55 `+0x88` (zoom) = 16.0 when row[2] != 0 (bit 6:
  09000006), else 8.0; 10 → 0xC with `GameState[0x52]` (game complete). Tables [L ee_start_05 0x2bee08..].
- Callers: `Pickup_Handler` (the take id), `Player_EquipWeapon` (SP: its id goes through it after the model check
  [M 0x1a8a58:24-30]), `Player_EquipAmmo` (0x1f / 0x25 → upgraded base + 1), `Player_InitWeapon` (the kit's
  handgun, stunner, sniper, dart gun, decryptor, the tail's laser [M 0x1a7ef8:126-133]) and `Player_Weapon`'s
  07000007 re-arm (`Upgrade_Weapon(6)` [M 0x1a24c8:248]).
- Every level whose kit holds a handgun ships the four handgun first-person models 0x05000088 / 0x050000a9 /
  0x05000047 / 0x050000a8 [D the archives' skin tables, all of `dev::LEVELS`], so the upgraded kits load.

## Level kit [M]

- `Player_InitWeapon` 0x1a7ef8's equips go through the single-player `Player_EquipWeapon`, which runs
  `Upgrade_Weapon` on the id [M 0x1a8a58:24-30]. On a finished game (`GameState[0x52]`) the default arm's
  `EquipWeapon(10, 999)` therefore grants record 0xC.

## Port

`nf_game::pickup` (WORKLOG 0040 and later): drops from Death_Anim / Dead, captain grenades, falling, resting, touch,
lifetime, respawn (state 2), spin, the handler for every kind (weapon, ammo, key, armour, label, message, bonus) with
`EquipWeapon` / `EquipAmmo`, the take messages and sounds (scene.rs), the auto-switch through `is_better_weapon` and
the `PlayerSetting+0xA` flag, and the weapon upgrades: `Inventory::upgrades` is the row
(`progress::upgrades_from_bonus`, set at level load from the profile's bonus word), read by the take, the
auto-switch, the level kit (`player::init_level_loadout`, whose `equip` runs the SP `Upgrade_Weapon` remap) and the
07000007 re-arm. The 0x54 zoom is applied once at level load (`WeaponData::apply_upgrade_zoom`) rather than on each
equip. The health-pane fade reset happens on the armour take only [M 0x18e650:220-226].

*Deviations / not ported*: the rest height (U-036), no sphere push-out at creation, static world only (no moving
platforms), `hashtable_getitem` taken as the level skin table.
mp-drops:
- `Pickup::player_death_drops` / `create_simple_mp` (nf-game pickup.rs) build the `Player_CheckForDeath` drop; the
  scene calls them on the death tick: offline from the local player's death start (`Sim::step_bond`), online from
  `step_net_combat`'s dead-wait loop on the slot's first dead tick, using only shared state (the slot's shot gate:
  held weapon and inventory, and its body), so every machine creates the same pickup.
- The drop's start matrix is `Sim::rest_gun_matrix`: the `Player_PositionGun` pose without the bob and the crosshair
  tilt [S] (those are per-machine view state). The fall then puts it on the floor.
- The bots' pickup rows append a new pickup instead of rebuilding (older visit stamps survive), and a falling row
  is not available until it lands (the game registers on landing).
- Drawing (mp-drops-draw): the renderer draws pickups per index from slots built at load, so the match load
  reserves inert (`gone`) slots for the drops after the placed and script pickups: every weapon a human can hold
  (the set row, its spawn column through `Upgrade_MPWeapon`, the arena's weapon pickups, the grapple, 0x3B, with
  the take remaps), each through `player_death_drops` itself, 2 per human per weapon (`Drones::mp_drop_slots`). A
  death fills a free slot of its id (never-used first; `Drones::mp_drop_used` keeps the bots' row order); no free
  slot falls back to the old undrawn push and logs it. The AR's hidden launcher rounds still push (never drawn).
- The match take id is `Inventory::take_id(raw, Some(dual), _)`; the local take, the online take and the shared gate
  use it. The online path used to pass the tick count as the game-complete byte (every 10 became 0xC) and ran the
  single-player handgun table (6 became 2).
- The online respawn wait is limit + 1 ticks like the offline one.
- The kit's 10 → 0xC on a finished game: `init_level_loadout_ex(.., game_complete)`, fed by the app's level loads
  (`load_front_end_gc`). The other `Upgrade_Weapon` tables need the `Upgrades` row (not in this tree).
- The bots' own death drops (mp-bot-drops): `Sim::mp_bot_death_drop` from the bot death states (one `Rand_FRand`
  per drop, two for the AR, on the shared stream), through the reserved draw slots (2 per body per weapon, humans
  and bots). Not ported: `MP_RegisterPickup`'s 64-slot cap and its eviction of the shortest-lived
  registered pickup; the Professional Mode one-gun swap in the match take (`MPSettings+0x1BC`, 0x18e650:168-200).
