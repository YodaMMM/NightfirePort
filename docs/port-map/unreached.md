# Unreached game functions (never called on any shipped path)

Evidence tags: [M] decompile, [D] disc bytes. Created by unported-guard-ai-other-2
(2026-10-21); sections are appended per task, newest last.

## NDrone2_init_DMODE rows 1-22 (unported-guard-ai-other-2)

The functions below are `DroneModeSettings` 0x29ad40 rows 1..22 slot +8. The only
reader of that slot is `NDrone2_DoModeSettingsOLD` 0x14cfa0 (row `param_2` ≤ 0x23),
whose only caller is `NDrone2_DefaultInit` 0x14b300:495-502 under placement
parameter 5 < 0x24. The three shipped creation paths give parameter 5 = 100
(all 555 drone placements, [D census]) or 0 (MP bots `BOT_init` 0x1249e8 and the
coder event 0x07000001 0x0600003B — both memset the DIVars, [M 0x1249e8:101 /
0x13a108:55]); 0 selects row 0 (`NDrone2_init_DMODE_Normal` 0x14d958 →
`NDrone2_init_DMODE_Defaults` 0x14d528, ported) and 100 dispatches to
`DoModeSettingsNEW` 0x14c600, which reads no row slot. No other reader exists in
the decompile. Rows 31/32/34 (fn 0) and rows 23-30 (owned by 856) are not listed
here.

| addr | name | row (mode, team) |
|---|---|---|
| 0x14d978 | NDrone2_init_DMODE_Guard | 1 (mode 1, team 1) |
| 0x14d998 | NDrone2_init_DMODE_Retreater | 2 (mode 1, team 1; alert word 1.0) |
| 0x14d9b8 | NDrone2_init_DMODE_Sniper | 3 (mode 2, team 1) |
| 0x14da78 | NDrone2_init_DMODE_Stealth | 4 (mode 1, team 1) |
| 0x14dab0 | NDrone2_init_DMODE_Attacker | 5 (mode 6, team 1) |
| 0x14dad0 | NDrone2_init_DMODE_RunToPoint | 6 (mode 7, team 1; alert 1.0) |
| 0x14db18 | NDrone2_init_DMODE_Assassin | 7 (mode 3, team 1) |
| 0x14db38 | NDrone2_init_DMODE_HostageKiller | 8 (mode 4, team 1) |
| 0x14dc18 | NDrone2_init_DMODE_Hostage | 9 (mode 5, team 3) |
| 0x14dc88 | NDrone2_init_DMODE_HostageTied | 10 (mode 5, team 3) |
| 0x14dcf8 | NDrone2_init_DMODE_JustStand4Demo | 11 (mode 8, team 3) |
| 0x14dd20 | NDrone2_init_DMODE_DeleteMe | 12 (mode 21, team 1) |
| 0x14dd28 | NDrone2_init_DMODE_Civilian | 13 (mode 9, team 3) |
| 0x14de48 | NDrone2_init_DMODE_CivilianScared | 14 (mode 10, team 3; alert 1.0) |
| 0x14df28 | NDrone2_init_DMODE_MissionFailer | 15 (mode 11, team 1) |
| 0x14df68 | NDrone2_init_DMODE_Mayhew | 16 (mode 12, team 2) |
| 0x14df90 | NDrone2_init_DMODE_Ninja | 17 (mode 13, team 1) |
| 0x14df98 | NDrone2_init_DMODE_AlarmRaiser | 18 (mode 14, team 1; alert 1.0) |
| 0x14dfe0 | NDrone2_init_DMODE_SearchLight | 19 (mode 15, team 1) |
| 0x14e050 | NDrone2_init_DMODE_Zoe | 21 (mode 17, team 2) |
| 0x14e098 | NDrone2_init_DMODE_PartyGirl | 22 (mode 18, team 3) |

Row 20 (`NDrone2_init_DMODE_Ambush` 0x14e018, mode 16) is not in this task's list
and is not claimed here. `Defaults` 0x14d528 itself is reached (via row 0) and is
ported — it is listed in the task only as the row-0 body's callee.

## 857-unported-guard-ai-other-4 — the NDrone2 states with no shipped-path caller

Evidence basis: the exhaustive entry-path sweep in docs/systems/guard-ai-other-4.md —
every `Drone_SM_SetState` call site (balanced-paren scan), every state-selector return, every
`DroneAnim_CallAnim` follow-state argument, every `DroneFunc_HandleImpact` resume argument,
every constant write to the return-state slots `+0x5A2/+0x5A0`, the full `DroneTypeSettings`
table 0x29aee8 dump, and the spawn initial-state path (`Drone_SM_InitObject` 0x175838 ←
`obj+0xF4`, zeroed by `NDrone2_DefaultInit` 0x14b300:58) [M].

| addr | name | state | why it is never reached |
|---|---|---|---|
| 0x166840 | NDrone2_DSTATE_AimStandDiscard | 0x6a | No entry path anywhere in ACTION.ELF [M]: no SetState constant, no selector returns 0x6a (CombatState 0x148338 / ChooseCombatMove 0x150c18 / EnemyAlerts 0x177c18 / AlertSound 0x177800 enumerated), no CallAnim follow (AimStandReload's reload anims follow 0x68, not 0x6a [M 0x1666d8]), no `+0x5A2` writer, no DroneTypeSettings row (dumped: 6/0x29/8/0xa/0x56/0x85/0xb7/0x10/0x12/0xb5/0x1c/0xa6/0x89/0x37/4/0x2e/0x30/0xb4/0x31/0x32/0x36/0x2a/0x8e/0xbd/0xc3+), no HandleImpact resume [M]. The weapon-discard anim 0x45 with follow 0x66 (DrawWeapon) suggests a cut weapon-loss flow [S]. |
| 0x166940 | NDrone2_DSTATE_Prone | 0x6b | Same sweep [M]: no producer of state 0x6b in any entry path. ChooseCombatMove's crouch option is 0x6f AimCrouch (prop 0x11); the prone anims are not referenced by any combat chooser. Only a script writing `obj+0xF4 = 0x6b` before `Drone_SM_InitObject` could spawn a drone into it [M]; DefaultInit zeroes the field. |
| 0x166a28 | NDrone2_DSTATE_ProneFire | 0x6c | Only enterers are Prone itself (CallAnim follow 0x6c and HandleImpact(0x6c) [M 0x166940]) — and Prone is never entered (row above). The blind-cheat arm inside it (`switch_BLIND_DRONES` → Idle) does not create an entry path [M 0x166a28:19-25]. |
| 0x16db90 | NDrone2_DSTATE_Tester1 | 0xb7 | Debug/test family: SetState'd only by `NDrone2_DSTATE_Tester2` 0x174150 and `NDrone2_DSTATE_Tester4` 0x1742d8 [M] — a closed tester cluster (Tester1↔2↔4) whose only outside edge is inbound-none; also `DroneTypeSettings[8].ret = 0xb7` (drone-type 8's impact-return state, table 0x29aee8+0x60 [M]) — no shipped level data is known to configure drone type 8 [D census not run; flagged [S]]. |

Reached-elsewhere notes for the same task: `NDrone2_DSTATE_SeenDeadBody` 0x16acd0 (0x9e) and
`NDrone2_DSTATE_SeenSurrenderedDrone` 0x16ade0 (0x9f) are reached only through
`NDrone2_ReactToDroneAlertMsg` 0x1437a0, whose only caller is the MP bots'
`NDrone2_DSTATE_BotGlobal` 0x16e060 [M] — MP counts as a shipped path, and the port now delivers
the transition flow (`mp/bots.rs react_to_drone_alert_msg`).
