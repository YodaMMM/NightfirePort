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
## NDrone2 router/debug states (unported-guard-ai-other-5, 2026-10-21)

The 17 DSTATE fns of task 858 live in `NDrone2_StateFuncs` 0x29b300 at ids 0x3b, 0x57, 0x5b, 0x5d,
0x61, 0x80, 0x9a, 0x9b, 0x9d, 0xa0, 0xa1, 0xb5, 0xb8–0xbc. Entry mechanisms in the game [M]:
`Drone_SM_SetState` 0x1758c0 (stores the id at SMInfo+0xc) with a literal, with a variable, or with a
message-carried id (`DSTATE_Disabled` 0x172440 does `SetState(info, *(msg+0xcd8), 0)`), plus the
DroneTypeSettings start column ([D/L] docs/fields/dronetype-tables.md) and the cover-anim completion
states. Evidence used per id: whole-decompile plain-name caller census, a literal + hex-assignment
sweep of every `Drone_SM_SetState` call site, the variable-id census (131 calls — only
`ReactToDroneAlertMsg` 0x1437a0:263 carries one of ours: 0xa0), the DTYPE start table, and the
mode-derived starts of `DoTypeSettingsOLD` 0x14d2e8 / `GetDroneTypeAttackTypeFriend` 0x14cbd8
({8, 0x29/0x2a, 0x85, 0x37, 0x30, 0x31, 0x36, 6, 4} — none of ours). `init_DMODE_Interogator`
0x14e238 is a Defaults-only stub.

