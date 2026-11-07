
## mp-fists-skin (2026-11-07, UTC)

Branch `s/mp-fists-skin` cut from `origin/master` (22d5836b7). Task: port the multiplayer
arm of `Player_Init` 0x1910f8:108-113 — `weapon_data._488_4_ = 0x50000b0`,
`weapon_data[0x3999] = 2`, and the skin from `MP_skins` by the player's MPSettings row
(the single-player arm uses the `PunchSkin` table with `weapon_data[0x3999] = 1`). The port
has the single-player arm only and no reference to 0x50000b0. Find every reader of
`weapon_data[0x3999]` and the +488 skin, confirm 0x50000b0 exists in the multiplayer
archives [D], pin a real match with fists to that model while single-player keeps its rows,
keep the shared Rand stream untouched, and run every mp_*_live binary,
mp_world_sync_live and nf-game.

- (02:30, 11-07) The MP arm ported: `WeaponData::apply_player_init_mp_override` (rec 1+0xDC =
  0x050000b0, rec 0x37+5 = 2 [M 0x1910f8 disasm 0x1913bc/0x1913c4]) and the one-arm pick at both
  nf-view weapon-build sites (MP loads take the MP arm, SP loads keep the PunchSkin rows). The
  per-player MP_skins skin/sleeve path already existed (slot_skin / MpBody.sleeve). Readers of the
  two fields enumerated (the twin link's ~20 flag readers + the three value-as-offset readers, the
  +0xDC loadout/equip gates). Pin test mp_fists_skin_live.rs written (MP real load, SP real load,
  8-arena [D] scan, seed-independence). Building once the baseline suite frees the cargo lock.
