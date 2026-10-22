# The drone's weapon-slot handling (`DroneWeap_ChangeWeapon`, `DroneWeap_DropWeapon`)

Proof levels: [M] decompile (file = `work/decomp/ACTION.ELF/<addr>_<name>.c`, `:NN` = line), [M-dis]
checked against the raw MIPS via `tools/crosscheck/mipsdis.py`, [L] live EE snapshot read
(`work/ee_start_05.bin`, file offset = EE address).

## The held byte and the slot

The drone's held weapon is `sAnimObject+0x62` (the obj at `Drone+0xC..0xDC` chain; the port's
`Drone.weapon`) — an id into `weapon_data` (0x10C-byte records, table at EE 0x2bf150). It is the
byte every consumer reads: the drawn model, `DropWeapon`'s pickup, the aim swap, `ObjectIsArmed`.

`DroneWeap_ChangeWeapon(DCVars*, w)` 0x17a6e0 takes **w = the new held weapon id** (the bots write
the same value to `Drone+0x768` and re-derive the anim set from it [M BOTWEAP_changeWeapon
0x12a028:43-48]), and is a **55-entry jump table** at 0x2fb740 (`sltiu a1,55` gate [M-dis
0x17a6e4]). Only six ids are handled; every other id reaches the shared `jr ra` at 0x17a78c and
changes **nothing** [M-dis 0x17a70c-0x17a790]:

| w | anim set `Drone+0xDA` | held `anim+0x62` | `+0xBBC` clip | `+0xBBE` clip_max | arm |
|---|---|---|---|---|---|
| 1 | 0xC | 1 | 0 | 0 | 0x17a72c → the 0x17a714 tail |
| 6 | 4 | 6 | 16 | untouched | 0x17a738-0x17a754 |
| 0x12 | 6 | 0x12 | 32 | untouched | 0x17a75c → the 0x17a748 tail |
| 0x18 | 5 | 0x18 (the raw param) | 32 | 32 | 0x17a770-0x17a788 |
| 0x19 | 5 | 0x19 | 32 | 32 | 0x17a770-0x17a788 |
| 0x36 | 0xC | 0x36 | 0 | 0 | 0x17a70c → the 0x17a714 tail |

`+0xBBC`/`+0xBBE` are the port's `clip`/`clip_max` (the reload refills read the pair [M
drone-cover.md; `DefaultInit` 0x14b300:289-344 seeds them per weapon class]).

## The writers of the held byte

- `NDrone2_DefaultInit` 0x14b300:668: placement parameter 9 (the switch's `default` arm replaces
  an unknown id with 1 and keeps the skin's anim set [M 0x14b300:266-344]).
- `DroneWeap_DropWeapon` 0x17a2a0 → 0 (always, even when the +0x80 gate refuses the drop).
- `DroneWeap_ChangeWeapon` — the table above.
- `Attack`/`AimStand`/`AimStandFire` swap 0x19 → 0x18 (the aim-id variant) [M 0x164210:57,
  0x166178:44, 0x166328:48].
- `Script_EventHandler` event 0xb writes a script-supplied id [M 0x1d94c0:155].

## The drop arm

`DroneWeap_DropWeapon` 0x17a2a0 [M]: gate `weapon_data[w+0x80] != 0` (the model; the port's
`model(w)`), then `Pickup_CreateSimple(mat, weapon_data[w+2], clip_size(+0x92)·(0.5 if <10 else
0.25 + FRand(0.5)), 0)`; the 0x1A base adds the hidden 0x1B ammo pickup. The matrix is the held
gun's bone world (datum `AnimDatumGetWeaponInfo`), the drone's own position when the ray from it
to the drone hits geometry. Callers: `SetAsDead` 0x14a2f0 (unconditional), `Death_Anim` case 3 /
`SpecialDeath` / `Death_By_Explosion` behind `firing(+0x3B) == 0 || +0xDA == 4 || +0xDA == 9`
[M 0x1637f8:72-75], `KnockedOut_Anim` [M 0x172ac0].

## The stun families' drop arms — the gate is the ANIM SET byte

- Taser `DSTATE_Taser` case 1 [M 0x162b98:27-34]: `IsCivilian` (= `Drone+0x44 != 1` [M 0x14aae0])
  skips; then `+0xDA == 4` skips; else `DropWeapon + ChangeWeapon(1)`.
- Stun grenade `DSTATE_StunGrenadeImpact` case 1 [M 0x172bb0:15-18]: `+0xDA != 4` →
  `DropWeapon + ChangeWeapon(1)` (no civilian gate).
- Tranq dart `DSTATE_StunDartImpact` case 1 [M 0x163430:20-27, dis 0x16347c-0x1634b4: `lhu 218`]:
  `+0xDA ∈ {5,6,9}` → `DropWeapon + ChangeWeapon(1)`; `+0xDA < 5` and the rest keep.
- Surrender `DSTATE_Surrender_Anim` case 1 [M 0x1626a0:21-22]: unconditional
  `DropWeapon + ChangeWeapon(1)`.

`+0xDA` is DefaultInit's anim set, keyed on the init weapon [M 0x14b300:267-344]: pistol class
{2,6,10,0xb,0xe,0x10,0x3b..0x40} → 4 (kept by every arm), {0x16,0x18..0x1b} → 5, {0x12,0x14,0x15,
0x30,0x32,0x33,0x6a,0x6e} → 6, {0x1c,0x1d,0x2a..0x2d} → 9, snipers {0x1e,0x1f,0x24,0x25} → 0x13.
[L ee_start_05.bin]: the Paris guards read wep 0x16 → +0xDA = 5 and wep 0x24 → +0xDA = 0x13.

## The wake re-arm — taser family only

`DSTATE_Stunned_Recover` case 1 [M 0x1630b8, dis 0x16314c-0x163150 loads `a1 = 6`]: for the
non-civilian (`+0x44 == 1`), `ChangeWeapon(6)` — the guard wakes with the fresh sidearm (weapon
6, anim set 4, clip 16). The game's skin-0x10 arm first flips `+0x44` to 1 [M 0x1630b8:17-19]
(not ported — not weapon handling). `DSTATE_StunGrenadeRecover` case 1 [M 0x163350] has **no**
DroneWeap call — the flashbang victim stays on the fists. `DrawWeapon`'s EXIT re-arms with
`ChangeWeapon(6)` when the held byte is still < 2 [M 0x166000:41-43].

Consequence: killing a tased-then-woken guard drops the weapon-6 pickup (base 6, model
0x0200025a, clip 16 [L ee_start_05.bin @ 0x2bf150+6·0x10C] → 16·(0.25+FRand(0.5)) ∈ [4..=12]
rounds). The old port modelled `ChangeWeapon` as a held-byte stand-in (`weapon = 1`), skipped the
wake re-arm entirely, and keyed the three stun gates on the held byte instead of `+0xDA` — a
woken guard's death dropped nothing, and a flashbang stripped anim-set-4 pistol guards the game
keeps armed.

## Port shape

`Drone::change_weapon(w)` (nf-game `drone.rs`) is the table; the taser/stun-grenade/dart/surrender
enter arms and the `DroneAnim_EventFunc` datum-2 drain (`drone+0x45` = placement param 10 [M
0x14b300:266]) call it. Tests: `crates/nf-view/tests/drone_weapon_drop_live.rs` (real input path
on 07000005/07000006) and the census in this doc's [L] tags.
