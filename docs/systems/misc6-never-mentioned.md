# misc-6: the 25 never-mentioned functions — evidence and the port's mapping

Whole-ELF scan (2026-10-21). For each: the callers in
`work/decomp/ACTION.ELF` [M], whether the shipped game reaches it on an on-foot level / the
front end / MP, and where the port does the same job under another name. Nothing here is
drawn by the port — these are the data/state/flow/getters.

Three families sit under one roof each:

## 1. The front-end widget runtime (M_* controls)

The front end is a widget system: page defs build NEW_CONTROL records, `Manager_SendMessage`
0x21c858 creates and pumps them, `__Menu_SendMessage` 0x1f1c18 dispatches per control kind,
`Handler_HandleMessage` 0x2029e0 is the per-widget handler table.

| function | reached how [M] | the port's site |
|---|---|---|
| `Button_Init` 0x1efc90 (216 B) | `Manager_SendMessage` 0x21c858's create switch | `FrontEnd::action` citation block (`crates/nf-view/src/menu.rs`) — the port's per-page row construction |
| `Button_SendMessage` 0x1ef7a0 (420 B) | `__Menu_SendMessage` 0x1f1c18's kind dispatch | same block — the port's per-row Confirm/step in `action` + `opt_step` |
| `Window_Init` 0x224110 (124 B) | `Manager_SendMessage` 0x21c858 | same block — pages draw their frames directly |
| `Window_Update` 0x224270 (8 B) | `Menu_Update` 0x1f1dc0's per-tick control walk | same block — an empty body even in the game [M decomp] |
| `Manager_HandleMessage` 0x21df10 (28 B) | `Manager_SendMessage` 0x21c858's tail | same block — pure forward to `Handler_HandleMessage` 0x2029e0 [M symtab] |
| `Radio_Update` 0x21ff80 (428 B) | `Menu_Update` 0x1f1dc0 | `FrontEnd::opt_step` doc — the M_RADIO ◀▶ row; Radio_Init/SendMessage/GetIndex/SetIndex were already cited there |
| `Font_GetAlignment` 0x1ef728 (116 B) | `Label_Init` 0x218638 / `Label_Update` 0x218cb8 | `FrontEnd::action` citation block — the port takes alignment from page-def rows, not the game's embedded 0xFA string tags |
| `Font_GetTextExtent` 0x1ef588 (220 B) | 11 front-end call sites (Sprite_SetFmt 0x1de628, Text_Init 0x222b80, List/Checkbox/Spin/Memo inits, Menu_ClipString 0x1f2e70, …) | `nf_formats::hud::Font::layout` (nf-formats/src/hud.rs; the name was already cited at nf-view/src/hud.rs:13) |
| `Font_GetTextCountExtent` 0x1ef668 (188 B) | `List_SendMessage` 0x2196e8, `Menu_ClipString` 0x1f2e70 | `Font::word_wrap` doc — the prefix-measuring job shape; the port re-measures whole prefixes with `layout` instead of counting glyphs |

The M_RADIO per-tick member `Radio_Update` is draw-time layout only: it resizes the label
child into the radio box (+0x70..+0x76 minus the scroll arrows), honours the +0x7a hide
flags and re-runs `Scroll_Update` + `Label_Update` [M 0021ff80]. Nothing gameplay-visible
for the port to model; the ◀▶ selection state already lives in `FrontEnd::opts`.

## 2. Matrix composition (EE FPU scalar 3×3)

All six are reached on ordinary gameplay paths (callers grepped [M]):

- `MulMatrix0` 0x1e2820 — `Car_Update` 0x1307d8, `Draw_MuzzleFlash` 0x1a3fe0,
  `Bullet_DoTrails` 0x12d890, `Bullet_update` 0x12e7b0, `Camera_SetToPlayer` 0x1bc048.
