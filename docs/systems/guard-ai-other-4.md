# Guard AI — the 25 never-mentioned NDrone2 states (857-unported-guard-ai-other-4)

The whole-ELF never-mentioned scan (master 20874c31, backlog/unported-all-2026-10-20) listed 25
`NDrone2_DSTATE_*` handlers whose name/address appeared nowhere in `crates/*/src`. This file is the
per-function disposition; the report is `docs/reports/unported-guard-ai-other-4.md`. Evidence tags:
[M] decompile, `work/decomp/ACTION.ELF/<addr>_<name>.c`; [L] EE snapshot; [S] suggested.

## How states are entered (the analysis method)

The state machine dispatches `NDrone2_StateFuncs[0x29b300][state]` through
`NDrone2_ProcessStateMachine` 0x175778; `Drone_SM_SetState` 0x1758c0 only queues
(`SMI+0x0C = state`, `SMI+0x18 = 1`) and `Drone_SM_RouteMsgDCV` 0x171df8 commits. An exhaustive
caller sweep of every entry path found in the decompile:

1. `Drone_SM_SetState(.., CONST, ..)` — balanced-paren scan of all 300+ call sites [M];
2. selector functions whose return value feeds SetState: `DroneFunc_CombatState` 0x148338,
   `NDrone2_ChooseCombatMove` 0x150c18, `DroneVision_EnemyAlerts` 0x177c18,
   `DroneVision_AlertSound` 0x177800, `NDrone2_EvasiveMove` 0x1506d8, `DroneFunc_FirstAttack` 0x148010;
3. `DroneAnim_CallAnim(.., anim, ?, followState, ..)` follow-state argument [M 0x13ab58 chain];
4. `DroneFunc_HandleImpact(.., resumeState, allow)` third argument [M 0x1469b8];
5. the return-state slots `+0x5A2/+0x5A0` — C writers: HeardNoiseSuspect 0x16b0e0 (=0x65),
   CombatOutOfRange 0x164690 (=0x7e), HandleImpact (=0x14), FirstAttack (=0x56), HostageSaved (=0xe),
   CivilianScared (=0x13), DefaultInit (=0xbd/0x8e), GetDroneTypeAttackTypeFriend 0x14cbd8 and
   DoTypeSettingsOLD 0x14d2e8 (from the `DroneTypeSettings` table 0x29aee8, dumped in full),
   `Drone_CoderCreate` 0x13a108 (script data), `BOTSTATE_gotoGoal` 0x1278a8 [M];
6. the spawn initial state `obj+0xF4` — `Drone_SM_InitObject` 0x175838 stamps all four SMI slots
   from it; `NDrone2_DefaultInit` 0x14b300 zeroes it, so the shipped initial state is 0 (Global)
   unless a script writes the field [M];
7. the switch-channel set (`NDrone2_PreDroneControl` 0x148db0:85 SetState with a variable) [M].

## Dispositions

### Ported this task (reached, had no port body)

- `NDrone2_DSTATE_Investigate` 0x168070 (state 0x7e): entered when the shipped blind-enemies cheat
  (`C_CHCHBLIND_Handler` 0x215d60 sets `switch_BLIND_DRONES` 0x30ce30) makes
  `NDrone2_DSTATE_CombatOutOfRange` 0x164690's ENTER park `+0x5A2 = 0x7e` and route to
  GoToGoalPosition 0x63, whose arrival SetStates `+0x5A2` [M 0x164690:26-32, 0x165640]. ENTER:
  vision on, `+0x4F8 |= 0x2000`, alert status 2 (share 0.66 — table 0x2f5cc8), anim 0x21; UPDATE:
  `DroneVision_EnemyLookForOpponent`, no pick and share < 0.66 → Idle; alerts through
  `DroneVision_EnemyAlerts`; impacts resume Attack 0x56; forced attack; explosives [M 0x168070].
  Port: `drone.rs` INVESTIGATE arms.
- `NDrone2_DSTATE_CrouchCover` 0x166d90 (state 0x6e): the mode-0x11 cover-croucher's
  `NDrone2_DSTATE_AimCrouch` 0x166f28 ENTER branch (mode byte `+0xC5 == 0x11`) SetStates here
  instead of playing the crouch-aim anim [M 0x166f28:15-22]. ENTER: alert status 0, crouch anim
  0x11, timer C = now + 2 s + Rand(60); UPDATE faces the opponent unless mode == 0xc; timer C and
  the impact resume land on AimStand 0x67 [M 0x166d90]. Port: `drone.rs` CROUCH_COVER arms +
  the AimCrouch mode branch.