| addr | name (state id) | row / evidence |
|---|---|---|
| 0x1729f0 | NDrone2_DSTATE_Interogator (0x3b) | never assigned: no SetState literal/variable path, no DTYPE start, no direct caller. Body: ENTER → Idle 4 [M 0x1729f0]. `init_DMODE_Interogator` 0x14e238 = Defaults only [M] |
| 0x173178 | NDrone2_DSTATE_Alerted1stEncounter (0x57) | never assigned. The `0x57` in `DroneFunc_HandleImpact` 0x1469b8:266 is an ANIM id (arg of DroneAnim_CallAnim), not a state [M]. Body: ENTER → 0x5a CombatOutOfRange [M 0x173178] |
| 0x173338 | NDrone2_DSTATE_CombatTooClose (0x5d) | never assigned by code or the DTYPE table; `DroneFunc_CombatState` 0x148338 emits 0x58/0x5a/0x5c/0x5e/0x68/0x70, not 0x5d [M — the port's coverage list]. Body: ENTER sets +0x4F8\|=0x10000 → 0x67 AimStand [M 0x173338]. Live check L-024 (docs/live/backlog.md:202, gdbwatch 0x173338) still open — the TooClose ×2 damage lives in FireWeapon's own close-threshold check [M 0x30cb14], not this state |
| 0x173390 | NDrone2_DSTATE_Obstructed (0x61) | entered only by msg 0x1b, which has no decompiled sender [M 0x15c498:95-97 + grep — already documented at crates/nf-game/src/drone.rs PATROL UPDATE]. The `0x61`s in CreateNinjaEyes 0x149600 / SearchArea 0x165bf0 / HeardNoise* 0x16b0e0/0x16b540 are field/anim values, not SetState ids [M spot-checks] |
| 0x173570 | NDrone2_DSTATE_HoldItRightThere (0x80) | never assigned. DefaultInit's `0x80` is a field store at +0xBA0 [M 0x14b300:825], not a state |
| 0x173b38 | NDrone2_DSTATE_SeenOpponent (0x9d) | never assigned. Body: ENTER → 0x56 Attack [M 0x173b38] |
| 0x173bb8 | NDrone2_DSTATE_SeenExplosive (0xa1) | never assigned. Body: no-op (accepts msgs 0/1/3) [M 0x173bb8] |
| 0x173f68 | NDrone2_DSTATE_FailMission (0xb5) | DTYPE row 11 start state [D/L dronetype-tables.md:93], but that row's creation path (mode 11, `NDrone2_init_DMODE_MissionFailer` 0x14df28) is unreached — see the init_DMODE section above. Body: ENTER SetMissionFailReason(0) → return-state +0x5a2 [M 0x173f68]. The port folds a 0xb5 start to 0x12 defensively [M drone.rs start_state] |
| 0x174150 | NDrone2_DSTATE_Tester2 (0xb8) | debug: ENTER plays anim 0xd, UPDATE → Tester1 0xb7 [M 0x174150]; never assigned |
| 0x1741c8 | NDrone2_DSTATE_Tester3 (0xb9) | debug; never assigned. InjuredTalk's `0xb9` is a per-level constant (case 0x7000011..0x700004a → 0xb9 = a level table index), not a state [M 0x13fa20:33-37] |
| 0x1742d8 | NDrone2_DSTATE_Tester4 (0xba) | debug: ENTER anim 10 + timer arm, msg 0xc → Tester1 0xb7 [M 0x1742d8]; never assigned |
| 0x1743d8 | NDrone2_DSTATE_HangUp (0xbb) | never assigned. Body: msg 1 clears +0x98 on the two objects at drone+0x530/+0x534 [M 0x1743d8] |
| 0x174430 | NDrone2_DSTATE_WaitForever (0xbc) | never assigned. Body: no-op parking state [M 0x174430] |

Reached / cited by this task (NOT unreached): 0x173280 CombatNewSighting 0x5b and 0x1739b8/0x173ac0
UnderCoverTypeChange/Leave 0x9a/0x9b (port consts + arms, cited in crates/nf-game/src/drone.rs),
0x173b78 SeenDroneShot 0xa0 (entered from `ReactToDroneAlertMsg` 0x1437a0:263,368 — the port folds
it into the FORCED_ATTACK send, cited in drone.rs). Caveat for all rows: the msg-carried state
channel (`DSTATE_Disabled` 0x172440 reads the id from the message at +0xcd8) means a shipped level
script could command any of these ids; no disc-side census was done in this window — a live check
(gdbwatch on each address during a full game run) would settle it.

# Unreached scripts-cluster functions (unported-scripts-1, 2026-10-21)

## M_SPIN widget: Spin_Init 0x2224c8, Spin_SendMessage 0x2226d8, Spin_Update 0x222948

The Spin control (menu widget type 0xb — `Spin_Init` self-stamps `+0x7b = 0xb`
[M 002224c8:67]; dispatch arms in `__Menu_SendMessage` 0x1f1c18:45-46 and
`Menu_Update` 0x1f1dc0:32-33) is an up/down integer spinner: a centred Label
showing `max − step + min` [M 00222948:43] over an embedded M_SCROLL at +0x94
whose d1 def word is the value range [M 002224c8 Scroll_Init call].

The only trigger for all three is an authored **ctype-11 CONTROL** in a menu def:
`MenuManager_Create` 0x21b908:318 → `Manager_SendMessage` 0x21c858 case 0x42
(:158, :220-227) mallocs 0x318 bytes and calls `Spin_Init` when the NEW_CONTROL
type byte is 0xb. Disc census [D]: kind-8 entry 08000002 of 07000048.bin (the
whole front-end def; all pages carry file hash 0x80000002) parses to 52 pages /
830 controls with ctypes {0 (Label): 824, 1 (Button): 1, 2 (Checkbox): 1,
3 (Combo): 4} — **zero ctype 11** (and zero List/Radio/Scroll/Text/Window/Memo).
A signature scan for CONTROL records with type byte 0xb over the entire 764 MB
FILES.BIN found **0 hits** [D]. The shipped front end builds its spinner-looking
UI from Combo/List/Label composites instead (menu.rs:1148 "row 0 is the
controller-style spinner — the list control 0x10000025").

So no caller runs on any on-foot level, the front end or MP: recorded unreached.
`menudef.rs` already parses CONTROL ctype as a raw u8, which is the right depth
of support for a dead widget class.
## unported-hud-1 (2026-10-21) — M_TEXTBOX: no shipped page builds one

`Text_Init(M_TEXTBOX*, NEW_CONTROL*, M_CONTROL*)` 0x222b80 (2688 B), `Text_SendMessage`
0x223600 (456 B) and `Text_SetupChar` 0x224098 (116 B) are **not reached by the shipped game**.

Evidence: the only construction path is the front-end control factory
`Manager_SendMessage` 0x21c858, whose type-0xc arm (:206-208, `Menu_Malloc(0x9b0)` +
`Text_Init`) runs for a page-def control whose type byte is 0xc [M]. The shipped front-end
menu def (kind-8 entry 08000002 in 07000048.bin — the file `MenuManager_Create` 0x21b908 is
called with, hash 0x80000002 [D]) contains **zero controls with ctype 0xc** — the census in
`crates/nf-formats/tests/hud_pane_census.rs::menu_def_textbox_census` parses every page and
every control and prints `[]` [D]. With no textbox control, `Text_SendMessage` (the control's
message switch, called via `__Menu_SendMessage` 0x1f1c18 / `TextBox_Update` 0x2237c8) and
`Text_SetupChar` (called only by `Text_Init` 0x222b80 [M grep]) are never reached either.

Caveat: the census covers the one menu def the game boots with [D docs/systems/frontend.md,
menudef.rs header]; no other kind-8 page-def entry is loaded by the boot path the port
reconstructs. `Text_SendMessage`'s opcode surface (0x14/0x16/0x21/0x2b/0x39/0x3b/0x3d/0x47/0x53
[M 0x223600]) is fully documented in docs/systems/hud-panes.md so a future task can port the
control if a def with a type-0xc control ever surfaces (e.g. from a different region build).
