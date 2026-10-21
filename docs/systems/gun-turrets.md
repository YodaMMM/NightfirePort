# Gun turrets: the kind-0x27 placement object (GT_*, class 0x36 '6')

Evidence tags: [M] decompile/disassembly (ACTION.ELF file + line), [D] disc bytes, [L] live EE
measurement, [S] suggested. The creator (`GT_Create` 0x17ffa0), the minigun deploy
(`GT_DeployMiniGun` 0x181560) and the script callback (`GT_Register` 0x181538) are already
ported/cited in `crates/nf-game/src/gt.rs` (their placement census: 41 disc-wide —
0700000a ×6, 0700000b ×5, 2 per MP map [D docs/systems/script-census.md:141]).

This page pins the behaviour functions (all reached through `control_funcs[54]`
0x26f890+54·0xc = {`GT_Update` 0x180498, —, `GT_Delete` 0x181678} [M action-symtab + the
`control_movement_object_handler` 0x1336f8:62 dispatch on obj+0xFF]) and the Sub_* group's
unreached status.

## Object state (obj_tag + GUNTURRET ext)

| field | meaning | proof |
|---|---|---|
| obj+0xF4 (s16) | state, 0..7 below | [M 0x180498 switch] |
| obj+0xF6 (s16) | previous state (`GT_Disable` stores 4; state 5 returns to it) | [M 0x1816a8] |
| obj+0xFE bit0 | self-delete request | [M 0x17ffa0:118, 0x180498 case 3/6] |
| obj+0xFF | class 0x36 | [M 0x17ffa0:74] |
| ext+0xB0/B4 | script A/B (`Script_Load` of placement params 2/3) | [M 0x17ffa0:100-106] |
| ext+0xB8 | owner object (placement parent 6, or the turret itself) | [M 0x17ffa0:90-92] |
| ext+0xBC | current target object | [M 0x180498 case 4] |
| ext+0xC0/C4/C8 | the ≤3 `GT_Register`-registered objects (0xC4 = the spinning barrel) | [M 0x181538, 0x180498 case 4 fire] |
| ext+0xCC (f32) | health, 100.0 at create | [M 0x17ffa0:84, 0x1816d8] |
| ext+0xD0 (f32) | muzzle offset along the barrel (script bbox max.z × 1.5) | [M 0x17ffa0:155-158] |
| ext+0xD4/D8 (f32) | barrel spin angle / spin velocity | [M 0x180498 case 4/7] |
| ext+0xDC (f32) | state timer (1.2 s restore, 0.5 s holster, 3.0 s sweep, 10 s wreck) | [M 0x180498] |
| ext+0xE0 (f32) | fire-interval timer (0.1 s per shot) | [M 0x180498 case 4] |
| ext+0xE8/EA (u16) | enable/disable switch channels (placement params 4/5) | [M 0x17ffa0:87-89, 0x180498 case 0/4] |
| ext+0xEC (s16) | facing mode (0 floor turret, 1 wall/ceiling weapon) | [M 0x17ffa0:82-83] |
| ext+0xF0 (s16) | shot counter: 200 (facing 0) or 0xffff; death at 0 | [M 0x17ffa0:92-96, 0x180498 death gate] |
| ext+0xF2 (u8) | mounted flag (`GT_TakeControl`/`GT_LoseControl`) | [M 0x181740/0x181408] |

## States (`GT_Update` 0x180498)

- **0 dormant** — facing 1 only: enable channel (ext+0xE8) must be 0 or ON, disable channel
  (ext+0xEA) 0 or OFF, then `GT_Track`; no target → stay. Both paths reset both scripts'
  done flag (obj+0x9C), timer, → 1. Facing 0 turrets skip straight to the reset [M case 0].
- **1 arming** — waits for both scripts' done flags, then → 4 [M case 1].
- **2 restoring** — slerps back to the script-A pose; after 1.2 s resets the scripts → 3 [M case 2].
- **3 finished** — when both scripts are done: mounted → `GT_LoseControl`; facing 1 →
  `Pickup_Create(0x2000366)` + self-delete; else → 0 [M case 3].
