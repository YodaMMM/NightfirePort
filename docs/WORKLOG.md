
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