- `NDrone2_DSTATE_HostageDie` 0x1725a0 (state 0x0b): `NDrone2_DSTATE_Hostage` 0x15cb78 case 0xb
  SetStates it [M 0x15cb78:61-64]. ENTER: alert status 3 (1.0), the death scream (sound 10,
  forced), anim 0x40 with follow-on HostageDead 0x0f, `NDrone2_LogDroneDeath` [M 0x1725a0]. Port:
  `drone.rs` HOSTAGE_DIE arm + `msg::DIE`.
- `NDrone2_ReactToDroneAlertMsg` 0x1437a0's dead-body / surrendered outcomes —
  `NDrone2_DSTATE_SeenDeadBody` 0x16acd0 (0x9e) and `NDrone2_DSTATE_SeenSurrenderedDrone` 0x16ade0
  (0x9f) are SetState'd by its msg-0x15 / msg-0x16 arms [M 0x1437a0:307-326, 283-290]; the only
  caller of 0x1437a0 is `NDrone2_DSTATE_BotGlobal` 0x16e060 [M], i.e. the MP bots. Port:
  `mp/bots.rs react_to_drone_alert_msg` (transition flow; the state bodies' presentation is not
  drawn — the main agent owns drawing).

### Already implemented — citations added at the Rust code

`NDrone2_DSTATE_SearchArea` 0x165bf0, `NDrone2_DSTATE_DroneStuck` 0x165ef0,
`NDrone2_DSTATE_AimStandReload` 0x1666d8, `NDrone2_DSTATE_AimCrouch` 0x166f28,
`NDrone2_DSTATE_AimCrouchFire` 0x167060, `NDrone2_DSTATE_AimCrouchReload` 0x167370,
`NDrone2_DSTATE_StepAimRight` 0x1676a0 (with Left 0x167590), `NDrone2_DSTATE_StrafeAimRight`
0x167980 (with Left 0x1677b0), `NDrone2_DSTATE_StrafeDodgeLeft` 0x167b50,
`NDrone2_DSTATE_StrafeDodgeRight` 0x167cf0, `NDrone2_DSTATE_RollRightCrouch` 0x167f80 (with Left
0x167e90), `NDrone2_DSTATE_InitPatrol` 0x1724f0, `NDrone2_DSTATE_UnderCoverAim` 0x16a2d0,
`NDrone2_DSTATE_UnderCoverReturn` 0x16a9c0, `NDrone2_DSTATE_UnderCoverLeaveNow` 0x16ab38 — each
name + address now cited at the match arm that does the job in `crates/nf-game/src/drone.rs`.
Reachability chains: the Step/Strafe/Dodge/Roll/AimCrouch/AimStand family from
`NDrone2_ChooseCombatMove` 0x150c18 (behaviour props 0x13/0x41/0x42/0x2e/0x11) via
`DroneFunc_CombatState` 0x148338 / AimBackoff 0x166b98 / `NDrone2_EvasiveMove` 0x1506d8 [M];
AimStandReload from CombatState's empty-clip arm [M 0x148338 case 0x10]; DroneStuck from
CombatState's MoveToObject-result-5 arm [M 0x148338 case 5]; SearchArea from
HeardNoiseSuspect's `+0x5A2 = 0x65` [M 0x16b0e0:62,153] read back by the GoToGoal family;
InitPatrol shown live on the shipped path by the [L ee_start_05.bin] fresh-boot snapshot
(guards cur=4 Idle, prev=6 InitPatrol); the UnderCover family from UnderCoverIdle 0x169e58 /
UnderCoverLeave 0x173ac0 [M].

### Never reached — evidence in docs/port-map/unreached.md

`NDrone2_DSTATE_AimStandDiscard` 0x166840 (0x6a), `NDrone2_DSTATE_Prone` 0x166940 (0x6b),
`NDrone2_DSTATE_ProneFire` 0x166a28 (0x6c), `NDrone2_DSTATE_Tester1` 0x16db90 (0xb7).

## Live checks for the main agent

1. Blind-cheat Investigate: enable the "Blind Enemies" cheat (front end), play any guard level,
   shoot a guard beyond his combat range — expect him to route to Bond's last position and then
   stand in the alert-idle pose (state 0x7e, `Drone+0x10C`). Address: drone SMI current state.
2. Cover-croucher: a mode-0x11 guard (property `+0xC5 = 0x11`) entering combat should crouch in
   cover for 2-3 s and stand up into AimStand (states 0x6e → 0x67).
