# The M_LISTBOX widget (the menu List family) — data, state, flow

2026-10-21. All six functions are REACHED in the
shipped game; the port had **no List runtime at all** (its FE hand-rolls the on-screen
lists). This session ports the data/state/flow into `crates/nf-formats/src/menudef.rs`
(`ListRow`, `ListState`); the DRAWING is separate.

## The byte that classifies a control (corrects an earlier census)

`MenuManager_Create` 0x21b908 parses CONTROL records `{u32 id, u8 type, u8 class,
x, y, w, h, …}`. TWO different bytes drive two different gates:

- **type** (first byte after the id) = the widget kind; `Manager_SendMessage` 0x21c858
  case 0x42 dispatches on `NEW_CONTROL+0x21` = this byte [M 0x21c858:159]: 1→Button,
  2→Checkbox, 3→Combo, 5→Label, **6→List (malloc 0x56c → List_Init 0x219488**,
  :196-202), 9→Radio, 0xa→Scroll, 0xb→Spin, 0xc→Text, 0xd→Window, 200→Memo; types 0
  and 4 return 0.
- **class** (second byte) = the instantiation gate at MenuManager_Create 0x21b908:315
  (only class ∈ {0, 3} gets a Manager_SendMessage 0x42).

The port's `ControlDef` fields are named `b` (first byte = the widget **type**) and
`ctype` (second byte = the **class**) — the names read backwards vs the game, and the
older census claim "zero ctype-4 List controls in the FE" had tallied the **class**
byte. Rescan of the disc [D, this session]: across all 34 kind-8 menu defs in
FILES.BIN's level archives the widget-type tally is {1:713, 2:594, 5:3783, **6:335**,
9:48, 10:1139, 200:67} (no 0s, no 4s). The **335 type-6 List controls** include:

- FE def 07000048.bin/08000002: **0x10000120** (35,296,570,92 — the option-box row
  list) and **0x100000fc** (50,296,540,92 — the memory-card dialog list) [D];
- the in-game menu def 08000001 in every level archive: 0x10000028 (94,185,465,153 —
  the pause rows list), 0x10000010, 0x1000003e, 0x10000120, 0x100001a4, 0x1000021c,
  ~9-10 records per archive [D census];
- and `Combo_Init` 0x1f0248:105 embeds an M_LISTBOX at M_COMBO+0x94 — the disc's 4
  class-3 combos each drive one every focused frame (`Combo_Update` 0x1f0498:113).

`Menu_CreateOptionBox` 0x1f9608 and `PS2_DoCardFlow` 0x217bd8 do NOT build lists —
they open the authored pages (0x4000002f / 0x4000002a) and poke the existing list
controls (row-text msg 0x10, then a direct `List_Update` :68 / :246).

## List_Init 0x219488 — the state the port models

`List_Init(M_LISTBOX* box, NEW_CONTROL* def, M_CONTROL* parent)` [M 0x219488]:

- type byte +0x7b = 6 (:61); rect def+0x18..+0x1e → +0x70..+0x76 (:114-117);
  def+0x16 (font size) → +0x78 and +0x10c = that −1 (:73,:76).
- **row stride +0x530 = Font_GetTextExtent(height) × 1.1** (:58,:75); **visible rows
  +0x500 = h / stride** (:79); scroll top +0x534 = 0 (:66).
- **1 column** (+0x504), column widths +0x508 = {100,0,0,0,0} (percents),
  per-column alignment +0x51c = 1 (left; 2/3 = right/centre, consumed by Update
  :404-415) (:92-94).
- colours: normal +0x53c = 0x404040ff, highlight +0x540 = 0x7f7f7fff, +0x538 = 0xff,
  +0x544 = 0x80 (:85-89); striped-rows flag +0x548 = 0 (:82); highlight mode +0x564 =
  def+0x04 (:80,:88); alpha flag +0x568 (:81).
- **row list** = embedded LINKEDLIST at +0x270 (`LinkedList_Init` 0x219280:90); rows
  are 0x2d0 mallocs: `LIST_ROW = {+0x0 next, +8.. text cells (5 × 0x8c stride),
  +0x2c4 id/value, +0x2c8 wrap-flag byte, +0x2cc stripe sprite}` [M
  List_SendMessage 0x2196e8:89,126,293]. Cursor row ptr +0x560 (empty at init),
  visible index +0x56b.
- embedded cursor-bar M_CONTROL at +0x94 (type 0x72, :106-111) and an embedded
  M_SCROLL at +0x128, width 0x10, docked x+w−0x10 (:95-105).