- **4 combat** — `GT_Track` every tick; a target plays `Music_Event(2,1)`, aims (slerp) and
  fires every 0.1 s while ext+0xF0 > 0 (`Bullet_init` weapon record 0x2c55d0, muzzle =
  barrel pos + dir·ext+0xD0, `Sound_Play3D(…, 0x4e7 facing-1 / 0x10a facing-0)`, barrel spin,
  muzzle smoke glist 0x2000039 when the script-B object's 0xF6 toggle is set). No target:
  slerps back toward the script-A pose; facing 1 with its channel gates satisfied re-holsters
  after 0.5 s → 2 [M case 4].
- **5 disabled sweep** — entered only by `GT_Disable` (below): `GT_RandMove` each tick (2
  shared-RNG draws), 3.0 s → obj+0xF6 [M case 5].
- **6 destroyed** — smoke every 10th frame while ext+0xDC < 10 s; at ≥ 10 s in MP with
  facing ≠ 1: self-delete + `Explode_Create(5.0, 5.0, 50.0, …, 0x6000052, …)` [M case 6].
- **7 mounted** — `GT_TakeControl` from state 4 only: aim from input actions 3/4 scaled by
  0.024543693 (0x3CC90FDB), clamped ±0.5 (pitch, ext+0x90) / ±1.5707964 (yaw, ext+0x94);
  action 9 fires on the same 0.1 s cadence + `PlrStat_LogShotFired`; action 14 (0xE) →
  `GT_LoseControl` [M case 7].

Death gate before the switch [M 0x180498:236-258]: health < 0.0 (shots and explosions through
`GT_ApplyDamage` 0x1816d8, `Explode_Propagate` case 0x36 [M 0x17c518:190-193]) **or** the shot
counter at 0 → if state 7 `GT_LoseControl` first, then state 6, timer 0, one
`Emitter_Create(10.0, 0, -2.0, base, up)` burst.

## `GT_Track` 0x17f9c8 — target selection

- SP: range 2500.0; the player (alive, health > 0, targetable flag) plus every guard
  (AI flags & 0x600 == 0, & 0x100 set, health > 0, type ≠ 0x11, not self-deleting) and every
  copter (distance ×0.25) within range, up to 63 targets [M 0x17f9c8:96-176].
- MP: range 500.0 over the 8 player slots, owner skipped [M 0x17f9c8:177-215].
- Sort by distance, walk: keep the first two with LOS (`Collide_LineOfSight` from the muzzle
  point, mask 0x72f); return the current target while it is one of those two, else the first
  LOS candidate; none → 0 [M 0x17f9c8:246-300].

## The rest of the group

- `GT_RandMove` 0x180348: `Rand_FRandHalf(π)`, `Rand_FRandHalf(2π)` → a random sweep matrix
  (RotMatrix × the aim matrix), slerped at 4·REC_FRAME_RATE, copied onto the turret and the
  three registered objects' matrices [M].
- `GT_Delete` 0x181678: `Script_Free` both scripts [M].
- `GT_Disable` 0x1816a8: state 4 → store 4 in obj+0xF6, state 5, timer 0 [M]. Reached from
  `Bullet_DoTrails` 0x12d890:374 (a turret bullet hit) and `Explode_Propagate` case 0x36 [M].
- `GT_Activate` 0x1816f0: states 0/1/2/3/6 ignore; otherwise facing 0 → timer 0, state 2
  (re-holster) [M]. Callers: `Player_Activate` 0x1a5170:147, `SP_Activate` 0x1dc338:68 [M].
- `GT_TakeControl` 0x181740 / `GT_LoseControl` 0x181408: the mount pair (state 4 ↔ 7,
  ext+0xF2, the player's substate/cam/action plumbing) [M]. `GT_LoseControl` also from
  `MP_RestartScenario` 0x183fc0:26 and the death gate [M].
- `GT_PlayerHasDied` 0x181808: the first state-7 turret owned by the dead player object loses
  control [M]; caller `Player_CheckForDeath` 0x190810:38 [M].

## Sub_* — never reached on the shipped data

`Sub_Create` 0x1ad6c0 (placement kind 0x36, `parsemap_create_dynamic_objects`
0x1d04f0:215) sets class 0x4A ('J') [M 0x1ad6c0:21], whose `control_funcs[74]` row is
{`Sub_Update` 0x1ad200, `Sub_Collide` 0x1ad868, `Sub_Delete` 0x1ad870} [M symtab + 0x26fc08].
The disc placement census has **no 0x36 rows** [D docs/systems/script-census.md; the proof in
docs/systems/ai-spawned-objects.md "Proven never called"], so no Sub object ever exists:
`Sub_Update`/`Sub_Collide`/`Sub_Delete` never dispatch, `Sub_Laser` 0x1acda0 and
`Sub_DeployTorpedoes` 0x1acf68 have only `Sub_Update` as caller [M 0x1ad200:111,127], and
`Sub_ApplyDammage` 0x1ad570's only other call site, `Explode_Propagate` case 0x3D ('J' check)
0x17c518:207, needs an existing Sub [M]. See docs/port-map/unreached.md.
