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
- [08:55 UTC] The research: `BOTWEAP_changeWeapon` 0x12a028 read in full — the +0x768 byte, the
  DroneWeap_ChangeWeapon 0x17a6e0 dispatch, then `+0xDA = BOTWEAP_getWeaponAnimSet` 0x12b140, then
  the clip pair. Every +0xDA reader in the game is the SP drone anim/combat family
  (SetDAnimInternal 0x13aee8's `Drone_AnimTables[danim][set]` column, SetCombatMoveAnim 0x13cfb0
  ×11, the cover/lean/idle choosers, CloseCombatImpact 0x1789c0's ==7 kick, DoBulletAccuracy
  0x179740's ==0x13 sniper band, HandleImpact 0x1469b8, ModBulletDamage 0x139f50 …) — BotGlobal's
  0xda hits are STATE ids and reallyWantACombatMove's is an argument, not the field. The bot's
  DRAWN body anim keys off `weapon_data[w]+0x84` instead [M PlayerAnimSetInitNormal 0x18fcd8] —
  the port's mp-body-anim already does exactly that, so no stand-in there; the missing piece was
  the field itself: `BotBrain.anim_set` added, written on the real change enter in the game's
  order (+0x768 → +0xDA → +0xBBC) [M 0x12a028:29-33]; the init value (DefaultInit 0x14b300's
  skin/mode write) not lifted for MP bots — 0 until the first change, documented + a live check.
  The per-set pin (7 sets through the real enter + take) and the live-match invariant (the pair
  asserted on every natural switch) both green; nf-game lib 687 passed.