## The four index helpers (exact walks, all [M])

- `List_GetIndex` 0x21b420: walk `row->next` from the first row, i = 0; return i when
  the node == the target; −1 when the walk runs off the end. (Arg 2, the LINKEDLIST*,
  is never read — callers pass box+0x270 redundantly.)
- `List_GetIndex_RealItem` 0x21b470: the same walk, but the counter bumps **after
  advancing**, only when the row just advanced TO has `+0x2c8 == 0` (non-wrap) — so
  the returned value is the 0-based index among non-wrap rows, and a **wrap row
  returns the preceding real item's index**. The wrap flag is set by
  `List_SendMessage` arm 0x19's long-row wrap [M 0x2196e8:293].
- `List_SetIndex` 0x21b4c8: the row `index` hops from the first row; **index ≤ 0
  returns the first row**; off the end returns NULL [M :16-24].
- `List_SetIndex_RealItem` 0x21b520: counter starts −1; per row, `+0x2c8 == 0`
  bumps the counter; the first row whose counter == `index` is returned; NULL when
  the list is exhausted [M :10-26]. (Empty list derefs first+0x2c8 in the C — the
  port returns None; the game's only caller, arm 0x1d, never runs on an empty list.)

Sole caller of both SetIndex_RealItem and the plain SetIndex flows:
`List_SendMessage` 0x2196e8 — arm **0x1d** (scroll to real-item index, :313-324: the
row = SetIndex_RealItem(real), visible index +0x56b = GetIndex(row), then a plain
SetIndex re-sync and `Scroll_SendMessage(0x2e)`), arm **0x34** (get the current
selection's real-item index, :460), arms 0x49/0x52/0x54 (scroll sync),
arm 0x13 (remove row :149-159), arm 0xf (add row :88-141: `Menu_Malloc(0x2d0)`,
`LinkedList_Add` to +0x270, **the first row becomes the cursor** +0x560, Scroll 0x12,
immediate List_Update). Senders: `Menu_PutCodenamesIntoControl` 0x1f79c0:52-53
(P_NFSELECT/P_CNSELECT/P_MPJOIN/C_RBMPSTART), `Menu_GetMPSkins` 0x1ffc98:66
(P_MPSETUP/C_RBMPSETUP), `C_GCPAUSE_Handler` 0x204408:260-271 (the pause menu).

## List_Update 0x21a598 — the flow halves the port models

Per tick (Menu_Update 0x1f1dc0:28-29 type-6 arm; Combo_Update 0x1f0498:113 for
embedded lists; direct calls from Menu_CreateOptionBox :68 / PS2_DoCardFlow :246):

1. **Hide/free** (:110-161): parent/own state bits → free self + the cursor bar +
   every row cell sprite (sprite+0x38 = 0xff hidden).
2. **Metrics** (:162-184): rows +0x500 = h/stride; **top +0x534 clamped**;
   focus state +0x1c = 0x20 (selected) / 0x10 by `page+0xac → +0x18 == list+0x18`.
3. **Cursor scroll-into-view** (:216-233): when the cursor +0x560 ≠ 0, `List_GetIndex`
   it and re-clamp top so the cursor row ∈ [top, top + rows).
4. **Per-row draw layout** (:234-528) — the DRAW half: stripe
   sprites, per-column x from widths × (w−0x10) × 0.01, cell text sprites at
   x = colOff + page.x + list.x + comp + 6 + 2, y = (row−top)×stride + list.y + comp
   + 8 + off, the cursor row re-measured with the highlight font, per-column
   alignment, highlight colour 0x7f7f7fff with Menu_AlphaGet, cursor-bar placement.
5. Tail: Component_SetupInstance on the list itself; `Scroll_Update`.

## Port notes (this session's scope)

`ListState` in menudef.rs models the state + the four helpers + arms 0xf/0x13/0x1d +
Update halves 2-3. The draw half is separate (what must be drawn:
row cell texts at the layout above, the cursor bar, the embedded scrollbar).

Anomaly left open [S]: `Combo_Update` 0x1f0498:116 sends msg **0x3b** to the list,
but the decompiled `List_SendMessage` switch has no 0x3b arm (0x39 at :471, 0x3c at
:474; default → 0) — either dead or a Ghidra jump-table fold of arm 0x35 ("get the
selected row's value", :463-470, whose result feeds `Label_SendMessage(0x18)` at
0x1f0498:117). Raw-disasm 0x2196e8 around 0x11a0xx before porting the combo label
refresh.
