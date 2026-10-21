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

# unported-front-end-menus-3 (2026-10-21)

## Camera modes 3/4/5: camera_freechase 0x1ba1c8, camera_fixed 0x1ba4e8, camera_snake 0x1baa78

All three are `Player_PositionCamera` 0x18f5d8 switch arms — case 3 → `camera_freechase`
(:157-161), case 4 → `camera_fixed` (:162-166), case 5 → `camera_snake` (:167-171) [M].
The switched byte is `BLData+0x950` (the camera mode). Its only writer is
`Player_SetCamMode` 0x195928:11 [M whole-decompile grep of +0x950; camera_freechase itself
resets it to 0 at 0x1ba1c8:106]. All 20 `Player_SetCamMode` call sites pass mode
{0, 1, 2, 0xb, 0xc, 0xd, 0xe, 0xf} — none passes 3, 4 or 5 [M census of every call site,
0x1910f8:118 through 0x184bd0:71]. The script side's only camera-mode path is NIS event 9 →
`Player_Cam2Mode` 0x195fc8, which maps to 2 or 0 only [M 0x195fc8:10-16]. Corroboration:
docs/systems/uncited-audit.md:226-228 already records modes 4/5/6/0xa dead (the same census
covers 3). So no code runs these three handlers on any on-foot level, the front end or MP:
recorded unreached. Residual caveat: a data-driven `+0x950` write from script bytes cannot be
fully excluded from decompile alone — a gdbwatch on `*(u8*)(BLData+0x950)` across a full
SP+MP run would settle it conclusively.

## QuickSort dead comparators: CompBoxList 0x1d3390, CompMsgTxt 0x1d3400

The seven `Comp*` comparators (`CompBoxList` 0x1d3390 … `CompStrings` 0x1d34f0) are not
passed as function pointers anywhere: they are dispatch-table targets of `QuickSort`
0x1d30a8:61-67 — the `COMP_FUNCT` id selects a jumptable entry at 0x304fc0 [M dump of
work/elf/ACTION.ELF: id 0→CompSpr 0x1d3320, 1→CompLight 0x1d3350, 2→CompHitData 0x1d33c0,
3→CompDrawCels 0x1d3430, 4→CompAlphaObj 0x1d3458, 5→CompMsgTxt 0x1d3400, 6→CompTargetObj
0x1d3490, 7→CompMenuCtrl 0x1d34c8, 8→CompStrings 0x1d34f0, 9→CompBoxList 0x1d3390, 10→
a dynamic comparator]. A binary-wide census found exactly 15 `jal QuickSort` sites, all 15
decompiled, and **none passes id 5 or id 9** [M]. So `CompBoxList` (key = f32 at
`*elem+4`, the box-list entry) and `CompMsgTxt` (key = s8 `isNew` at TXTMSG+0x1d) are dead
table entries: recorded unreached.

Correction to prior art: docs/reports/objectives-0b-start.md:115 implies the objective
message queue sorts TXTMSG records by the `isNew` byte — contradicted by the census; the
message queue is FIFO via the CirBuf ring (CirBuf_Add 0x1cd040, replaced by the port's
subtitle queue). Also fixes a mislabel in docs/systems/fix-cel-order.md:28, which calls
0x1d3320 "CompLight": 0x1d3320 is CompSpr; CompLight is 0x1d3350.
## unported-front-end-menus-5 — the debug TWEAKS pages, the FMV island, the dead hub and the C_CHCH* cheat toggles (2026-10-21)

