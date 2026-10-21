# Menu engine — control/component/sprite helpers (task unported-front-end-menus-4)

The front end's runtime menu engine under the parsed page data: per-control rects, the
component sprite-set layout, cursor hit-testing and directional focus movement, the combo
widget and the allocator/menu-sprite helpers. All facts are [M] — ACTION.ELF decompile
under `work/decomp/ACTION.ELF/`, cross-checked against the disassembly where Ghidra dropped
arguments (noted inline). The port lives in `crates/nf-view/src/menu_engine.rs`
(data/state/flow only — the draw code is `menu_overlay`'s, which the main agent renders).

## Dispositions (the 25 never-mentioned functions)

| Address | Function | Outcome |
|---|---|---|
| 0x1f0248 | Combo_Init | ported — `Combo::init` |
| 0x1f0868 | Combo_SendMessage | ported — `Combo::send_message` |
| 0x1f0950 | Component_SetupInstance | ported — `component_setup_instance` |
| 0x1f1038 | Component_InitComponents | ported — `component_init_components` |
| 0x1f10e8 | Component_DestroyAll | ported — `component_destroy_all` |
| 0x1f1198 | Component_AddComponent | ported — `component_add_component` |
| 0x1f1200 | Component_InitInstance | ported — `component_init_instance` |
| 0x1f1268 | Component_GetHandle | ported — `component_get_handle` |
| 0x1f12e0 | Component_Free | ported — `component_free` |
| 0x1f1368 | Component_Destroy | ported — `component_destroy` |
| 0x1f13c8 | Menu_CursorOverMe | ported — `menu_cursor_over_me` |
| 0x1f16d8 | Menu_FindControl | ported — `menu_find_control` |
| 0x1f2708 | Menu_GetFirstControl | ported — `menu_get_first_control` |
| 0x1f27f8 | Menu_GetPage | ported — `menu_get_page` |
| 0x1f2870 | Menu_InitializeSprite | ported — `menu_initialize_sprite` |
| 0x1f2e30 | Menu_ResizeControl (x,y,w,h) | ported — `menu_resize_control` |
| 0x1f2e48 | Menu_ResizeControl (copy) | ported — `menu_resize_control_copy` |
| 0x1f2e70 | Menu_ClipString | ported — `menu_clip_string` (measurer injected) |
| 0x1f2f18 | Menu_Assert | ported — `menu_assert` |
| 0x1f2fa8 | Menu_CreateSprite | ported — `menu_create_sprite` |
| 0x1f2fe8 | Menu_DeleteSprite | ported — `menu_delete_sprite` |
| 0x1f3008 | Menu_Malloc | cited — tagged-alloc job = the port's Vec allocations (menudef) |
| 0x1f3028 | Menu_Free | cited — same |
| 0x1f3048 | Menu_SetAction | ported — `menu_set_action_targets` |
| 0x1f30d8 | Menu_GetLastController | ported — `menu_get_last_controller` (the game returns 0) |

## Evidence

- **Control runtime rect** +0x70..+0x76 = x, y, w, h shorts; +0x78 layer short; +0x7a
  disabled byte; +0x7b type byte [M every function below; the rect fields are what
  `Menu_FindControl`'s scoring reads — disasm 0x1f1768-0x1f1798].
- **Control types**: 3 = combo (`Combo_Init` 0x1f0248 stores 3 to +0x7b — disasm
  0x1f0280 `sb $v0, 0x7b($s0)`), 8 = the page container (`Menu_GetPage` walks to it
  [M 0x1f27f8]), 0xd (13) = the page header control (`Menu_GetFirstControl`/`Menu_FindControl`
  skip it in walks [M 0x1f2708/0x1f16d8]).
- **Menu_Assert 0x1f2f18**: returns 1 iff ctrl != 0, ctrl+0xc (parent) != 0, and
  type(+0x7b) ∈ {1, 2, 3, 5, 6, 9, 10, 11, 12, 200, 114}. The gate is UNSIGNED
  (`sltiu (t-1) < 3`, disasm 0x1f2f40-0x1f2f48) so t=0 fails — the decompiled C's signed
  `(2 < bVar1-1)` would wrongly admit 0.
- **Menu_GetPage 0x1f27f8**: assert-gated (a page header's own type 0xd fails the assert →
  returns 0); type 0xd would return +0x94 (the page's control list); otherwise walks the
  +0xc parent chain to a type-8 control and returns it, 0 at the end.
- **Menu_GetFirstControl 0x1f2708**: the page's current control (page+0xac) when set and its
  +0x18 is not 0x1fffffff [M disasm 0x1f2720-0x1f2744]; else walk from the list head, skipping
  type-0xd headers and +0x7a-disabled controls; when none, a static backup control:
  name 0x1fffffff, x=y=-100 (0xff9c), w=h=0, byte +0x6b = 0xff [M 0x1f2708:8-14].
- **Menu_CursorOverMe 0x1f13c8**: page-space point = cursor x/y (manager+0x94 sprite +0x24/+0x26)
  minus the page rect (+0x70/+0x72); walk the page's controls, skip 0x7a-disabled and 0xd
  headers, keep those whose rect (+0x70..+0x76) contains the point; among the hits the one
  with the LOWEST +0x78 layer wins (the loop keeps the min in slot 0); returns whether that
  topmost hit is the passed control.
- **Menu_FindControl 0x1f16d8** (directional focus move; disasm-verified):
  directions 1=UP, 2=DOWN, 4=LEFT, 8=RIGHT (arg &0xff; other values score nothing).
  From the current control, walk the page's controls (skip current, disabled, type-0xd,
  w·h == 0; ≤63 candidates per array — `sltiu $v0, $s3, 0x3f` 0x1f17a0). Candidate gates:
  the candidate's rect must overlap the current one on the OTHER axis (bVar4 = x-overlap
  for vertical moves, bVar5 = y-overlap for horizontal), plus a one-sided gate on the move
  axis. Scores: vertical = 2·|Δy| + 0.1·|Δx| (disasm 0x1f18d8-0x1f191c), horizontal =
  |Δx| + |Δy| (0x1f1a04-0x1f1a5c). Same-direction candidates land in array 1, opposite
  (wrap-around) ones in array 2, pre-seeded with the current control
  (`sw $s0, 0x200($sp)` 0x1f1748). `QuickSort(..., 7)` 0x1d30a8 with comparator id 7 =
  `CompMenuCtrl` 0x1d34c8 (three-way sign compare of the signed short at elem+4 —
  ascending, min first; ties unspecified). Winner = sorted array1[0]; when array1 is empty
  (its seed — the current control — survives), sort array2 and take array2[0]; if array2
  is empty too, the current control (no move). The cursor sprite (manager+0x94) is then
  repositioned: x/y = page rect + control x/y + w/2 / h/2; +0x2c/+0x2e = w+10 / h+10;
  +0x38 = control+0x78 (layer) + 1 [M 0x1f1b74-0x1f1bf4].
- **Component set records** [M 0x1f1198/0x1f1268/0x1f1368]: record = {+0 hash u32, +4 count
  i16, +6/+8/+0xa/+0xc = a,b,c,d u16s, +0x10 = malloc(count · 0x38) instances} —
  `Component_AddComponent` writes exactly those fields; count 0 leaves the array null.
  13 record slots per set [M `Component_Destroy` 0x1f1368 walks 0xd]. The set node carries
  a set id at node+0x120 and a page byte at node+0x124 [M `Component_GetHandle` node+0x48
  int-words, `Component_DestroyAll` node+0x49 int-words byte]. Handle = (set id << 16) |
  record index [M 0x1f1268: match node id == handle>>0x10, record = node+8+index·0x14;
  `Combo_Init` builds `(u16 from NEW_CONTROL+0x14) << 16 | 2` — disasm 0x1f0290].
- **Component_InitInstance 0x1f1200** fills a 0x38-byte instance: +0 hash u32 (param 13),
  +8 = 0xffff, +0xa = 0, dx/dy/w/h shorts at +0xc..+0x12, four more u16s +0x14..+0x1a,
  +0x1c = 0xffffffff, +0x20/+0x24 = 0, byte +0x28 = 0, wf/hf f32 at +0x2c/+0x30,
  flags u32 at +0x34. The parse's SPRITE −0x05 record {8×u16, u32, u32, f32, f32}
  (menudef.rs) feeds those in order.
- **Component_SetupInstance 0x1f0950** — the layout math (all [M], expressions quoted from
  the decompile): params (mask u32, x s16, y s16, w int, h int) with -1 → the control's own
  rect fields; x/y here are RELATIVE to the page (final x = page.x + x + offset). Per
  instance whose flags & mask != 0 (else its sprite's layer byte = 0xff = hidden and it is
  skipped): width w' = (wf == 0 ? inst.w : clamp(wf · (span), 0, span)) where span = the
  horizontal accumulation (right edge seeded with w; fixed instances push it down at
  dy+w unless flag 0x400; right-anchored (flag 2) instances pull the right edge to
  (w - dx) - w'); height likewise on the vertical span with hf / flag 0x800 / flag 1.
  Flags 0x100: inst.w slot = hf · (vertical span), or the control sprite's current +0x2e
  when hf == 0; 0x200: inst.h slot = wf · (horizontal span), or the +0x10 slot when wf == 0.
  Both slot writes land in the record and persist across calls. Placement: flag 2 (right) x = page.x + x + w - w' - dx;
  flag 4 (centre-x) x = page.x + x + (w - w')/2 — dx ignored [M the (iStack_d4 - lVar11)>>1
  branch]; flag 0x20-less default x = page.x + x + dx; flag 1 (bottom) / 8 (centre-y) on y
  symmetric. Alpha: when mask & 0x20 == 0 or wf/hf != 0 → Menu_AlphaReset only if
  mask & 0xc0 (else nothing); otherwise (0x20 set, no fractions) → Menu_AlphaGet. Finally `Menu_InitializeSprite(sprite, ctrl.layer - page.layer, x, y, w', h',
  0, record.hash)`.
- **Menu_InitializeSprite 0x1f2870**: sprite.layer(+0x38) = param; x/y/w/h to +0x24/+0x26/+0x2c/+0x2e;
  byte +0x39 = 7 always; +0x1c = 0; +0x22 = param 15; UV quad +0x30..+0x36 written only when
  param 11 != -1; text ptr +0x18 / font +0x1c = &DAT_0030d170 / len +0x20 = strlen when text != 0;
  colour +0x10 = param 9, or 0x7f7f7fc0 when 0.
- **Menu_CreateSprite 0x1f2fa8**: null info → `Sprite_Create()`; else byte +0x28 = 7
  (HARDCODED — `addiu $v0, $zero, 7` 0x1f2fc8 in the delay slot; the MENUSPRITETYPE second
  argument is dead in this build) then `Sprite_Create2(info)`.
- **Menu_DeleteSprite 0x1f2fe8**: null-guarded `Sprite_Delete`.
- **Menu_ResizeControl 0x1f2e30 / 0x1f2e48**: store x/y/w/h into +0x70..+0x76; the 2-arg
  form copies those four shorts from another control.
- **Menu_ClipString 0x1f2e70**: sprite+0x18 (text) != 0 → count = Font_GetTextCountExtent(width
  limit), sprite+0x20 = count; extent → sprite+0x34 w / +0x36 h; returns the width; 0 when
  no text. The port injects the measurer (the port's `Font` metrics live in nf-formats::hud;
  the game's `Font_GetTextCountExtent`/`Font_GetTextExtent` are not separately ported).
- **Combo_Init 0x1f0248** (3 args: self, NEW_CONTROL*, page — disasm prologue): type = 3;
  handle = (u16 ctrl+0x14) << 16 | 2; copies ctrl+0x00 → +0x18, ctrl+0x0c → +0x20; layer =
  u16 ctrl+0x16 → +0x78; rect x/y/w from ctrl +0x18/+0x1a/+0x1c, h = (int)item width + 5
  (written last, :109); item width
  +0x714 = Font_GetTextExtent(0x305c60 charset) + 2.0; item count - 1 → +0x718 (count from
  ctrl h ÷ control's own +0x5c4 row height); a LABEL child at +0x600 (Label_Init with a copy
  of the NEW_CONTROL: x += comp+6, layer − 1, h = the combo's +0x76 before its :109 write;
  msg 0x2b) and a LIST child at +0x94 (List_Init: x += comp+6, y += (int)item width +
  comp+8 + 4, w −= comp+6 − comp+10, h = 1, layer − 1) [M 0x1f0248:84-105]; then
  Script_PrepareScripting.
- **Combo_SendMessage 0x1f0868** switch: 0x14 → Script_ExitLoop (returns its result);
  0x33 → List_SendMessage(list child, 0x33); 0x39 → returns the combo itself (jump table 0x305ca0 → 0x1f0938); 0x41 → return +0x718
  (item count - 1); 0x47 (destroy) → label/list 0x47, Component_Free, Script_DelAllScripts,
  Mem_Free(self), return 0; 0x49/0x4b → forward to `__Menu_SendMessage(page, msg, self, 0)`.
- **Menu_SetAction 0x1f3048**: slot -1 broadcasts `Input_SetAction` to pads 0..3, else only
  that pad. (Reached from MenuManager_Delete.)
- **Menu_GetLastController 0x1f30d8**: `return 0` — the whole body [M]. (Callers:
  Menu_RunMiniMission 0x1e2??, C_SBNFMAP_Handler.)
- **Menu_Malloc 0x1f3008 / Menu_Free 0x1f3028**: `Mem_Malloc(size, 0x3104, 0)` /
  `Mem_Free(ptr)` — thin tag-0x3104 wrappers. The port has no heap model; its job (tagged
  menu allocations) is the parsed/runtime `Vec` storage in nf-formats::menudef + the
  instances Vec here — cited at those sites.

## Where the port's existing code does the same job

- Focus movement: `FrontEnd::action()` (menu.rs) models each page's cursor moves by hand;
  the generic scorer is now ported as `menu_find_control`.
- Cursor hit-test: the port's mouse path (docs/reports/menu-mouse-parity.md) feeds
  page-space points; `menu_cursor_over_me` is the game's own test.
- Combo wheels: the port's `wheel`/`wheel_at` (menu.rs) draw the game's combo pages;
  `Combo` here is the widget's own data/flow.
- Component parse: `SpriteSet`/`ComponentDef`/`SpriteDef` in nf-formats/src/menudef.rs hold
  the −0x03/−0x04/−0x05 tag data `Component_AddComponent`/`InitInstance` consume.