- `MulMatrix2` 0x1e26b0 — `DroneWeap_ThrowGrenade` 0x179558 (`m = a·m` in place).
- `RotTransMat` 0x1e29d8 — `RotTransMatrix` 0x1e2478, `View_RotTransMatrix` 0x1e5168,
  `Control_GetObjMatrix` 0x134210, `control_create_object` 0x1344e0, the camera suite
  (`camera_freechase`/`camera_snake`/`Camera_ApplySwing`/`Camera_Enable`),
  `AnimGetBoneWorldTrans2` 0x1c32e0 (`m = rot·m`, translation += rot's).
- `RotMatrixY` 0x1e4718 — `Player_SSScanMode` 0x193098, `GT_Update` 0x180498,
  `GunImp_Update` 0x1320d8, `Car_Update` 0x1307d8, `Car_SetBits` 0x130060.
- `RotMatrixZ` 0x1e4798 — `GT_Update`, `Car_Update`, `Draw_MuzzleFlash`, `Bullet_update`,
  `Player_PositionCamera` 0x18f5d8, `Player_PositionGun` 0x1a3b40, `RotMatrixZYX` 0x1e25a8,
  `View_DrawCelGlist` 0x1e6538.
- `RotMatrixXY` 0x1e4818 — `Car_SetBits` 0x130060 only (the car frame's pitch/yaw).

The port composes matrices with `nf_formats::anim::mul` (the VU0-guard-bit palette path)
and `nf_game::sky::rot_matrix`/`concat` (the `RotMatrix` 0x1e2368 / `Concat` 0x1e4a90 port),
plus per-site inline builders (`drone.rs`'s `rot_y` aim matrix). Citations with the full
caller lists live on those three functions' docs: `crates/nf-game/src/sky.rs`
(`rot_matrix` — the Euler family incl. RotMatrixY/Z/XY; `concat` — MulMatrix0/2 +
RotTransMat) and `crates/nf-formats/src/anim.rs` (`mul`). The car matrix composition itself
(RotMatrixZYX + the quat integrator) is the declared next-session `Car_Update` work
(`car.rs` module doc) — no car-specific port claims here.

## 3. AI navigation bind (every level load)

`GameFlow_Main` 0x1c98c8 → `ResetMap_Load` 0x1d6360:
`ResetMap_GameInit` 0x1d5080 → `AIPath_Init` 0x1d2ea8 (zeroes the five path-system globals
DAT_002c7944/58/5c/60 + 0x2c8774), and → `AIPath_BindNodes` 0x1d2af8 →
`AIPath_Link2Cel` 0x1d2ec8 (each node head-inserted into its room cel's +0x24 list) +
`AIBounds_Prepare` 0x1d3008 (boundary edges into their two end cels' +0x34 lists) [M].
Reached on every on-foot and MP level load alike.

The port rebuilds the network per level with no cels (the one-cel deviation,
`crates/nf-game/src/nav.rs` module doc): Init's reset is the fresh `Network::build`,
Link2Cel's per-room lists are subsumed by the single-cel model, AIBounds_Prepare's per-cel
edge lists are the flat `walls` + `first_wall_hit`/`bounds_test`. Cited at the module doc
and `Network::build`.

## 4. Load/save boot plumbing

- `LS_FlushStates` 0x1ce1b0 (12 B: `Load_State = 0; Save_State = 0`) — reached from
  `PS2_DoCardFlow` 0x217bd8's terminal block (jal at 0x2184e8) and
  `Menu_UpdateMessageBox` 0x1f7de8:25 (jal at 0x1f7ea8) [M ELF jal scan]. The port's
  save/load is a synchronous file write with no async LS state machine — nothing to flush;
  cited at `FrontEnd::reload_profiles` (alongside the already-documented `Menu_FlushEnum`).
- `LS_LoadFromBuffer` 0x1ce1c8 (148 B) — one caller: `Boot_LoadPTPData` 0x1c8560:17, which
  `GameFlow_Main` 0x1c98c8:80 runs at boot [M]. When the PTP block's first 4 bytes (the
  carried codename text) are not all zero it zeroes `InBufData` 0x3416e0 and copies the
  block in (0x12c0 bytes), priming the LS slot buffer the LS_* parser reads. The port
  re-applies the carried fields directly in `ptp::DriveReturn::apply` (status/stats/
  difficulty) and the codename profile carries the rest — cited there.
- `LS_LoadBonus` 0x1ce488 (124 B) — **never reached**: zero references in the whole ACTION.ELF
  LOAD segment (no jal, no absolute word) [M capstone scan, every 4-byte step of
  vaddr 0x107100..0x30d670]. The decompile is also register-garbage (unaff_s0/s1), so the C
  cannot even be read faithfully; the disassembly shows two `BIN_PullBits` runs + a
  `Menu_SetBonus` call. Dead code → `docs/port-map/unreached.md`.

## 5. World/visibility (drawing-adjacent — data/flow only)

- `OneSided_Update` 0x1e7d58 (160 B) — reached through the object-kind dispatch table
  `control_funcs` 0x26f890 (85×12-byte entries): index
  (0x26fc20 − 0x26f890)/0xC = **0x4c**, the exact kind byte `OneSided_Create` 0x1e7ce8
  writes at obj+0xff; `OneSided_Create` is `parsemap_create_dynamic_objects` 0x1d04f0's
  placement case 0xd5 [M]. Level 07000041's placement census lists 0xd5 [D levels.md].
  It dots (obj pos − `glb_viewer` cam pos) against the obj matrix's dir and sets obj+0xF0
  bit 0x10 when facing away. The port's per-triangle backface cull of the placed glist is
  the same job at draw time — cited at `crates/nf-view/src/scene.rs`'s static-dynamics
  comment (the OneSided_Create line).
- `vision_generate_display_list` 0x1e8848, `Vision_Init_Portal_Recurse` 0x1e89a8,
  `vision_GetCamPos` 0x1e8990 — all inside `View_CaptureScene` 0x1e6df0 (their only
  caller) [M]: reset `VisCelCount` + rebuild the view frustum, sort `VisCelList` by
  recursion depth and thread the cels through cel+0x20, and read the eye position from
  `ViewPoints` 0x2d67d0. The port's `portal_cel_chain*` in `crates/nf-view/src/scene.rs`
  produces exactly that sorted order from the eye — cited on
  `portal_cel_chain_from_cell_doors`.

## Test coverage

No new behaviour was ported, so no new pass/fail assertions: every item above is either a
citation (the port already did the job under another name — the tests that pin those sites
already exist) or the unreached record for `LS_LoadBonus` (absence of references cannot be
asserted from Rust; the evidence is the capstone scan documented here). The existing suite
(`tools/test/suite.py`) guards the cited sites.