**P_TWEAKS_Handler 0x2067c8 / P_TWEAKS2_Handler 0x207950 (debug "TWEAKS" pages) — UNREACHED.**
Sole caller `Handler_HandleMessage` 0x2029e0 (itself only fed by `Manager_HandleMessage` 0x21df10):
they are the page handlers of menu pages **0x40000044 / 0x40000046**, routed by the receiving
control's id [M 0x2029e0:505-507, 439-440]; the handlers then switch on the page msg id
0x4c/0x4b/0x50/0x5d — no debug-flag gate, disasm 0x2067cc-0x20683c. The only id that can ever reach
them is the page id itself: 0x40000044 occurs in the whole decompile only in the dispatcher,
0x40000046 only there and in P_TWEAKS's own "more tweaks" arm, and neither occurs in any shipped
data as a message target or `Manager_SendMessage(0x44, …)` argument [D structured walk of all 35
kind-8 menu defs on the ISO + raw aligned dword scan of ACTION.ELF = 0]. The pages do register —
but only in the mission archives' pause def `08000001` (sha1 f0225d338b93, all 33 mission .bins),
whose openers are KMSG `{target −3, msg 0x44}` on the confirm scripts of three rows of hub page
**0x40000007** — a page with no dispatcher arm, zero code references and zero data references
anywhere in the shipped disc [D; the shipped pause rows go through C_GCPAUSE 0x204408 /
P_PAUSE 0x203c28 only]. Debug-only leftover; the gameplay globals they tune (`DroneDamage_*`,
`DroneArmour_*`, `Plr_DMod_*`, `DroneFiring_*`) are already modelled by
`nf_formats::tuning::Tuning` / `read_tuning_vars` (ReadTuningVars 0x1d3a80). Note for
docs/systems/unlock-popups.md §4: the P_CHEATMEDAL page record + its trigger row 0x1000021e ship
in the mission pause def 08000001 behind the same dead hub (the front-end def doesn't have them).

**P_FMVTEST_Handler 0x20a968 — UNREACHED** (debug). Dispatches on page **0x4000004f**, a
movie-picker page that exists in the front-end def 08000002 [D PAGE @ ISO 0x192a8796: list control
0x10000229 (134,75,372,331), hidden on entry, focused by the page-open script]. Nothing reaches it:
no code path sends 0x44 → 0x4000004f (the id occurs only in the 0x2029e0 dispatch), no
`GameState+0x28` writer stores it (writers store 0x40000034/09/1c/33/36/42/43), and a FILES.BIN-wide
scan finds no MGRMSG/KMSG navigation to it [D]. On open it fills the list with 22 movies
("Castle - Halo Jump" … "Menu - Making Of", 0x3090f0..) [M 0x20a968:14-36]; accept stores the pick
into `movie_hashcode` 0x30d374 and goes to page 0x40000050 [M :10-11] — a page with **no dispatch
arm at all** and a single placeholder control [D 0x192a886c]. Debug-only FMV preview island;
superseded by the port's `nf-view` movie harness (cited in movie.rs).

**C_LBPMMAP_Handler 0x215ac8 — UNREACHED** (debug). Dispatches on control **0x10000010**, a
(50,64,258,376) map board that exists only on the per-level menu-def page **0x40000007** (33 defs,
hash 0x80000002; the page-open script focuses the board and its siblings are the C_CHCH* cheat rows
— an in-level debug menu) [D]. The page is never opened: no code references 0x40000007 (0 hits in
the decompile; every `GameState+0x28` writer stores 0x40000034/09/1c/33/36/42/43), and no def
MGRMSG/KMSG navigates to it [D]. Its behaviour, for the record [M 0x215ac8]: msg 0x49 stores
GameState+0x28 = 0x40000009 (title) and GameState+8 = the board's map value; accept sets
`MPSettings+0x1AC` (player count) = 1, `ResetMap_LevelToLoad` of the selected map,
`GameFlow_PushState(80.0f,7,0xff)`, `SFXFadeDown(1)`, `MenuManager_Delete` — a 1-player quick
launch; finalize adds one list item (string 0x308830, archive 0x07000048). The port's
`mp_quick_start` already covers Quick-Game launching through the shipped scenario path (cited).

**The C_CHCH* cheat-toggle controls — C_CHCHFLY 0x215c70, C_CHCHDRONES 0x215cf0, C_CHCHBLIND
0x215d60, C_CHCHWEAP 0x215dc8, C_CHCHHEALTH 0x215ee0, C_CHCHHUD 0x215f48, C_CHCHDEBUG 0x216098,
C_CHCHCOORDS 0x216100 — UNREACHED.** Checkbox toggles (msg 0x4b = read the widget via
`__Menu_SendMessage(control,0x40)` into the cheat global; msg 0x51 = reply 0x2e) dispatched by
`Handler_HandleMessage` 0x2029e0 on control ids **0x10000019 / 0x10000015 / 0x10000016 /
0x100000e7 / 0x100000e8 / 0x1000001c / 0x10000017 / 0x1000001a** with no gate [M]. Their toggle
targets: FLY → `debug_scanmode` 0x30cb5d (+ `debug_clipmode` 0x30cb5e, write-only) with
`Player_ChangeSubState(glb_players, on?5:0)`; DRONES → `switch_NO_DRONES` 0x30ce2c = !checkbox;
BLIND → `switch_BLIND_DRONES` 0x30ce30; WEAP → `CheatInfo+8` 0x2a3740 and `CheatInfo+4` 0x2a373c
(all weapons @999 + armour 50.0 via `Player_EquipWeapon`); HEALTH → `CheatInfo` 0x2a3738 (god);
HUD → `hud_state` 0x30d2a4 → PlayerSetting/MPSettings HUD bytes + `HUD_Enable(pane)` 0..0x15;
DEBUG → `Drone_DebugSpr` 0x30ca9c; COORDS → `switch_SHOW_COORDS` 0x30ce7c [M — the effect readers
are shipped gameplay code: Player_HandlePain 0x1902c8, Player_InitAmmoWeapons 0x1a17f8,
Player_Update 0x193600, the NDrone2 combat states, HUD_UpdateHealthPane 0x19b9c8]. Nothing in code
sends those ids (each occurs in exactly one decompile file — the dispatcher); the only senders can
be menu-def CONTROL/KMSG data, and their only owner page in shipped data is the never-opened
per-level hub **0x40000007** (see C_LBPMMAP above — no code reference, no navigation) [D].
Recorded unreached.
## unported-front-end-menus-6 (2026-10-21) — the debug-toggle handlers, the orphan FMV page and the SDK/libc set

### The seven `C_CHCH*` debug-toggle handlers — their controls never shipped

`C_CHCHALLOWFREEZE_Handler` 0x216258 (`switch_AllowFreeze`), `C_CHCHDUMMY_Handler` 0x2162b8
(`Drone_DummyPlayer`), `C_CHCHDRAWALL_Handler` 0x216320 (`switch_ForceDrawAll`),
`C_CHCHLOCKUP_Handler` 0x216400 (`bDebugLockup`), `C_CHCHBRIGHT_Handler` 0x216460
(`bTweakBrightness` 0x30d2b4), `C_CHCHUNLOCK_Handler` 0x2164c0 (`menu_unlock_everything`
0x30d2a7), `C_CHCHCONTROLS_Handler` 0x216520 (`bDebugControls`) are each a two-way checkbox bind
(msg 0x4b read / 0x2e write [M the seven bodies]) dispatched by `Handler_HandleMessage` 0x2029e0
on control ids 0x100000c7 / 0x100000bd / 0x10000101 / 0x10000169 / 0x100001f7 / 0x10000216 /
0x1000021d [M 0x2029e0 — the seven dispatch arms; the binary's jal scan shows no other caller].

The shipped front end never contains those controls: the front-end def (07000048's kind-8
08000002) and every level archive's kind-8 08000001 were parsed and each of the eight ids —
these seven plus the keypad's 0x100000ba — has **zero hits** outside the keypad's own in-game
page (see the keypad census in docs/systems/keypad-menu.md) [D
`crates/nf-formats/tests/front_end_menus_census.rs::front_end_def_has_no_debug_toggle_controls_or_tweaks_pages`].
Their pages don't ship either: `P_TWEAKS` 0x40000044 and `P_TWEAKS2` 0x40000046 — the only pages
the decompile shows carrying debug toggles — are **absent from the shipped front-end def** [D the
same census asserts both]. The bound globals are otherwise known to the port where the game reads
them: `menu_unlock_everything` = the `--unlock-all` dev flag [M 0x20b8d8:18],
`switch_AllowFreeze` in the freeze gate (docs/port-map/core-loop.md:19), `switch_ForceDrawAll`
the portal bypass (docs/atlas/cel.md:107), `bTweakBrightness` (docs/atlas/gs-leaf.md:121). The
`%.2f` health-pane sprintf that prints coordinates is gated on `switch_SHOW_COORDS` [M
0x19b9c8:266-273, written only by the same debug family] — so even printf's float path never
runs from it.

### `P_FMV_Handler` 0x2169b0 — an orphan page left in the def

Page 0x4000004d EXISTS in the shipped front-end def with 2 controls [D census], and the 8-byte
handler (`return 1` [M 002169b0]) is the dispatch arm for it [M 0x2029e0:547-550; jal 0x2033fc
the only caller]. But nothing navigates there: no `Manager_SendMessage(…, 0x44, 0x4000004d, …)`
anywhere in the decompile [M grep + jal scan], and no mgr-msg, control-msg or keyframe message in
the whole def targets the id [D `front_end_menus_census.rs::fmv_page_is_orphaned_in_the_front_end_def`].
Unreached in the shipped game (the FMV pages that DO get wired are the trailer 0x4000004e and the
FMV player 0x40000050).

### The PS2 SDK / libc set — reached only through the layers the port replaces

These are Sony SDK/libc internals, not game logic. The port has no IOP RPC, no kernel semaphores
and no libc printf; disc access is nf-extract/iso9660 direct reads and the save flow is the
port's own. Each with its caller evidence (binary jal scan + decompile):

- `CreateSema` 0x23be20 / `PollSema` 0x23be70 — kernel semaphores; callers are the SDK file-IO
  layer (_sceFsSemInit, sceOpen/sceRead/sceLseek/sceIoctl/sceSifBindRpc/sceSifCallRpc), the CDVD
  driver (sceCdInit's cmd_sem_init, the ncmd/scmd prechecks) and the memcard library (sceMc*,
  jal 0x227dc8..0x228de8) [M]. Reached at boot and during disc/memcard IO in the real game —
  but every caller is a layer the port replaces wholesale; there is nothing to port.
- `cmd_sem_init` 0x230d58 — the CDVD driver's semaphore pair init; called by `sceCdInit` 0x2316a8
  (jal 0x231944) on the `mode < 2` arm — main 0x109080 / FS_Init 0x10bc48 call `sceCdInit(0)` at
  boot [M]. Replaced: the port reads the ISO directly (nf-formats iso9660.rs).
- `cdvd_exit` 0x230df0 — the CDVD shutdown arm; called by `sceCdInit` when `mode == 5` (jal
  0x231918), i.e. `RunNextModule` 0x120008:88's `sceCdInit(5)` at the boot module chain [M].
  Same replacement as above.
- `PowerOffCB` 0x230eb8 — the power-off callback `sceCdInit` registers and invokes inline at
  init (jal 0x23194c) [M]. Console power-event plumbing; the port has no poweroff path.
- `CB_DelayTh` 0x230b70 — the SetAlarm callback of `sceCdDelayThread` 0x230b98 (the address
  rides the alarm registration, hence no jal) [M]; reached when the loader's synchronous waits
  (`sceCdSync` from psiFileRead 0x10bdf0 / psiFileLoad 0x10ba38) block. Replaced: the port's
  reads are synchronous already (nf-extract), there is no alarm to service.
- `cvt` 0x2386a0 — the float→ASCII worker of `_vfprintf_r` (jal 0x237644) [M]. The game's only
  float-format sprintfs are the two debug health-pane overlays, gated on `switch_SHOW_COORDS`
  [M 0x19b9c8:266-273, 0x19b630:94-99], whose writer is the debug-toggle family above with no
  shipped control — so the float printf path is unreachable in the shipped game [M + D census].
  Replaced by Rust's `format!` machinery.
- `copysignf` 0x235870 — libc float helper; its only caller is `scalbnf` 0x235710 (jal 0x235824),
  a libc-internal repair path [M]. No game code path reaches it in the decompile (the game's own
  float math is inline/`PS2Sin f`-style); the port's float semantics live in ps2float.rs and std
  `f32` ops.

# unported-front-end-menus-7 (2026-10-21)

## libcglue `close` 0x23c288 (8 bytes) — newlib fd-close stub, never invoked

The 8-byte body is `jr $ra; addiu $v0, $zero, -1` [M disassembly of work/elf/ACTION.ELF
0x23c288-0x23c290; decompile 0023c288_close.c `return -1`]: it always fails and never
sets errno. Its whole-ELF reference census (every PT_LOAD segment scanned for `jal`
targets, `lui+addiu` address constructions with correct sign extension, and raw 4-byte
data words) finds **exactly one reference**: `jal 0x23c288` at 0x23bab4 inside `_close_r`
0x23ba98 [M]. From there the chain terminates in glue that nothing ever calls:

- `_close_r` 0x23ba98 (`errno=0; close(fd); on -1 && errno: *reent = errno`) is called
  only by `__sclose` 0x23b958 (`jal` at 0x23b964, args FILE->_data +0x54 / _file +0xe) [M].
- `__sclose` 0x23b958 is referenced only by `std` 0x239f30, which stores it into the
  FILE `_close` glue slot at +0x2c (lui/addiu at 0x239f48, `sw $t1, 0x2c($a0)` at
  0x239f68; the other glue slots get `__sread` 0x23b808 / `__swrite` 0x23b870 /
  `__sseek` 0x23b8f0) [M disassembly of std]. `std`'s only caller is `__sinit`
  0x239fa0:11-13, wiring stdin/stdout/stderr [M]; `__sinit` runs lazily from vfprintf
  0x237110, the FILE accessor 0x238930 and `fflush` 0x239e28 [M].
- Nothing in ACTION.ELF ever invokes the `_close` glue: `work/action-symtab.tsv` has no
  `fclose`/`_fclose_r`/`freopen`; a decompile grep for `__sclose` outside its own file
  and `std` is empty; and an instruction-level census of all 172 `lw rt, 0x2c(base)`
  sites in .text finds no same-register `jalr rt` within the following 24 instructions
  [M census, capstone scan of ACTION.ELF .text].
- The exit path closes nothing: `exit` 0x2358b0 (its 5 `jal` callers all sit in the
  failure arms of `sceMtapInit` 0x226b68) runs the atexit walk, then `(*(reent+0x3c))()`
  = `_cleanup_r` 0x239f88 (stored by `__sinit`:6 and `__smakebuf` 0x23a978), which calls
  `_fwalk` 0x23a898 with `fflush` 0x239e28 as the per-FILE callback [M _cleanup_r:4] —
  flush only, never close. So even the `sceMtapInit`-failure exit cannot reach `close`.

The game's real file I/O never uses EE-side fds: loaders go through sce* / memcard
primitives (`sceOpen` 0x23edd0, `sceMcOpen` 0x227f48, `strFileOpen` 0x108cc0,
`psiFileOpen` 0x10bd08 — symtab) [M], and the Rust port reads disc data through
nf-formats' ISO reader, which has no fd table to close. So there is no ported job to
cite either: recorded unreached (dead libcglue stub, unreachable on every on-foot
level, the front end and MP).

Method note: the scan driver's earlier "jal" hits inside the 16-byte kernel import
stubs (0x23bde4, 0x23bf54) are the PS2 syscall-trap word 0x0c000000, not calls; the
census excludes it.
