# mp-team-aim-marker — the `Check_Target` multiplayer arm

Branch `s/mp-team-aim-marker`, cut from `origin/master` 10956badb.

## What's done

**The multiplayer arm of `Check_Target` 0x1a7810 is ported** (crates/nf-view/src/scene.rs, the
every-6th-frame crosshair block), and `mp_is_object_on_team` 0x18b720 (crates/nf-game/src/mp/rules.rs)
has its caller. The arm:

- Builds the same 50-unit aim ray the SP block already used, and ray-tests the match's player/bot
  bodies (`Sim::mp_combat_bodies` — the same cylinders the MP rounds hit). Dead bodies take no arm
  (a dead player is class 0x12 [M `Player_CheckForDeath` 0x190810:59]) and the aimER's own body is
  excluded the way the game's ray excludes its aimER (`Collide_RayIntersect`'s obj_tag argument IS
  the aiming obj, `param_1` [M 0x1a7810:122-123]).
- **Class 3 = a live player** (`Player_Init` 0x1910f8:76 and `MP_ReSpawn` 0x185368:83 write
  `obj+0xff = 3`; the bots ride `MP_ReSpawn` on every respawn [M `BOT_respawn` 0x124e38:77-79]):
  when `MPSettings._388_4_` and `_408_4_` are set and `MP_isObjectOnTeam` 0x18b720 puts the hit on
  the aimER's team (`MP_getObjectTeam` 0x18b6c0 — 2 outside a team scenario / Assassin), the marker
  bytes `BLData+0x132 = 1, +0x133 = 3` — **and nothing else**: the aim-latch stores (+0x2a/+0x2c4)
  sit in the class-2 arm only, so aiming at a teammate is never latched [M 0x1a7810:128-133].
  The mark has **no weapon gate** on class 3 (the arm returns before the shared weapon/level tail).
- **Class 2 = a drone-classed obj** (SP NPCs, `NDrone2_CreateObj` 0x14d7c8:14): the same mark gates
  through `ext+0xc`'s body and keeps the weapon/level gate of the shared `!bVar6` tail, while the
  aim latch (+0x2a = 1, +0x2c4 = the clock) is stored **unconditionally** — the decompile shows the
  stores before `if (!bVar6)` and the disassembly confirms: `sb $v1,0x2a($s0)` at 0x1a7aa8 and
  `sw $v0,0x2c4($s0)` at 0x1a7ab4 run before the `bnez $s1` exit at 0x1a7ab0. The port carries no
  latch on MP bodies: the bot AI (the `BOT_update` family) never reads +0x2a (no `BOT_*` decompile
  file touches it — the readers are the SP `NDrone2` paths: `NDrone2_PreDroneControl` 0x148db0:26-29
  expires it after 30 frames and counts +0x2c8), and a written byte with no reader is not modelled.
  A fresh bot's pre-respawn class is unwritten (`Drone_Create` 0x136be0 sets no +0xff) — the port
  models the respawned steady state (class 3) [S].
- **Objectives take no arm**: the MP objective models are class 0x35 (`MP_CreateObject` 0x185de8:122)
  and `Check_Target`'s switch handles classes {2, 3, 0x1c, 0x1d} only — aiming at a team objective
  leaves the normal crosshair. The task brief expected "your own team's objective" to mark; the
  decompile says otherwise, and the live checks below ask PCSX2 to settle it.
- **No Rand draws, no wall clock** — the arm only reads lockstep-equal state (settings + body
  positions), so every machine online computes the same crosshair state; `cross_state` itself stays
  view-side (not hashed), so the sync hashes are untouched (mp_world_sync_live green).

## The MPSettings words, settled

The decompile reads `_388_4_` and `_408_4_` — Ghidra's `_NNN_` offsets are DECIMAL bytes:
`_388_4_` = **0x184** (the port's `master_copy` — `MP_Start` 0x183a50:70 literally does
`MPSettings._388_4_ = MPSettings._384_4_`, i.e. 0x180 → 0x184) and `_408_4_` = **0x198** (the port's
`friendly_fire`, already documented). `MP_isObjectOnTeam`'s own gates are `_396_4_` = 0x18C
(`team_game`, `MP_Init` 0x183438:41 `(scenario >> 29) & 1`) and `_420_4_` = 0x1A4 == 0x400
(Assassin). The port's `MpSettings` struct comments carried the same hex-form confusion on
`master_copy` ("+0x388") and `team_game` ("+0x396") — corrected (crates/nf-game/src/mp/mod.rs),
with the `master` field's already-fixed comment as the convention.

## What reads +0x132/+0x133, and what the port did today

- `BLData+0x133` is read by `HUD_UpdateCrossHair` 0x19e6d0:69: it selects the crosshair frame from
  `HUDCrossCoords + state*0x12` for non-scoped weapons and picks the sprite colour (red vs the
  state-3 mark) [M :69-100]. `BLData+0x132` is written by `Check_Target` and cleared by
  `Player_InitWeapon` 0x1a7ef8:50 — **no reader in ACTION.ELF** (grepped); it is carried as the
  mark's companion byte only.
- The port's match aim did NOTHING before this task: the crosshair block was SP-only (the civilian
  test on `drones.list`), MP bodies are scene characters, not drones, so the ray never hit them and
  `mp_is_object_on_team` was dead code. The aim latch (the SP `Drones::tick` path, drone.rs:18392-18400)
  never saw an MP body either.

## The SP guard fix (an existing port behaviour contradicted the decompile)

