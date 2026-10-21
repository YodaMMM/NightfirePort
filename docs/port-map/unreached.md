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
