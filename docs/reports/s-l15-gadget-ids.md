# s/l15-gadget-ids: the 07000015 keypad tests hold the game's decryptor id

No change to the shared random stream. Test and doc fixes only; no port code changed.

## The game's 07000015 start kit

- [M] `Player_InitWeapon` 0x1a7ef8 case 0x7000015 runs `Player_RamLoad`; with the handgun unowned
  (a cold load) it falls into the 0x7000014 kit: `Upgrade_Weapon(6)` 48, 0x11 12, 0x34 5, 0x37 5,
  `Upgrade_Weapon(0x4a)` 999, 0x50, 0x54, `Upgrade_Weapon(0x56)`, then the tail
  `Upgrade_Weapon(0x4e)` 999 and the fists; held 0x11 (:126-133, :289-307, :318-325). The SP
  `Player_EquipWeapon` 0x1a8a58 also runs `Upgrade_Weapon` after the model check (:24-30).
- [L] PCSX2, the retail image, fresh boot, `goto_level 07000015` (cold): `Upgrades` row 0x2beea8 =
  all zero; owned 0x1, 0x2, 0x11, 0x34, 0x37, 0x4e, 0x54, 0x56; held request 0x11. The stunner
  and grapple are equipped but revoked by `Player_CheckWeaponsLoaded` (no skins on 15).
- [L] Same, with `Upgrades[0] = 3`, `[5] = 1`, `[7] = 1` written before the load: owned 0x1, 0x8,
  0x11, 0x34, 0x37, 0x4f, 0x54, 0x57. This is exactly the port's set for the all-medals bonus
  word (`upgrades_from_bonus(u64::MAX)` = [3,1,1,2,1,1,1,1,1]).

## What was wrong

The tests, not the port. `interactables_07000015_live` loads with every gold medal
(`bonus = u64::MAX`, needed for the kind-6 pickups), so since the gold-medal upgrades landed
the kit carries the upgraded laser 0x4f and decryptor 0x57, as the game does. The two tests
cycled to a literal 0x4e and called it the Q-Decryptor. 0x4e/0x4f is the watch Laser (kind-1
lock burn only); 0x56/0x57 is the Q-Decryptor (docs/systems/decryptor.md, `nf_game::gadget`).
The tests then typed the keypad code by hand, so the held gadget never did the unlocking.

## What changed

- `interactables_07000015_live`: a `hold_decryptor` helper cycles G to
  `Upgrade_Weapon(0x56)` under the loaded profile's row; the typed-code shortcut is gone, so
  `the_keypad_burn_unlocks_its_door` and `the_pickups_take_on_the_walk` now prove the decryptor's
  own decode opens #683 (ch 0x95) and #685 (ch 0xf9).
- docs/atlas/weapon-data.md and docs/port-map/gadgets.md: the shifted gadget names corrected
  (0x4a/0x4c Stunner, 0x4e/0x4f Laser, 0x50/0x51 Grapple, 0x54/0x55 Micro-Camera, 0x56/0x57
  Q-Decryptor; which second ids are upgrades). `nf_game::gadget` header comment likewise.

## Other levels

Boot owned sets on every `dev::LEVELS` entry, bonus 0 and bonus `u64::MAX`, read from the real
load path. Fresh profile: unchanged by the upgrade commit except the default arm, whose raw
`EquipWeapon(6)` now owns 2 (the [M] remap; `loadout_disc_rows_live` checks every level
against the disc rows). All medals: every kit with an upgradeable id moves to the table entry
`Upgrade_Weapon` picks (handgun 8, stunner 0x4c, laser 0x4f, dart gun 0x44, sniper 0x22,
silenced sniper 0x28, decryptor 0x57), consistent with [M] and the 15 [L] above. No other
test was found asserting a literal un-upgraded id under a medal load; 06 and 08's medal tests
pass.

## Tests

interactables_07000015_live 19/19, walkreal_07000015_live 9/9, pickups_g029_upgrades_live 3/3,
fix_start_loadouts_live 8/8, loadout_disc_rows_live 2/2, gold_medal_realpath_live 2/2 (2
ignored), interactables_07000006_live 5/5, interactables_07000008_live 15/15.

Unproven: the live medal run wrote the `Upgrades` row directly rather than earning medals, so
`upgrades_from_bonus` itself (the row from a real bonus word) is [M] only here.
