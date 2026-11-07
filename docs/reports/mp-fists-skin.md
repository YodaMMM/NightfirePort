# mp-fists-skin — the multiplayer fists use the game's multiplayer model

Task: `Player_Init` 0x1910f8:108-113 takes a separate multiplayer arm — `weapon_data._488_4_ =
0x50000b0`, `weapon_data[0x3999] = 2`, and the skin from `MP_skins` by the player's MPSettings
row — where the single-player arm uses the `PunchSkin` table and `weapon_data[0x3999] = 1`. The
port had the single-player arm only (`crates/nf-formats/src/weapon.rs`
`apply_player_init_override` + the two nf-view build sites) and no reference to 0x50000b0.

## What the game does (the evidence)

`Player_Init` 0x1910f8 (ACTION.ELF) branches on `MPSettings._384_4_` (the MP master enable,
Ghidra DECIMAL 384 = +0x180) [M 0x1910f8:83,108]:

- **SP arm** (:83-107, disasm 0x191358-0x191394): `weapon_data._488_4_ = PunchSkin_204[idx]`
  (`sw $a1, 0x1e8($v1)` @0x191394) and `weapon_data[0x3999] = 1` (`sb $v0, 0x3999($v1)`
  @0x19138c, $v0 = 1).
- **MP arm** (:108-113, disasm 0x191398-0x1913e0): `weapon_data._488_4_ = 0x050000b0`
  (`lui $v1, 0x0500 / ori $v1, 0xb0 / sw $v1, 0x1e8($a0)` @0x1913b4-bc) and
  `weapon_data[0x3999] = 2` (`addiu $a3, $zero, 2` @0x1913a8, `sb $a3, 0x3999($a0)` @0x1913c4);
  per player, `AnimObjectNew(obj, MP_skins[row].skin)` (`lw $s0, ($v0)` @0x1913d4) and
  `BLData+0x965 = MP_skins[row].sleeve` (`sb $a2, 0x965($s1)` @0x1913e0), row =
  `MPSettings + slot·0x30 + 0x24`.

