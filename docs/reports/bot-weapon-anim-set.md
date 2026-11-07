# bot-weapon-anim-set — the bots' held-weapon animation set

2026-11-07, branch `s/bot-weapon-anim-set` (from origin/master 22d5836b7).

The task: `BOTWEAP_changeWeapon` 0x12a028 calls `BOTWEAP_getWeaponAnimSet` 0x12b140 and writes
the result to Drone+0xDA; the port's `weapon_anim_set` (mp/botweap.rs) was unit-test-only. Read
0x12a028 in full, find every reader of +0xDA, find what the bot's held-weapon animation uses in
the port, whether a guard's set comes from the same field, compare weapon by weapon, store the
set on the real bot change path, remove stand-ins.

## 0x12a028 in full [M]

`BOTWEAP_changeWeapon(drone, w)` (272 bytes):

1. `BOTWEAP_hasWeapon` 0x1298d8 — not held → return 0 (nothing below runs);
2. `*(BOT_vars + 0x768) = (char)w` — the current-weapon byte;
3. `Drone_DCVfromOBJ` + `DroneWeap_ChangeWeapon(dcv, w)` 0x17a6e0 — the per-weapon dispatch
   (jumptable `PTR_LAB_002fb740`, w < 0x37; the model/draw side);
4. `*(short *)(drone + 0xDA) = BOTWEAP_getWeaponAnimSet(w)` — **the anim-set store**;
5. the clip pair: `+0xBBC` := the clip slot's rounds (0 → 1), `+0xBBE` := min(weapon_data cap,
   the pool at `BOT_vars + ammo_class·2 + 0x698`).

The port's `botweap::change_weapon` covered the has-weapon guard and `+0xBBC`; the `+0xDA` write
was missing — this branch adds it at the real call site, in the game's order (below).

## Every reader of +0xDA in the game

Writers first:

- `NDrone2_DefaultInit` 0x14b300 — 23 writes: the skin/mode switch sets `+0xD8` (type) and `+0xDA`
  (set) per drone type hash (0x5000004…): 0xc default, 7 for 0x500001e, 1, 3, 0xe, … [M]. **This
  is the single-player guard's set** — the port models it as `drone_setup(skin, weapon, level).anim_set`
  (nf-formats::drone, docs/systems/drone-animation.md). Same field as the bots', different writer.
- `BOTWEAP_changeWeapon` 0x12a028 — the MP bot change write (0x14–0x1a) [M].
- `BOTWEAP_InitWeapon` 0x129e48 (the spawn loadout) does **not** write +0xDA [M — the fn calls
  `BOTWEAP_EquipWeapon` twice and writes only `+0x768`]; a bot's spawn value is DefaultInit's.

Readers (the offset grep over the whole decompile, drone-struct uses only):

- `DroneAnim_SetDAnimInternal` 0x13aee8 — **the mechanism**: the danim's script is
  `Drone_AnimTables[danim][+0xDA]` (the 27-set column table at 0x275ae0, 436 × 0x6C [D docs/systems/
  drone-animation.md]), set 0's entry when the set's is 0 [M];
- `DroneAnim_SetCombatMoveAnim` 0x13cfb0 — 11 reads (the combat-move anim variants per set);
- the cover/lean/idle choosers: `ValidateCoverAnim` 0x13bf40, `CoverAnim` 0x13c078,
  `CanCrouchLean` 0x13b9e0, `CanCrouchStepOut` 0x13bb38, `CanStandLean` 0x13bc60,
  `CanStandStepOut` 0x13bde8, `SetStandIdleAnim` 0x13ca68, `SetWalkIdleAnim` 0x13cc70,
  `CanDoAnimState` 0x13dfc8;