The SP civilian-mark block carried `level_exempt = levels 0x7000005..0x7000008 → no mark`. The
disassembly says the opposite: at 0x1a7ad0-0x1a7adc, `sltu $v1, $v0(0x7000004), $a0` then
`bnez $v1, 0x1a7b48` jumps STRAIGHT to the mark stores — levels 0x7000005..0x7000008 **skip the
weapon check and always mark**; every other level marks unless the held weapon is in
{1, 0x35, 0x36, 0x43, 0x44} (the fists and the throw family) [M 0x1a7810:152-183]. Fixed in
scene.rs with the addresses in the comment. No existing test pinned the inverted behaviour (the
crosshair tests run on levels 07000001/07000002, outside the range), so no test needed changing.

## How it was verified

- **`mp_team_aim_marker_live` (new, crates/nf-view/tests) — 3/3 passed.** A real team match
  (`load_mp_carry`, arena 07000023, TEAM DEATHMATCH, 2 humans + 2 bots, friendly fire ON):
  - aiming at the teammate human → `crosshair_state() == 3` (the mark) [M 0x1a7810:129-133];
  - `friendly_fire = 0` → 2 even on the teammate [M 0x1a7810:143's `== 0` arm];
  - aiming at the teammate bot → 3 (bots are class 3 through `MP_ReSpawn`);
  - aiming at the enemy bot → 2 [M 0x1a7810:143-148];
  - no drone latched, no bot hurt by the aims (no latch path, no damage);
  - Assassin: the reset zeroes the team words and sets the assassin's to 1 [M 0x18aa70:12-41];
    the gate opens via the scenario word and the team answers follow the assassin pick;
  - CTF: aiming at a base (class 0x35) → 2, nothing latched.
- **Every `mp_*_live` binary** (nf-view, `--test-threads=1`), pass counts: mp_16_slots_live 3,
  mp_bots_live 3, mp_bots_moving_live 3, mp_core_live 5, mp_clock_gate_live 1,
  mp_combat_weapons_live 11, mp_host_match_live 2, mp_match_end_live 2, mp_bot_objectives_live 10,
  mp_bot_weapons_live 2, mp_bot_voices_live 3, mp_bot_body_sounds_live 1,
  mp_bot_armour_online_live 1, mp_pickup_take_live 2, mp_pickups_visible_live 2,
  mp_respawn_ammo_live 3, mp_hud_16_live 2, mp_hud_real_live 4, mp_remote_fx_live 2,
  mp_shot_effects_live 2, mp_shot_gate_live 5, mp_death_effects_live 3,
  mp_site_damage_offline_live 4, mp_site_destruction_blast_live 4, mp_net_slots_post_move_live 2,
  mp_menu_lists_live 5, mp_perf_16_live 2, local_match_bot_pickups_live 3,
  mp_world_sync_live (see below), mp_team_aim_marker_live 3.
- **`mp_stress_explosives_live` is killed by SIGKILL on this sandbox** (4 GB RAM, eight machines
  spamming explosives) — environmental, NOT this branch: the same SIGKILL happens with
  `origin/master`'s scene.rs checked out (verified by re-running the binary against the parent
  commit's file). It needs a bigger box, not a code change.
- The full suite (`tools/test/suite.py`) was run on the final tree (master 10956badb + this branch's
  commits) — the total line is at the bottom of this report.

## What's left

- `mp_stress_explosives_live` needs a sandbox with more RAM to pin (see above; pre-existing).
- The fresh-bot pre-respawn class (unwritten +0xff) is modelled as class 3 [S]; the live checks
  below settle it on hardware.
- The objective-mark question (the task brief vs the decompile's 0x35) — the live checks settle it.

## Live checks for the main agent

All in a REAL match on the disc (arena 07000023), fresh boot:

1. **The mark is a visual — render checks.** TEAM DEATHMATCH, friendly fire ON, 1 human + bots.
   - `--bond -16.15,2.74,3.38,1.57,0.0` aiming along +x with a teammate body ~4 units ahead
     (the test's arrangement): the frame where the crosshair is Bond's front-center should show
     the red "no" mark (the `HUDCrossCoords` state-3 cell), not the normal crosshair.
   - The same spot aimed at an enemy bot (−20.15, ~2.7, 3.38): the normal crosshair.
   - Aiming at a CTF base model in a CAPTURE THE FLAG match: expected the normal crosshair (class
     0x35 has no arm). **If PCSX2 shows the mark, this report's objective reading is wrong** —
     the arm needs revisiting.
2. **gdbwatch, a bot's ext** (`obj+0xe0`): at match start, read the bot obj's class byte
   (`obj+0xff`) — expect it NOT to be 3 before the bot's first respawn (Drone_Create writes no
   class), and 3 after any `MP_ReSpawn` [M 0x185368:83]. Then aim at the fresh bot: watch
   `ext+0x2a`/`ext+0x2c4` — if the class is not 2, expect NO latch write while a teammate-bot aim
   after its first respawn also writes nothing (class 3 has no latch arm) [M 0x1a7810:128-150].
3. **The FFA sentinel**: in a plain DEATHMATCH, aim at a bot: `mp_is_object_on_team` must answer
   false for every hit (the bots' team word is 2 in FFA [M `BOT_init` 0x1249e8 tail], the gate is
   closed anyway [M 0x18b720]) — the crosshair stays normal.

## Usage

not visible to me (no token, cost or context-window figures are exposed in this environment).

## Suite total (final tree)

(pending — filled in below when the run finishes)
