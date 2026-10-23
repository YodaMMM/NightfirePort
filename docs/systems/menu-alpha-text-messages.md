# Menu alpha pulse, text formats and the delayed-message queue

The front end's per-frame text plumbing: the focus alpha pulse (`Menu_Alpha*`), the text-format
phase (`Menu_UpdateTextFmt` / `Menu_GetNormalTextFmt`), the delayed-message queue
(`Menu_ClearDelayedMessages`), the retail string-pool stubs (`Menu_StringInit` / `Menu_StringKill`),
the MP ready fade (`Menu_MPFadeWhenReady`), the codename-list highlight sync
(`Menu_SelectCodenameInControl`), and the boot language flow (`C_LANGUAGE_Handler`).
All [M] = ACTION.ELF decompile + disassembly (`work/decomp/ACTION.ELF/`,
`tools/crosscheck/mipsdis.py`); [D] = disc bytes; [S] = port choice.

## The alpha block (globals, all zero-init .sbss)

| global | addr | type | accessors |
|---|---|---|---|
| `alpha` | 0x30d17c | s16 | written by `Menu_AlphaUpdate` 0x1f3170 (`sh` 0x1f31d4), read by `Menu_AlphaGet` (`lh` 0x1f31ec) — no others |
| `counter` | 0x30d180 | f32 | `Menu_AlphaUpdate` (lwc1/swc1 0x1f3170..0x1f31b8) + `Menu_AlphaReset` (`sw zero` 0x1f31e4) |
| `menu_counter` | 0x30d178 | u32 | zero accessors — dead |
| `throb` | 0x30d188 | f32 | one write (`swc1` 0x1f3388, `Menu_UpdateTextFmt`) — **no reader in the binary** |
| `count` | 0x30d18c | f32 | `Menu_UpdateTextFmt` only (4 hits) |

`Menu_AlphaUpdate` 0x1f3170: `counter += 0.2`, wrap at 6.2831855, `alpha = (s16)(sin(counter) * 64.0)`
— called from `MenuManager_Update` 0x21c3d8 (lines 35/54; the second pair runs when
`PS2FramesToSkip == 2`).

- **`Menu_AlphaReset` 0x1f31e0** — `counter = 0.0f` (alpha itself untouched; the next update lands
  on sin(0)·64 = 0). Callers: `Page_Update` 0x21edc8 (jal 0x21ee58) when the page's focused control
  changes and `mgr+0x1d2 != 0` (the per-row-move flash restart); `Component_SetupInstance` 0x1f0950
  (jal 0x1f0e1c) when the setup flags carry 0xc0.
