# Pickups and inventory against the game

Scope: `Pickup_Create` 0x18dda0, `Pickup_CreateSimple` 0x18f348, `Pickup_CreateFromSet` 0x18f1b0,
`Pickup_Update` 0x18e148, `Pickup_Handler` 0x18e650, `Player_EquipWeapon` 0x1a8a58,
`Player_EquipAmmo` 0x1a8d18, `Player_InitAmmoWeapons` 0x1a17f8, `Upgrade_Weapon` 0x1a8f30,
`DroneWeap_DropWeapon` 0x17a2a0, `Player_RamSave` 0x195670 / `Player_RamLoad` 0x1957a8, and the
placement call in `parsemap_create_dynamic_objects` 0x1d04f0 (disasm 0x1d05a4-0x1d05ec), all read
against the port. Census over every level in `nf_view::dev::LEVELS`: the 0xF0 pickup placements and
the guards' held weapons (placement param 9).

Random stream: unchanged. Neither fix adds or removes a draw.

## Fixed, biggest player impact first

1. Weapon-10 drops give weapon 10. `Upgrade_Weapon` maps 10 to 0xC only when the byte
   `GameState[0x52]` is set [M 0x1a8f30:18-22]. That byte is the game-complete flag: its writers
   are `Menu_SetLevelBonus` 0x1ffa10 (finishing 0x0700001B), `Boot_LoadPTPData` 0x1c8560 and
   `Menu_MapDefaultCodename` 0x1f7fa8 (the profile) [M every writer]. The port passed the tick
   count, which is never 0 in play, so every weapon-10 pickup became record 0xC. Record 0xC is the
   stronger one: damage 4.0 vs 2.0, full auto (+0x28 999 vs 1), autoaim 100 vs 80, its own name
   0x05000073 [L weapon_data]. Guards holding weapon 10: 07000007 x8, 07000008 x15, 0700000c x2,
   0700004a x23 [D]. The SP take now reads `Progress::game_complete`, and so does the dev panel's give.
2. Taking the Ronin case back puts the case in hand. `Player_EquipWeapon` with 0x52 clears
   `BLData+0x59E`, which is record 0x53's owned byte, and requests 0x52 when the held weapon is
   0x53 [M 0x1a8a58:81-85]. This runs whatever the auto-switch option says. The port left the
   remote 0x53 in hand and in the weapon cycle after the fold, with no turret left to drive
   (0700000c #561, 07000012 #753).

## Checked, matches the game

- Drops: base id, the clip times (0.5 or 0.25) + FRand(0.5) rounds, the 0x1A grenade pile, the
  0x6A/0x6E to 0x33 remap and the refused ids, a lifetime of FRAME_RATE_INT x 120 [M disasm
  0x18f43c-0x18f488], and the ray fallback to the drone's position.
- Placement arguments: type +0x2C, id +0x30, rounds +0x34, channel +0x38, sound +0x3C, rest 1,
  respawn +0x40, lifetime 0, label +0x44 [M disasm 0x1d05ac-0x1d05ec].
- Respawn: 0 deletes, 0xFFFF stays, otherwise hidden for FRAME_RATE_INT x respawn x 10 ticks.
  The take, the lifetime skip on a touch tick and the hidden-pile take also match.
- The equip and ammo arithmetic, ammo_data caps, the 0x1F/0x25 ammo remap, armour (refused at
  50 or more, else set to 50), keys, labels, and the kind-5 and kind-6 arms.
- Carry between parts: carried rounds, clips, owned flags and keys ride `RamSave`, as the game's
  copies do. The fire-mode byte (+0x1BB) is not carried. It can never leave 0 for a player
  weapon, since +0x2E is 1 on every record below 0x60 [L].

## Not changed (unproven or off scope)

- The MP take paths (`update_pickups_net`, `mp::combat::gate_take`) still pass the tick count.
  The game's MP arm calls `Upgrade_MPWeapon` 0x1a9040, which has no case 10, so 10 should not
  remap there either. That is MP rules, not on-foot levels, so I left it alone.
- `Player_EquipWeapon` clears record 0x11's owned byte (+0x286) when the equipped base is 0x33,
  SP only [M 0x1a8a58:30-31]. It is not ported, because `Inventory` has no MP flag. No campaign
  kit owns 0x11 on the 0x33 levels.
- `create_placed` turns a kind-0 placement with 0 rounds into a full clip. The game passes 0
  [M disasm]. The only such placements sit in non-main maps at the origin (03 m4, the MP
  arenas) [D]. The script arm was not censused.

## Tests

- New `pickup_upgrade_gate_live`: a weapon-10 drop at Bond's feet on 07000008 gives record 10 on
  a fresh game, with a sister case that gives 0xC once the game is complete. It fails on the old code.
- `ronin_live::ronin_remote_view_and_pick_up` now also checks that re-taking the case clears the
  remote and switches the hand to 0x52. It fails on the old code.
- `walkreal_07000008_live::..._00agent` failed with fix 1. The walk had reached the Hind fight
  carrying record 0xC from the weapon-10 drops. With weapon 10 it arrived with 54 health, and its
  own rocket blast (-54) killed it. On the old code it scraped through at 1 health. The 00 Agent
  route (D3-gated, so the sister walks are unchanged) now takes the ledge armour #282 (kind 3,
  (109.5, 27.8, -129.4) [D]) before it boards, and passes with 29 health. The Hind launch gate
  that lets those self-blasts through is still the walk's weak point.
- Passing: walkreal 07000008 (6), walkreal 07000012 (3), ronin_live (9), pickup_upgrade_gate_live (1), fix_dev_panel_live (6), pickups08_live (5),
  fix_pickups_ammo_live (4), pickups_g029_live (4), drone_weapon_drop_live (3), nf-game pickup
  unit tests (13), walkreal 07 (6) and 0c (3, 1 ignored).
