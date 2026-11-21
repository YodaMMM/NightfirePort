# s/mp-drops: multiplayer death drops, the online respawn wait, the weapon-10 remaps

No change here moves the shared random stream: the human death drop draws nothing
(`Player_CheckForDeath` has no `Rand` call on the drop arm), and no existing draw was added,
removed or reordered. Logic only, no draw code changed.

## What a player notices

- **In a match, a player who dies drops the gun in hand.** It falls from where the gun was held,
  spins, and lasts 30 s. Anyone alive can pick it up: players first, then bots; the dead player
  too once respawned. It carries the rounds that were in the clip (not the spare ammo), so a gun
  with an empty clip drops nothing. The assault rifle also drops its grenade rounds as a hidden
  ammo pickup. Online, every machine creates the same pickup on the same tick.
- **Online, a taken arena weapon comes back one tick later than before**, on the game's tick.
- **Online, picking up the P99 gives the P99.** The online take ran the single-player handgun
  table (6 became the P2K record 2) and fed the tick count in as the game-complete flag, so every
  weapon-10 pickup became record 0xC. A match takes through the game's own MP remap now. Offline
  matches take the same way.
- **On a finished game, the give-everything kit holds record 0xC instead of 10.** This arm only runs
  on levels outside the kit table.

## Evidence

- Drop: `Player_CheckForDeath` 0x190810:42-56 [M]. Held record = `anim+0x62`; gates
  `weapon_data[held]+0x80 != 0` and `BLData+0x7E8 != 0`; `Pickup_CreateSimple(view model +0x90,
  weapon_data[held]+2, BLData+0x1B8 + base·0xC, 0)`; base 0x1A adds `(…, 0x1B, clip of
  weapon_data[0x1B].id, 1)` with `obj+0xF0 |= 0x10`. The drop runs before `Player_WeaponNone`
  (:61). No scenario test sits on this arm.
- Lifetime: `Pickup_CreateSimple` disasm 0x18f440-0x18f47c: `FRAME_RATE_INT · (MPSettings+0x184 ?
  30 : 120)` into `Pickup_Create` param 12 (`+0x2E`). The stack arguments: 0xF5 sound, rest 0
  (falls), respawn 0 (deleted when taken). +0x184 is the master copy `MP_Start` writes.
- Spin: `Pickup_Create` 0x18dda0:138-144 sets `+0x38 = 1` for every pickup while
  `MPSettings+0x180` is set; `Pickup_Update` 0x18e148:193-201 turns it by `REC_FRAME_RATE` per tick.
- Takers: `Pickup_Update` 0x18e148:203-246: the players (`+0xF4 == 1`, class not 0x12), then the
  bots while +0x184 is set.
- Registration: `MP_RegisterPickup` runs on the landing (0x18e148 fall arm), so the bots' row for
  a drop is available only once it rests.
- Respawn: `Pickup_Update` returns while `elapsed <= FRAME_RATE_INT · respawn · 10`
  (0x18e148:183-185), the stamp written by `Pickup_Handler` 0x18e650:345-347: the restore is
  limit + 1 ticks after the take. The offline path already counted that; the online arm counted
  limit.
- MP take id: `Pickup_Handler` 0x18e650:141-160: `Upgrade_Weapon` only when `MPSettings+0x180 == 0`;
  in a match `BOT_getDefaultStats(MPSettings[slot·0x30+0x24])` byte 9 == 1 turns a 10 into 0xD when
  `BLData+0x232` (owned 10) is set, else a 0xD becomes 10; then `Upgrade_MPWeapon` 0x1a9040.
- Kit: `Player_EquipWeapon` 0x1a8a58:24-30 runs `Upgrade_Weapon` in single player; its case 10
  returns 0xC while `GameState[0x52]` is set [M 0x1a8f30].

## Changes

- nf-game `pickup.rs`: `MP_DROP_LIFETIME`, `Pickup::create_simple_mp`, `Pickup::player_death_drops`,
  `Inventory::take_id` / `take_ex` (the match take id).