Field identities (the task's "find what reads … so each reader gets the right value"):

- `weapon_data._488_4_` is **decimal** 488 = 0x1E8 = record 1 + 0xDC (the fists' first-person
  skin; `weapon_data` itself is at 0x2bf150 [M the `lui $v1, 0x2c / addiu $v1, -0x0eb0` pair
  @0x191378-0x191380; record stride 0x10c [M `Player_AmmoIndex` 0x1a9148]). Its readers:
  `Player_CheckWeaponsLoaded` 0x1a97c8 (the loadout's hashtable gate — an fp skin not loaded
  refuses the weapon), `Player_EquipWeapon` 0x1a8a58, `Player_WeaponFiring` 0x1a5460 (the
  first-person arms' anim object), and the port's `botweap::BotWeaponTable` fp/`loaded` lift +
  the view-model build.
- `weapon_data[0x3999]` is a **single byte** (both arms store with `sb`) at record 0x37 + 5
  (0x37·0x10c = 0x3994) — the twin/variant link (docs/fields/weapon_data.md:28). ~20 readers
  test it as a flag and resolve the twin through the record's +0x02 short (`Player_AmmoIndex`
  0x1a9148, `Player_RoundToFire` 0x1a18e0, `Player_WeaponFiring` 0x1a5460, `Player_EquipWeapon`
  0x1a8a58, `Player_EquipAmmo` 0x1a8d18, `Player_AmmoInGun` 0x1a9190, `Player_ReloadAmmoType`
  0x1a1988, `Player_SetFiringAnim` 0x1a3c70, `BOTWEAP_ReloadAmmoType` 0x129d28,
  `BOTWEAP_changeWeapon` 0x12a028, `BOTWEAP_EquipWeapon` 0x12a138, `BOTWEAP_EquipAmmo`
  0x12a308, `BOTWEAP_decrRounds` 0x12a470, `BOTWEAP_AmmoInGun` 0x12ae40,
  `BOTWEAP_getWeaponAmmoAmount` 0x12afa0); three read the **value** as an index offset:
  `Player_HandleHasNoAmmo` 0x1a2ba8:40 (`next = w + (s8)link` when +0x68 & 0x10 — rec 0x37's
  flags are 0x14090 [L ee_start_05.bin @ 0x2c2ae0]), `Player_LaserPointer` 0x1a65d8:100 and
  `Player_WeaponFiring` 0x1a5460:112 (both `w + (s8)link`) — so 1 (SP) vs 2 (MP) selects
  different linked records (SP: 0x37→0x38, whose fp is 0x0500004b [L ee_start_05.bin @
  0x2c2bf8]; MP: 0x37→0x39, fp 0 [L ee_start_05.bin @ 0x2c2d04]). The port's readers:
  `gun::fire_mode_switch` (`new = w + twin`, the scene's mode press, scene.rs ~:3011) and
  `botweap::BotWeaponTable` (`twin: r[5] != 0`).

0x050000b0 is on none of the 29 `MP_skins` records [D ELF 0x26d488], so `MP_NeedSkin` 0x120c48
loads it unfiltered [M]; the MP arenas' own archives carry it — **[D] checked: all 8 arenas**
(0x07000021-29/4b/4c set: 23, 24, 25, 26, 27, 28, 29, 4b) list it in their kind-3 skins
(`every_mp_arena_carries_the_mp_fists_skin`).

The MP arm's per-player half (the MP_skins body skin + the sleeve) was already ported
(`nf_game::mp::slot_skin`, the scene's `MpBody.sleeve`) — this task adds the missing
**weapon_data** half.

## What changed

1. `crates/nf-formats/src/weapon.rs` — new `apply_player_init_mp_override()`: rec 1+0xDC =
   0x050000b0, rec 0x37+5 = 2 (the disasm stores, addresses inline). `apply_player_init_override`
   (SP) unchanged in behavior; its comment now cites its own disasm stores.
2. `crates/nf-view/src/lib.rs` — both weapon-data build sites (the `--give`/dev merge path and
   the plain rebuild) pick ONE arm like the game: MP loads (`mp` flag or settings present) run
   the MP arm, SP loads keep `apply_player_init_override(level)`.
3. `crates/nf-view/tests/mp_fists_skin_live.rs` — the pin (see below).

No Rand draw was added anywhere (the two values are load-time constants); the shared RNG and
the fixed 60 Hz are untouched.

## How it was verified

- **Red before green**: with master's `lib.rs` restored (`git checkout HEAD~1 --`), the MP test
  fails `assert_eq!(fp, MP_FISTS_SKIN)`: `left: 83886251` (0x050000ab — the SP default row the
  old code applied on the arena) `right: 83886256` (0x050000b0). With the fix, green.
- **`mp_fists_skin_live`** (3 tests, all pass):
  - `the_mp_match_carries_the_mp_fists_skin_the_2_link_and_own_fists` — the real
    `load_mp_carry` (the confirm page's path) on 07000023: rec 1+0xDC = 0x050000b0, rec
    0x37+5 = 2, the skin resolves in the arena's own table, the loadout gate
    (`mp_weapon_loaded[1]`) sees it, and the real `MP_EquipPlayer` equip owns the fists — on
    the old code the SP row 0x050000ab is on no arena, so the gate refused the fists and an MP
    player had no fists to cycle to. A second load with seed 0x1234_5678 has a bit-identical
    `weapons.records` + `mp_weapon_loaded` (lockstep: every machine builds the same table; no
    Rand draw on the path).
  - `single_player_loads_keep_their_punchskin_rows_and_the_1_link` — the real `nf_view::load`
    of 07000005 keeps PunchSkin[0] 0x050000ab and the 1 link (matching the disc's own post-boot
    values [L ee_start_05.bin @ 0x2bf338 = 0x050000ab, @ 0x2c2ae9 = 1]); the other three
    GameState classes pinned at the table level (0x7000001-4 → 0x05000099, 0x7000007/8 →
    0x0500003d, 0x700001b → 0x050000a1 [L ee_start_05.bin @ 0x2b9430]).
  - `every_mp_arena_carries_the_mp_fists_skin` — the [D] scan above.
- **Per-binary runs** (the suite's own log, each binary whole, `--test-threads=1`): every
  `mp_*_live` binary (incl. `mp_world_sync_live`), `nf-game`, and the two binaries that call
  `apply_player_init_override` (`weapon_assets_live`, `weapon_model_census_live`). The counts:
  see "The full suite" below.
- **The full suite** (`tools/test/suite.py`, the final tree): see below.

## The full suite

(placeholder — filled at the end of the session)

## What's left

Nothing on the task statement. [S] note: the MP twin target rec 0x39 has fp 0 — the game's own
data; the port mirrors the value as the game writes it.

## Live checks for the main agent

Addresses: `weapon_data` base 0x2bf150; `weapon_data[1]+0xDC` = 0x2bf338;
`weapon_data+0x3999` = 0x2c2ae9.

1. **gdbwatch write 0x2bf338 (word)** during a Quick-Game deathmatch load on 07000023: expect
   the store `0x050000b0` from `Player_Init` 0x1913bc (`sw $v1, 0x1e8($a0)`) once per spawned
   player; on an SP level load (e.g. 07000005) expect `0x050000ab` from 0x191394 instead.
2. **gdbwatch write 0x2c2ae9 (byte)**: expect `2` from 0x1913c4 on the MP load, `1` from
   0x19138c on the SP load.
3. **Render (the model change is a visual)**: in the MP match, hold the fists (weapon 1 — the
   weapon cycle with nothing owned, or the dev give): the first-person arms ~0.5 s after the
   draw starts show the **MP fists model** (0x050000b0's anim object), not the SP PunchSkin
   arms (0x050000ab). The port should match the PCSX2 frame. The spawn itself holds 0x47
   (hands) for one tick before the spawn-weapon draw [M 0x1855f8:50-53], so the fists must be
   selected to see the model.

## Usage

not visible to me — this session exposes no token, credit, context-window or cost figures to copy.
