# WORKLOG

One entry per task, append only (never rewrite or reorder others' entries).

## bot-weapon-anim-set

2026-11-07 08:22 UTC, branch `s/bot-weapon-anim-set` (from origin/master 22d5836b7).

Task: MP bots hold each weapon with the game's animation set. `BOTWEAP_changeWeapon` 0x12a028
calls `BOTWEAP_getWeaponAnimSet` 0x12b140 and writes Drone+0xDA; the port's `weapon_anim_set`
(mp/botweap.rs) is unit-test-only. Read 0x12a028 in full, find every reader of +0xDA in the game,
find what a bot's held-weapon animation uses in the port and whether a guard's (SP) set comes
from the same field; compare weapon by weapon; store the set on the real bot change path, remove
stand-ins; no Rand-draw or state-hash changes. Pin through a real match (one weapon per set).
Run every mp_*_live, mp_world_sync_live and nf-game.

- [x] setup: repo cloned, branch created from origin/master 22d5836b7, pushed.