- nf-game `drone.rs`: the local take and the online take use the match id in a match; the online
  respawn wait is limit + 1; `NetBody::dual`, `Drones::mp_take_dual`.
- nf-game `mp/combat.rs`: `gate_take` takes the character flag instead of a clock.
- nf-game `mp/mod.rs`: `MpSettings::dual_wield(slot)`.
- nf-game `player.rs`: `init_level_loadout_ex(.., game_complete)`.
- nf-view `scene.rs`: the drop on the death tick (offline from the local death start, online from
  `step_net_combat`'s dead-wait loop, shared gate state only); `rest_gun_matrix`; the bots'
  pickup rows append instead of rebuilding and wait for the landing.
- nf-view `lib.rs`: `load_front_end_gc` carries the profile's game-complete flag into the kit; the
  app's level loads use it.
- Docs: `docs/systems/pickups.md`, `docs/systems/mp-shot-gate.md`.

## Tests

- New `mp_death_drops_live` (6 tests): the offline drop (base id, clip, 1800, spin, falls near the
  eye, lands, spins, taken after the respawn); the offline expiry at exactly 1800 ticks; the online
  drop identical on two machines with the hashes in step; the online respawn wait = limit + 1; the
  online P99 take stays record 6; the finished-game kit holds 0xC.
- New unit tests (nf-game pickup): `a_dead_players_gun_drops_with_its_clip_for_30_seconds_and_spins`,
  `the_match_take_id_skips_upgrade_weapon_and_swaps_the_second_smg`.
- Fail-first: with the drop calls removed and the online `+ 1` removed, all four drop/respawn live
  tests fail; with the take id forced back to `Upgrade_Weapon(raw, 0, 1)` and the kit arm disabled,
  the P99 and kit tests fail. All pass on the branch.
- Also run: see the results list below.

Results (all green on the branch):
- All 40 `mp_*_live` binaries plus `mp_death_hands_online`. One red on the way: `mp_perf_16_live`'s
  pinned 16-player digest moved (48 → 36 kills: the drops and the match take id change the run;
  the two machines agree on every tick). Re-pinned to 0xa730_a7ac_a608_f75a with the reason.
- `local_match_bot_pickups_live`, `fix_start_loadouts_live`, `loadout_disc_rows_live`,
  `fix_pickups_ammo_live`, `fix_pickups_remainder_live`, `pickups_g029_live`,
  `drone_weapon_drop_live`, `walkreal_07000005_live`.
- nf-game lib `pickup::` / `player::` / `mp::` / `death::` (117).

## Not done / unproven

- Bots' own death drops are not ported. The game's bot death states (`BotDeathAnim` 0x175508 case 3,
  `BotDeathByExplosion` 0x1755a8) call `DroneWeap_DropWeapon`, which draws `Rand_FRand`: porting it
  moves the shared stream. A separate RNG-shifting change.
- The drop's start point uses the gun pose without the bob and the crosshair tilt (shared state
  only, so every machine agrees). The game uses the live view model matrix; the difference is a few
  centimetres before the fall. Not compared live.
- The game's push-out of a new pickup from overlapping geometry (`Collide_SphereIntersect` in
  `Pickup_Create`) is still not done for any drop.
- `MP_RegisterPickup`'s 64-slot cap and its eviction of the shortest-lived registered pickup are not
  modelled (the bots' rows just grow).
- The Professional Mode one-gun swap in the match take (`MPSettings+0x1BC`, 0x18e650:168-200) is
  not ported.
- The `Upgrades` row in a match (`Upgrade_MPWeapon` reads `Upgrades[slot·9]`) is taken as zero.
- Online player respawn: the per-slot dead counter respawns on its 300th count, which starts on the
  kill tick; whether that matches the game's `elapsed >= 300` from the `Player_CheckForDeath` stamp
  to the tick was not settled here (an existing test pins the current count).
- Needs the two-instance online check: a death drop seen on both machines (position, spin, the
  take by the other player), and a P99 / SMG pickup take online.
- Nothing here was compared against the real game live.