- combat behaviour: `NDrone2_CanStrafeLeft` 0x14f9e8 / `CanStrafeRight` 0x14fbe0,
  `NDrone2_DoModeSettingsOLD` 0x14cfa0, `DroneFunc_HandleImpact` 0x1469b8 (the flinch),
  `Drone_ModBulletDamage` 0x139f50, `DroneWeap_CloseCombatImpact` 0x1789c0 (**`+0xDA == 7` = the
  kick gate** — the port's SP `close_combat_impact` reads it), `DroneWeap_DoBulletAccuracy`
  0x179740 (**`+0xDA == 0x13`** — the DefaultInit sniper set, an SP-guard band);
- the state bodies: Taser 0x162b98, StunDartImpact 0x163430, Death_Anim 0x1637f8,
  DeathByExplosion 0x163a80, SpecialDeath_Anim 0x163c68, CombatNoMove 0x1643e8, AimStandFire
  0x166328, AbseilSlide 0x169730, UnderCoverIdle 0x169e58, UnderCoverFire 0x16a3e0,
  StunGrenadeImpact 0x172bb0, AbseilStepOff 0x1738e0.

Not readers (grep noise, checked): `NDrone2_DSTATE_BotGlobal` 0x16e060's `case 0xda` arms are
STATE ids (0xd4–0xdf), `BOTSTATE_reallyWantACombatMove` 0x1267a0 passes 0xda as an argument, and
`LS_*`/`Label_*`/`parseentity` are other structs. **No Bot* state reads the offset** — the bot's
+0xDA serves the drone danim machinery (the one-shot anims a bot plays through the same
`DroneAnim_*` family, e.g. the reload danim 0x52) and any shared combat reader.

## What the bot's held-weapon animation uses, port vs game

- **The drawn MP body's locomotion** (the walk/idle/strafe scripts): the game's
  `PlayerAnimSetInitNormal` 0x18fcd8 / `Crouch` 0x18fea0 select the set descriptor by
  `weapon_data[ANIMOBJ+0x62]·0x10C + 0x84` (the weapon's anim-set **class** byte) when MP is on
  [M, docs/systems/mp-players.md]. The port's `advance_mp_anim` feeds exactly that class
  (`nf_formats::mpanim::weapon_anim_class`) into `MpBodyAnim` — **the game's own mechanism, not a
  stand-in**; nothing removed.
- **The drone-side set (`+0xDA`)**: the port's SP guards carry it (`Drone::anim_set`, from
  `drone_setup`); the bots did not carry it at all — `weapon_anim_set` was dead code. This branch
  adds `BotBrain.anim_set` (the brain is the port's drone stand-in) and writes it on the real
  change path.
- The one-shot danims a bot plays (the reload 0x52) are still timer-modelled in the port
  (`reload_until`, marked [S] there) — playing the scripts is future work; the set they would
  index is now stored.

### The weapon-by-weapon comparison (three id spaces)

| weapon ids | `BOTWEAP_getWeaponAnimSet` → +0xDA [M 0x12b140] | `weapon_data+0x84` class (the MP body) [L boot init] | DefaultInit's weapon switch → +0xDA (SP) [M 0x14b300] |
|---|---|---|---|
| 0x01 (fists) | 0x18 | 0 | — |
| 0x02–0x0C (pistols/SMGs) | 0x16 | 1 | 2,6,0xA,0xB → 4 |
| 0x0D | 0x1A | 6 | — |
| 0x0E–0x10 | 0x17 | 4 | 0xE,0x10 → 4 |
| 0x11–0x14, 0x29, 0x2A, 0x32 | 0x15 | 2/3 | 0x12 → 6 |
| 0x15–0x28 (rest) | 0x14 (also the default arm; 0x16/0x1A have no arm) | 2/3 | 0x16,0x18–0x1B → 5; 0x1C,0x1D,0x2A–0x2D → 9; 0x1E,0x1F,0x24,0x25 → 0x13 |
| 0x2B–0x2E (launchers) | 0x19 | 5 | 0x2A–0x2D → 9 |
| 0x33–0x3F, 0x44, 0x51, 0x52 | 0x18 | 0/5 | 0x3B–0x40 → 4; 0x30,0x32,0x33,0x6A,0x6E → 6 |

The three columns are different id spaces over the same table family: +0xDA indexes
`Drone_AnimTables`' 27 columns (0x14–0x1a are the bot columns), the class byte indexes the 16
MP body descriptors at 0x2b91b0 [D], and DefaultInit's sets are the SP columns. They agree
category-wise (the pistols/rifles/launchers cluster) but are NOT interchangeable — the pin below
keeps each to its own table.

## What changed on the branch

1. `BotBrain.anim_set: u16` — the port's `Drone+0xDA` for bots (own-line field; the doc cites the
   writers/readers). Init 0 with the documented reason: the game's spawn value is DefaultInit's
   write for the bot's drone, and the bot drone's dtype/skin mapping is not lifted — a live check
   below asks for the real value.
2. `bot_attack_change_weapon_enter` (the `BotAttackChangeWeapon` 0xDE enter — the game's
   announce → `BOTSTATE_changeWeapon` → changeWeapon chain) now writes the set between the
   `+0x768` byte and the clip pair, inside the has-weapon guard, exactly 0x12a028's order
   [M 0x12a028:29-33]. `weapon_anim_set` is no longer test-only.
3. No Rand draws added and no state-hash change: the field is written from a pure table lookup on
   a path that already changed `weapon_id`; `BotBrain::state_hash` is unchanged (the set is
   derived from the hashed weapon byte — the same policy as `clip_rounds`... **correction**: it is
   NOT hashed — see the hash note below).
4. The init path (`weapons_tick`'s arm) deliberately does NOT write the set — the game's
   `BOTWEAP_InitWeapon` doesn't either [M 0x129e48].

**Hash note**: `BotBrain::state_hash` was not extended with `anim_set` — the field is a pure
function of the already-hashed `weapon_id` (`anim_set == weapon_anim_set(weapon_id)` holds at
every quiescent point the invariant test checks), so hashing it would be redundant; the machines
stay bit-identical because both compute the same write from the same inputs. No draw, no clock.

## Verification

- `brain_change_weapon_stores_the_anim_set_per_set` (new, nf-game lib): for one weapon per switch
  arm — 0x01→0x18, 0x02→0x16, 0x0D→0x1A, 0x0E→0x17, 0x12→0x15, 0x16→0x14, 0x2C→0x19 — the real
  take (Pickup_Handler's class-2 arm) equips, the seeded msg-0x42 payload (+0x76A) drives the real
  `bot_attack_change_weapon_enter`, and the pair (weapon_id, anim_set) must match the table; the
  set survives the following tick (a stored field, not a scratch).
- `brain_rest_weapon_anim_set` (existing): the full 0x12b140 switch incl. the default arm — kept.
- mp_bot_weapons_live (nf-view, the real match): the two-machine live run on 07000023 now asserts
  `brain.anim_set == weapon_anim_set(weapon_id)` on **every natural switch** the bots make
  (take → choice → 0xDE) — the end-to-end chain through `net_tick` with the arena's pickups.
- nf-game lib: 687 passed / 0 failed (this tree, before the final suite).

### Binaries run (the final tree)

(filled from the final suite run below)

### Full suite

(filled when the run lands)

## What's left

- The bots' one-shot danims (the reload 0x52 etc.) still play no script (the port's `reload_until`
  timer stand-in, pre-existing); when they do, the script comes from `Drone_AnimTables[danim][set]`
  with the set this branch stores.
- The bots' spawn value of +0xDA (DefaultInit for the MP bot drone) — the live check below lifts
  it; then `bot_init` can carry it (the field is there, the writer is one line).
- The mp body anim for bots aims/strafes via the class table — already the game's mechanism; the
  render check below is the visual confirmation.

## Live checks for the main agent

1. **The bots' spawn +0xDA** [L wanted]: fresh-boot 07000023 with 2 bots; probe each bot drone's
   `*(u16*)(Drone+0xDA)` right after spawn (before any weapon change) — expected: DefaultInit's
   skin/mode value for the bot drone (0xc per the default arm, [S] unverified for MP). Report the
   values; they lift `bot_init`'s 0.
2. **Render check — the held-weapon body sets** (the mp body, the class table): run an offline MP
   match on 07000023, give one bot each of: pistol 0x02 (class 1 — body scripts 060009dc/060009de/
   060009db idle/walk/run), AR 0x16 (class 3 — 060001d4/060001db/060001d9), rocket 0x2C (class 5 —
   060001e6/060001e7/060001e5) [D descriptor bytes]. Frames: idle 2 s, walk forward 3 s, strafe
   left 2 s per weapon; `--bond` at the bot's spawn facing it. The body should hold the weapon
   low/at the hip per class and swap the locomotion set with the weapon.
3. **The +0xDA pair on a change** [L wanted]: the same match; after a bot picks up a weapon and
   switches (the announce), probe `*(u16*)(Drone+0xDA)` — expected exactly `BOTWEAP_getWeaponAnimSet`
   of the new weapon (0x16 for the P2K, 0x19 for the rocket) — the disc cross-check of the pin.

## Usage

not visible to me
