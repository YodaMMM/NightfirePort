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

# Unreached guard-AI functions (call-tree audit outcomes)

Functions the call-tree flags that the shipped game never reaches on an on-foot level, the front
end or MP — with the evidence. Ported/cited functions are not listed here; each has its citation at
the Rust code that does the work.

## NDrone2_DSTATE_CastleChatGuard1 0x1612e0 (state 0x36)

- Handler of `NDrone2_StateFuncs[0x36]` [L 0x29b300 table read]. In-code entries: the DTYPE-0x18
  home state (`NDrone2_GetDroneTypeAttackTypeFriend` 0x14cbd8 case 0x18 → +0x5a2 = 0x36 [M]) and the
  PlayScript redirect (`NDrone2_DSTATE_PlayScript` 0x15ba68 case 1, `*(char*)(drone+0xc5) == 0x18` →
  SetState(0x36) [M]).
- The disc census over all 99 `.bin` level archives (555 kind-0x0f drone placements) shows **no
  placement carries DTYPE (placement parameter 15) 0x18** [D census; docs/reports/
  unported-guard-ai-other-3b.md]. The port's `dtype_of` (`NDrone2_GetDTYPENEW` 0x14c938) never
  returns 0x18 either, so the PlayScript redirect's `+0xc5 == 0x18` precondition has no disc producer
  [M]. The mode-keyed start-state arm the port carries (`start_state`: mode 0x18 → 0x36) mirrors the
  game table for completeness; no disc placement produces mode 0x18 [D census of parameter 16: only
  0x00/0x02/0x03/0x04/0x05 occur].
- Remaining theoretical entry: a level script's Global msg-0x1d SetState (0x15b830 case 0x1d [M])
  naming state 0x36 — script-data-driven, not exercised by any placement or start script found in
  the census. Recorded unreached on the shipped placement data; a script-driven entry would be a new
  finding.

## NDrone2_DSTATE_InterogateWalk 0x162338 (state 0x3d)

- Handler of `NDrone2_StateFuncs[0x3d]` [L 0x29b300]. **No in-code entry exists**: an exhaustive scan
  of the decompile (SetState constant 2nd args, direct state-field stores, `return 0x3d/61` in state
  mappers) finds none — the interrogation family's own transitions go to 0x3c/0x3a/0x56/0x60 [M
  0x161a40/0x1616c0/0x161860; the scan is in docs/reports/unported-guard-ai-other-3b.md].
- Entry is data-driven only: `DroneTypeSettings` rows whose start/alert state is 0x3d, or the
  obj+0xf4 coder initial state. The interrogator DTYPEs 0x1b/0x1c come from the start-script
  specials 0x600089c/0x6000892 [M 0x14b300:613-627]; **no placement carries DTYPE 0x1b/0x1c as its
  parameter 15** [D census] and their DroneTypeSettings rows do not start in 0x3d
  (docs/fields/dronetype-tables.md [D ELF data]).
- The same Global msg-0x1d script caveat as CastleChatGuard1 applies.

## Data-driven states deliberately not proven unreached

- `NDrone2_DSTATE_EnemyMission` 0x15e1e8 (0x1b): entered from Idle/Alert ENTER when the mission-path
  route has pending nodes (`+0x9f2 != 0` after `AssignAIPath`) [M 0015bf18:36-60]. Prop 0x1d is set
  on 484/555 drone placements [D census], so the state is data-reachable; the port's route-walking
  machinery (`update_mission_route`, 0x154a80) exists and the state's port is future work.
- `NDrone2_DSTATE_CivilianMissionWait` 0x15dbb0 (0x17): entered from CivilianMission's msg-0x22 hold
  command [M 0x15d860:130-136]; whether any shipped script sends 0x22 to a CivilianMission drone is
  not censused — left open rather than claimed unreached.
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

## The Sub_* group (unported-level-objects-1, 2026-10-21)

The `SUBINFO` submarine (distinct from the `MiniSub_*` family): `Sub_Create` 0x1ad6c0 is the
placement dispatcher's case 0x36 only (`parsemap_create_dynamic_objects` 0x1d04f0:215) [M],
sets class 0x4A ('J') [M 0x1ad6c0:21], and `control_funcs` 0x26f890 row 74 is
{`Sub_Update` 0x1ad200, `Sub_Collide` 0x1ad868, `Sub_Delete` 0x1ad870} [M action-symtab +
0x26fc08-0x26fc10]. The disc placement census has **no 0x36 rows** [D
docs/systems/script-census.md; the proof is in docs/systems/ai-spawned-objects.md "Proven
never called on the shipped path"], so no Sub object ever exists on a shipped on-foot level,
front end or MP map, and nothing below can dispatch.

| addr | name | bytes | why never reached |
|---|---|---|---|
| 0x1ad200 | Sub_Update | 880 | control_funcs[74] slot 0 — runs only for a class-0x4A object; `Sub_Create`'s only caller is parsemap case 0x36, which has no disc rows [M callers / D census] |
| 0x1ad868 | Sub_Collide | 8 | control_funcs[74] slot 1 (`Collide_Update` 0x1ea398:59-63 dispatch) — same class gate [M callers / D census] |
| 0x1ad870 | Sub_Delete | 8 | control_funcs[74] slot 2 (`control_delete_object` 0x133178:16-17 dispatch) — same class gate [M callers / D census] |
| 0x1acda0 | Sub_Laser | 452 | only caller is `Sub_Update` 0x1ad200:111 [M] — never runs itself |
| 0x1acf68 | Sub_DeployTorpedoes | 664 | only caller is `Sub_Update` 0x1ad200:127 [M] — never runs itself |
| 0x1ad570 | Sub_ApplyDammage | 328 | two callers: `Explode_Propagate` 0x17c518:207 (its case-0x3D branch checks the victim's class byte == 'J' = 0x4A first [M]) and `Sub_Update` — both need an existing Sub [M callers / D census] |

The auxVec_Sub 0x1e4f48 hits in the same name grep are the vector-subtract helper
(`Creature_CalcWaypoints` calls it), not this family.
