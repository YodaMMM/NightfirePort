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
- **3 finished** — when both scripts are done: mounted → `GT_LoseControl`; then **facing 0** →
  `Pickup_Create(obj+0xC0, 0, obj+0x90, 0x2000366, kind 0, id 0x52, 1 round, switch 0, sound 0xF5,
  at rest, respawn 0, lifetime 0)` + self-delete (the Ronin case back on the floor); **facing ≠ 0** → 0
  [M case 3, disasm 0x180914-0x18098c: `lhu v0,236(s2)` / `bnel v0,zero` with the state-0 store in
  the delay slot]. (Corrected 2026-11-01: the earlier reading had the arms swapped, so the port's
  facing-1 wall guns deleted themselves after their first re-holster.)
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

## The Phoenix Ronin (weapon 0x52 / 0x53) — the player-deployed turret

- **Where the campaign gives it** [D placements dump]: 0700000c (Chain Reaction) #561
  `oPickUp_Briefcase_3rd_LOD0` (53.98, 1.26, −173.46) and 07000012 (Phoenix Fire, part 2) #753
  `oPickUp_Briefcase_LOD0` (13.07, 0.39, 15.71) — type 0xF0, kind 0, id 0x52, 1 round. No SP loadout
  equips it [M Player_InitWeapon 0x1a7ef8]; the default (non-campaign) kit does.
- **Deploy** [M Player_WeaponInitBullet 0x1a4bf0:265-282]: the pellet loop calls
  `GT_DeployMiniGun(obj+0x30 + Mat_GetDir(obj+0x90)·1.0, obj+0x90, player)` instead of `Bullet_init`;
  a 0 return refunds the round (`BLData+0x590 += 1` = clip[0x52]). `GT_DeployMiniGun` 0x181560:
  `Collide_PlaceObject(0.5, 0.6)` 0x1ecfb8 (the 3.0 down ray, seat = hit + n·0.5, the −0.13 / 0.2 /
  0.96 plane gates, the 0.6 support reach), then `GT_Create` with the stack record facing 0, param 1 = 2,
  scripts 0x06000045 (gun: 0x200035a + 0x2000361/0x2000362, `GT_Register` f(2)/f(0)/f(1)) and
  0x06000044 (case halves 0x2000358 ×2, body 0x2000359), enable 0, the disable word unwritten (stack
  residue) [M disasm 0x1815e0-0x181614; D nis dump], glist 0x2000359, owner = the player.
- **0x52 → 0x53** [M Player_HandleHasNoAmmo 0x1a2ba8 'R' arm, from Player_WeaponFiring
  0x1a5460:442-455]: clip[0x52] < 1 → owned[0x52] = 0, `Player_EquipWeapon(0x53, 999)`, request 0x53.
- **Targets** [M GT_Track 0x17f9c8:62-64]: SP, owner == `glb_players` → the player block is skipped;
  the guards (facing 0) within range² 2500 in the front cone with LOS. The rounds are the player's:
  `Bullet_init(−1, ext+0xB8, …, 0x2c55d0)` [M 0x180498 case 4 / case 7].
- **Remote control** [M 0x1a4bf0:113-133, GT_TakeControl 0x181740]: firing 0x53 finds the first
  class-'6' object the player owns → state 7, substate 0x10, `Player_WeaponNone`, cam mode 0xf →
  `HUD_Reset` + pane 18 (`Player_SetCamMode` 0x195928). The view: registered[0]'s matrix, eye
  (0, 0.15, −0.4) in its frame [M Player_PositionCamera 0x18f5d8:277-299]. Case 7 aims with actions
  3/4 (±π/2 yaw, ±0.5 pitch), fires on action 9 every 0.1 s, and action 0xE (mask 4) →
  `GT_LoseControl` 0x181408 (weapon +0x62 = +0x63 = 0x53).
- **Pick-up** [M Player_Activate 0x1a5170:146-148, GT_Activate 0x1816f0, 0x180498 cases 2/3]: E on
  it → state 2 → 1.2 s → 3 → both scripts done → the 0x52 case (1 round) on the floor, the turret
  deleted; walking over the case takes it.
- Port: `nf_game::gt` (`from_deploy`, `place_object`, the owner skip, the case-3 drop),
  `Drones::deploy_ronin` / `ronin_take_control` / `ronin_view` / `gt_activate_pick`, the viewer's
  0x52/0x53 arms and the pane-18 draw; pinned by `crates/nf-view/tests/ronin_live.rs`. Open: the
  `Collide_Pick` volume is approximated (triangles within 0.6 of the sweep) [S]; the gun's
  look-at/aim pose is an angle ease standing in for `Quat_Slerp_Acc` [S].
- **Barrel spin** [M 0x180498:360-382, 496, 536-556]: registered[1] (the 0x2000362 barrel) is drawn at
  `RotMatrixZ(ext+0xD4)` · its script pose on the ticks the spin block runs — the angle is absolute
  (wrapped into (−π, π) by `fmodf`), not accumulated in the matrix. Case 4 runs it on every tracked
  frame at a fixed 2·2π/60 per tick (ext+0xD8 untouched); case 7 decays ext+0xD8 ×0.9 per tick,
  re-arms it to 2·2π/60 on every held-trigger tick, and spins while it is non-zero. The mounting
  press is already a held trigger for the same frame's case 7 (the turret updates after the player).
  Live [L PCSX2 audits/render-1101-roninview + roninview2]: an 8-frame press, 45 frames later
  ext+0xD4 = −2.741 and the barrel's row0 = (cos, sin)(−2.741) of the script-A pose. Port:
  `GTurret::spin_draw`, scene.rs `gt_barrel_spin`; `gt` unit tests + `ronin_live`
  `ronin_barrel_spin_on_the_press`.
- **The barrel end in the remote view** [L PCSX2 audits/render-1101-roninview2 eye A/B]: the eye sits
  0.104 ahead of the barrel's origin, 0.15 above its axis, so the tube strips (glist z 0.031..0.369)
  start behind it. The PS2 shows the muzzle ring's face (the octagon with the hole) and no tubes.
  Moving the eye literal from −0.4 to −0.5 (all tube vertices in front of the eye) shows the tubes,
  but only outside the ring's outline. The VU1 clip program cuts the strips at the 0.1 near plane
  [M micro 0x241e30 `_$ROTATECLIPED` … `_$ZMINUS`], where zbuf = `Ps2Far1`. On 0C that is past the
  Z24 range (render.md), so the clip vertices wrap to a far depth and the strips lose the depth
  test to the ring behind them. Port: on levels whose wrap distance is past the near plane, the
  mounted Ronin's script parts carry the distance in their palette m[3][3], and the world shader
  drops their triangles that have a nearer vertex (`ronin_view_wrap_marks`; `ronin_live`
  `ronin_remote_view_marks_the_wrapped_parts`). On 07000012 (default Far1) the wrap sits inside the
  near plane and the tubes draw. The general garbage-depth effect on other near-clipped geometry
  is not modelled.
- **The shot gauge** (the red bar on the right of the remote view): the ammo pane's substate-0x10
  arm (hud.md).
