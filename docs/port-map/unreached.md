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