- **`Menu_AlphaGet` 0x1f31e8** — `return alpha`. The four call sites add it byte-wise into a colour
  word: `Component_SetupInstance` 0x1f0dc4 (instance colour, no fractional scale, flags bit 0x20),
  `Label_Update` 0x218cb8 0x218e68 (base `label+0x104` → `label+0xb4`, the flashing-label pulse),
  `List_Update` 0x21a598 0x21b06c (every row's `row+0x10` when `listbox+0x568` is 1..2),
  `MenuManager_Update` 0x21c64c (`*(mgr+0x94)+0x10 = 0x201f3400 | (alpha + 0x7f)`).

Port: `FrontEnd::alpha_phase` (counter) + `FrontEnd::alpha` (the s16);
`FrontEnd::menu_alpha_tick()` = the update, `menu_alpha_reset()` = counter 0 (wired into
`focus_moved`, the Page_Update focus-change analog), `menu_alpha_get()` = the getter. The byte-wise
colour application is draw-side.

## Text formats

- **`Menu_UpdateTextFmt` 0x1f3328** — `count += 0.1` (wrap 2π), `throb = sin(count) * 0.03125 + 1.0`.
  `throb` has **zero readers** (`Menu_GetNormalTextFmt` writes 1.0 constants directly;
  `Label_Update`'s `Font_GetTextExtent` 0x218f30-area hardcodes 1.0) — the scale-throb was cut in
  retail; the function stays on the tick as a dead per-frame update. Callers: `MenuManager_Update`
  0x21c444/0x21c4e0.
- **`Menu_GetNormalTextFmt` 0x1f3398** — `if (sprite) { sprite+0x40 = 1.0f; sprite+0x3c = 1.0f; }`
  — the M_LABEL text sprite's x/y scales (sprite = `label+0xa4`). Callers: `Label_Update`
  0x218e48 (focus path) + 0x218f30 (un-focus path).

Port: `FrontEnd::text_fmt_count` / `menu_update_text_fmt()` (keeps the phase; the throb value is
kept in `text_throb` for parity and documented write-only) and `menu_get_normal_text_fmt()` →
`(1.0, 1.0)` for the label draw layer.

## The delayed-message queue

128 slots at 0x2ddb00, 20 B each: `{M_CONTROL* target +0x0; u32 when +0x4; u32 msg +0x8; u32 a
+0xc; u32 b +0x10}`; the clock `menu_delay_frame` 0x30d190 only ever increments
(`Menu_ProcessDelayedMessages` 0x1f3490 fires entries whose `when == frame`). Allocation
(`__Menu_SendDelayed` 0x1f24e0 / `__Menu_SendDelayedMessage` 0x33c0): slot 0 if `when < frame`,
else the first free slot in 1..127, else fail.

- **`Menu_ClearDelayedMessages` 0x1f3508** — zeroes only the `when` field of all 128 slots;
  zeroed `when`s never re-fire, so this drops everything pending. Callers: `Manager_SendMessage`
  0x21c858 case **0x44** (change page: old page marked dying, `+0x1c0 = old`, **Clear**,
  `+0x1bc = new` — skipped on the `param_4 & 2` keep-page variant) and `PS2_DoCardFlow` 0x217bd8
  state **8** (card-dialog teardown drops queued UI msgs, then `state[2] = 0`).

Port: `FrontEnd::menu_clear_delayed_messages()` drops the port's pending delayed page/block model
(the 128-slot queue's analog is `delayed_page`/`delayed_block`); wired into the card-flow
completion (`flow_done`) per the state-8 caller.

## The string-pool stubs

- **`Menu_StringInit` 0x1f3540** and **`Menu_StringKill` 0x1f3548** are literally `jr ra; nop`
  [M disasm] — retail stubs (Init's `int` arg is ignored; `MenuManager_Create` 0x21b988 passes
  32000). Their companion globals (`string_count.453` 0x30d524, `menu_counter` 0x30d178) have zero
  references. Port: `menu_string_init()` / `menu_string_kill()` no-ops with the citations.

## `Menu_MPFadeWhenReady` 0x201fc0

Builds `M_MESSAGE {id 0xe1, a 0x60606080, b 0}` and `Process_Create(control, msg, 0x1e, 0)` —
msg 0xe1 is a per-control colour-fade process (`Process_Create` 0x21f7a8 snags the current colour
via msg 0x30; `Process_RunFrame` 0x21f380 lerps each RGBA byte toward the target over the duration,
sending msg 0x1a per frame): fade the control to 0x60606080 over **30 frames**. Caller:
`C_RBMPSTART_Handler` 0x20d048 msg **0x4b** (✕) when `mp_stuff+pad*4+0x360 == 5`: binds Txt 0x39c
"Player Ready", sets `mpjoin+pad*0x10+1 = 1`, fades the panel; the un-ready arms (0x5c/0x6b)
restore 0xfa6e5aff. Page: P_MPJOIN 0x40000019.

Port: the ready state is `MpJoinAgent.ready` (the same job under another name — cited at the ready
sites); the 30-frame fade itself is control-layer cosmetics left to the draw pass.

## `Menu_SelectCodenameInControl` 0x2028d8

Walks `codename_buf` 0x2e1c58 (32-byte entries, count 0x2e1ad4 — 2 defaults then saves) with
`strcmp` against the active codename name at 0x2e1ae8 (written by `C_SBCNSELECT` 0x212f58:61,91 /
`C_KEYBOARD` 0x213a98:54 / `Menu_MapDefaultCodename` 0x1f7fa8:82), then
`Menu_Send(pad, control, 0x2e "set list selection", index, 0)`: index = the match, `count` (no row)
when the name isn't listed, 0 when the name is empty or count is 0. Callers: `P_NFSELECT_Handler`
0x20aec0:44 and `P_CNSELECT_Handler` 0x212ad0:42, both in the page-open msg **0x4c** arm, target
control 0x100000eb — the select-codename pages open with the active profile's row highlighted.

Port: `FrontEnd::select_codename_in_control()` — on `Page::Codename` / `Page::CnSelect` open,
cursor = the active name's row (slot 0/1 are the defaults, saves follow; miss = rows count, empty
name = 0).

## `C_LANGUAGE_Handler` 0x208c68 (boot language flags)

Dispatches on control **0x1000015e** on the P_LANGUAGE boot page **0x40000034** (`bootup` 0x1c7e78:24
opens it every boot; `bDisplayLangMenu` 0x30d2a8 boots 1 in .data, so the flag page shows on a
normal boot). Msg **0x4e** (focus): `LanguageToLoad` 0x30d2ac = {0,2,1,4,5,3,8} indexed by the
control's value (tLANGUAGE 0=EN 1=FR 2=DE 3=ES 4=IT 5=NL 8=SV) + the "~A to select" prompt from the
per-language strings 0x308f38.. (EN/DE/FR/IT/NL/ES/SV). Msg **0x4b** (accept): the same map →
`Txt_SetLanguage(LanguageToLoad)`, `SFXSetLanguage` (VO: 2=DE 1=FR 3=ES else EN), then
`Manager_SendMessage(0x44, 0x40000048, 1)` — the memcard-init page.

Port: `LANGUAGE_TO_LOAD` + `language_flag_to_load()` + `language_sfx_vo()` +
`language_accept_next_page()` (constant 0x40000048). The page itself stays unmodelled in the
boot walk (FrontEnd::new opens the title) — wiring it in is a boot-flow decision.

## `C_RBMPFINISH_Handler` 0x20db48 (MP setup FINISH)

Dispatches on control **0x1000022a** — the FINISH control on P_MPSETUP **0x40000051**, activated by
`C_RBMPSETUP` 0x20de58:114 (`Manager_SendMessage 0x5a`) when the setup steps to state 5. Msg
**0x60** (focus lost): `mpjoin[p]+1 (ready) = 0`. Msg **0x5c** (focus gained): hide self, colour
0xfa6e5aff, show prompt 0x100001a2 / list 0x100001a1 / plate 0x100001a0; at
`mp_stuff+0x360+p*4 == 5` bind label 0x1e9 "Press ~A when ready". Msg **0x6b** (back, from
Manager case 0x5f's pop): at state 5 re-activates the handicap list (0x5a on 0x100001a1), steps
`mp_stuff+0x360+p*4` 5→4, rebuilds the handicap combo (strings 0x30d390.. = −75..100), reselects
`MPSettings+p*0x30+0x2c`, and writes **0xfffffffe through param_6 to VETO the pop**
(checked 0x21c858:573-580). It never handles 0x4b — ready/launch is P_MPSETUP's tick
(`Menu_MPAreWeReady`).

Port: the ready/launch flow is `mp_setup_state` + `MpJoinAgent.ready` (same job — cited at the
MpSetup site); the focus-pair (0x5c/0x60) and the state-5 back-veto are control-layer details
documented here, unmodelled.

## Citations for the replaced functions

- **`Menu_FlushEnum` 0x202000** (`enum_state = 0`, resets `Menu_EnumSaves` 0x1f7b78's enumeration
  machine; called from `PS2_DoCardFlow` 0x2184f0's terminal block): the port's profile load is a
  synchronous directory read (`FrontEnd::reload_profiles`), no persistent enumeration state —
  cited there.
- **`Menu_UpdateOptionBox` 0x202280** (returns + zeroes `option_return`, written by
  `C_LBMSGOPTIONS_Handler` 0x2022a0: 0x4b → selected row, 0x6d → 2 or 4; called from the msg-0x50
  "modal closed" arms of 7 page handlers): the port resolves the option box's pick directly —
  cited at the port's msg-0x50 sites.
- **`P_FMVTEST_Handler` 0x20a968** is UNREACHED (debug FMV island —
  docs/port-map/unreached.md); its job (browse/play a movie by hash) is the port's
  `nf-view` movie harness, cited in movie.rs.
