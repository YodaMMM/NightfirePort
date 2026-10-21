# The in-game keypad menu (page 0x40000021) — and the front-end never-mentioneds this task closed

Evidence tags: [M] ACTION.ELF decompile (`work/decomp/ACTION.ELF/`), [D] disc bytes, [L] EE
snapshot, [S] port choice. Task unported-front-end-menus-6 (branch `s/unported-front-end-menus-6`).

## The keypad mini-game

Nightfire's door-keypad locks open a real menu page: pressing E on a mode-0 lock without the key
(`Lock_Activate` 0x1aed00 — state 0/5, no pause manager, key-ring slot empty) sets the lock's
state to 3 and arms the flow [M 001aed00_Lock_Activate__FP7obj_tagT0.c:20-29].

- **Creation** — `Lock_Update` 0x1ae3d8 case 3 copies `KeyCodes[lock_param1]` (5 bytes) to the
  globals 0x2a37d8, sets the type cap 0x2a37e2 = 4, the count 0x2a37e3 = 0, and the typed buffer
  0x2a37dd..e0 = '*' (the display masks untyped positions), then calls
  `MenuManager_Create(0x80000003, 0x40000021, …)` [M 0x1ae3d8:176-243]. **0x80000003 is the PAGE
  TAG file-hash of the per-level defs** — every level archive's kind-8 entry 08000001 carries the
  in-game pages, and its page 0x40000021 (24 controls) exists in 07000003/05/08/09/0b/0c/0d/11/
  12/13/14/15/16/1b/23/25/26/27/28.bin [D census, `front_end_menus_census.rs`]. The front end's
  own def (07000048's 08000002) has no such page [D].
- **Key codes** — `Locks_Init` 0x1b0548 (GameFlow_Main's state-4 pop) warms the RNG with
  `GameState+0x34 · 10 & 0x1ff` draws, then fills `KeyCodes` 0x2a37a8: 51 entries × 5 bytes, each
  digit `(Rand_Random() % 9) + '0'` — '0'..'8', never '9' — plus a display flag [M
  001b0548_Locks_Init__Fv.c]. `Rand_Random` 0x1e36f0 steps SEED_X then SEED_Y and returns
  `(SEED_X << 16) | (SEED_Y & 0xffff)` [M] — the opposite step order of `Rand_Rand`.
- **The Q-Worm reveal** — `PCQWorm_Update` 0x1aee20 sprintf's the entry's four digits and sets
  `KeyCodes[i*5+4] = 1` after a PC hack [M 001aee20:28-31]; `Lock_Update` case 3 then binds the
  code text through `Txt_BindLabel(0x2000014)` + `Text_AddMsg(0xb4)` so the page shows it
  [M 0x1ae3d8:249-258]. docs/systems/decryptor.md's "the display alone" path.
- **Digit input** — each digit button is a control with id **0x100000ba** on page 0x40000021;
  `Handler_HandleMessage` 0x2029e0 routes it to `C_KEYPAD_Handler` 0x216810 [M 0x2029e0:90-94].
  Msg 0x4b: the count wraps to 0 when it reaches the cap, `typed[count] = '0' + control+0x20`,
  the display string 0x30d8c8 = typed[0..=count] NUL-terminated, and msg 0x18 (set text) fires at
  the display control **0x10000075** [M 0x216810:8-23]. Msg 0x51 rebinds + beeps (0x62) [M :24-29].
- **Match** — `Lock_Update` case 4 compares typed[i] == code[i] for all i < cap; a full buffer
  that matches runs `Lock_Unlock` (state 1, the out channel, unlock sound 0x42c80000) and deletes
  the keypad manager; a page closed without a match drops the lock back to state 0 (re-armable)
  [M 0x1ae3d8:262-296]. The per-frame `MenuManager_Enable(1, slot)` keep-alive [M :273-276] is
  the manager-pool housekeeping the port folds into the page staying open.

Port (nf-game `lock.rs`): `KeyCodes::init` ([S] private-stream digits — the real boot draws are
pre-consumed by the level-start snapshot; the shape/count/domain are [M]), `Keypad::arm/digit/
display/hint/tick/close` — the exact buffer, wrap, inclusive display and match semantics above,
unit-tested with a fixed code (`lock_keypad.rs`). The view work (main agent): draw the page's 24
controls from the level def, the digit grid bound to `Keypad::digit`, the display label to
`Keypad::display`, the Q-Worm hint line to `Keypad::hint`.

## The front-end functions this task disposed

- **The debug-toggle handlers** (`C_CHCHALLOWFREEZE/DUMMY/DRAWALL/LOCKUP/BRIGHT/UNLOCK/CONTROLS`
  0x216258/0x2162b8/0x216320/0x216400/0x216460/0x2164c0/0x216520) — two-way checkbox binds
  (msg 0x4b read / 0x2e write) for `switch_AllowFreeze`, `Drone_DummyPlayer`,
  `switch_ForceDrawAll`, `bDebugLockup`, `bTweakBrightness` 0x30d2b4,
  `menu_unlock_everything` 0x30d2a7, `bDebugControls`. All dispatch on control ids
  0x100000c7/0x100000bd/0x10000101/0x10000169/0x100001f7/0x10000216/0x1000021d via
  `Handler_HandleMessage` [M 0x2029e0]; **none of those ids exists in any shipped menu def**
  [D census over 07000048's 08000002 and every level archive's 08000001], and their pages
  (`P_TWEAKS` 0x40000044 / `P_TWEAKS2` 0x40000046) are absent from the shipped def [D].
  Unreached (docs/port-map/unreached.md). The globals themselves are known to the port where the
  game reads them: `menu_unlock_everything` = `--unlock-all` [M 0x20b8d8:18], `switch_AllowFreeze`
  in the core-loop freeze gate (docs/port-map/core-loop.md), `switch_ForceDrawAll` the portal
  bypass (docs/atlas/cel.md:107), `bTweakBrightness` (docs/atlas/gs-leaf.md).
- **`P_FMV_Handler` 0x2169b0** (8 B, `return 1`) — page 0x4000004d EXISTS in the shipped def with
  2 controls [D], but nothing reaches it: no `Manager_SendMessage(…, 0x44, 0x4000004d, …)` in the
  decompile [M grep] and no mgr/control/keyframe message in the def targets it [D census].
  Unreached (orphan page).
- **`Page_SetHelpText` 0x21f2b8** — reads the help control global `h` (set by
  `Page_FindHelpControl` 0x21f250, which `MenuManager_Create` calls per page [M 0x21b908:137]),
  matches the focused control's id against `help_table` rows {id, match, label} (0xf0f0f0f0 = the
  wildcard default) and binds the label to the help control [M 0x21f2b8]. **No callers** in the
  decompile [M grep] — the help-bar flow is dead; unreached.md. `Page_FindHelpControl` itself IS
  called, but its result is only read by SetHelpText, so the port keeps it as the parse-side
  getter `PageDef::find_help_control` (nf-formats menudef.rs) with the census test asserting the
  shipped def has no help control for it to find ([D] — see the test's sister).
- **`ConvLanguageToFlagIndex` 0x216948** — no direct callers: GCC inlined it (twice) into
  `P_LANGUAGE_Handler` 0x208830 (page 0x40000034 EXISTS in the shipped def [D]) for the flag
  spinner preselect + the 9-flag disable loop [M 0x208830:77-135]. The inverse table is
  `C_LANGUAGE_Handler` 0x208c68:8-24. Ported as `conv_language_to_flag_index` (nf-view menu.rs),
  every arm decompile-pinned.
- **`MenuManager_Init` 0x21b570 / `MenuManager_GetStatus` 0x21deb0 / `MenuManager_Enable`
  0x21def0** — the manager-pool triple: boot init (caller `ResetMap_GameInit` 0x1d5080), the
  busy-status max (callers `Menu_InputAction` 0x1f1f40:102, `P_PAUSE_Handler` 0x203c28:99), and
  the per-slot enable (caller `Lock_Update` 0x1ae3d8:273-276, the keypad keep-alive). The port
  folds the 5-slot pool into one GameFlow/FrontEnd: citations at `GameFlow::boot` and
  `FrontEnd::action` (nf-view menu.rs); the Enable side lives in the Keypad doc comment
  (nf-game lock.rs).
- **`Process_Init` 0x21f780 / `Process_Destroy` 0x21f860** — the delayed-process list on the
  control (+0x5c): init in `Script_PrepareScripting` 0x220ed8, freed in `Script_DelAllScripts`
  0x220f30 (+ the MP start/finish handlers). The port's script runtime models the list as the
  parse-side `Proc` records (nf-formats menudef.rs:522-551, cited there).
- **The PS2 SDK/libc set** (`CB_DelayTh` 0x230b70, `cmd_sem_init` 0x230d58, `cdvd_exit` 0x230df0,
  `PowerOffCB` 0x230eb8, `copysignf` 0x235870, `cvt` 0x2386a0, `CreateSema` 0x23be20, `PollSema`
  0x23be70) — reached only through the disc/memcard/runtime layers the port replaces
  wholesale (nf-extract/iso9660 direct reads, no IOP RPC, no semaphores, Rust `format!`/`f32`
  math). Where each sits: docs/port-map/unreached.md (SDK section), with the caller evidence.
