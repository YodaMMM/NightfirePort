# cn-controls-page — the codename Controls page (P_CNCONTROLS 0x40000022)

The codename wheel's row 1 ("Controls" — the wheel's label 0x1f3 [L ee_snapshot wheel table
0x2dfc28 row 1; the same id is the page's header label 499 [L the page dump]) opens
**P_CNCONTROLS 0x40000022** [M C_SBCNOPTIONS_Handler 0x213528:38-39 — case 1 → 0x40000022 →
`Manager_SendMessage(0x44, page)`]. The port had no model for it; this adds the data, the flow and
the getters (the main agent draws).

## The page's data

All disc facts are [L the page dump via `nf_formats::menudef` — 07000048.bin entry 08000002, page
0x40000022 (aux 0x7350048, kind 3, 52 controls), walked 2026-10-20]; the behaviour facts are [M]
the decompile under `work/decomp/ACTION.ELF/`.

- **The style list** — control **0x10000025**, rect (60,109,316,24) [L]. `P_CNCONTROLS_Handler`
  0x213cd0 fills it on the page-open message 0x4c with the 8 controller-style names,
  `Txt_BindLabel(id)` for ids **0x82, 0x91, 0xa0, 0xaf, 0xbe, 0xcd, 0x100018b, 0x100019a** at rows
  0..7 [M 0x213cd0:12-27] — the same 8 ids the in-game pause's Controls tab binds
  [M P_PAUSE_Handler 0x203c28; the port's pause draw comment cites them as the style names]. The
  selection IS the controller style: the commit sends the control's value (`Menu_SendMessage
  0x35`) to `Menu_ChangeControllerStyle(0, sel)` [M C_RBCONTROL_Handler 0x217908:13-14], which
  writes `PlayerSetting+0xE` [M Input_ChangeControllerStyle 0x17f5f0; the port's `PadConfig.style`
  doc cites the same]. The style's display arm is an 8-way jump [M Menu_DisplayControllerStyle
  0x1f3558 — `param_2 < 8`, the PTR_LAB_00305f50 table].
- **The Y-Axis (invert look) row** — the static label text 1170 at (376,113) and the value control
  **0x100000c9** at (461,109,99,25), whose disc placeholder label is 518 = 0x206 [L the dump]. At
  runtime the open handler overwrites the value with `Txt_BindLabel(0x3bd)` ("Normal") when
  `PlayerSetting+0x0 == 0`, else `Txt_BindLabel(0x206)` ("Inverted") [M 0x213cd0:31-38] — the same
  label pair the pause options' inversion row uses [M 0x203c28:80-93].
- **Header 499 (0x1f3), hint 1175 (0x497)** [L the dump, controls 39/43]. The pad diagram is the
  page's line-art controls (0x10000061, tex 50331714) [L]; the drawing is the main agent's.

## The settings behind the page

The codename profile's player-settings blob is the `PlayerSetting` array at **0x2a38c8
(1376 bytes, 4 player slots × 0x158)** [M the symtab; Input_Init 0x17edd8's stride walk]:

- `+0x00` (1 bit) — the **look inversion** [M Input_Update 0x17ee88:50-53: when set, the slot's
  vertical look floats (+0x24/+0x28 of the per-slot block) are negated; the port consumes the same
  flag as `PadConfig.invert_look`, negating actions 4/5 [M nf-game/src/input.rs:241]].
- `+0x0E` (4 bits) — the **controller style** [M 0x17f5f0; range < 8 per 0x1f3558's jump].
- `+0x10` (4 bits) and eight 1-bit toggles `+0x01..+0x0C` — the Advanced/AV options' bytes
  (`DAT_002a38d1/c9/d0/cc/cb/d2/d4/d3` — the ADV_ROWS doc cites them) [M LS_MakePlrSettings
  0x1cd418:37-48].
- **Defaults** [M Input_Init 0x17edd8:8-27]: memset 0, then `+0x0C=1, +0x01=1, +0x04=1, +0x05=1,
  +0x06=1, +0x07=1, +0x08=1, +0x0A=1`, `+0x0E (style) = 0`, `+0x10 = 0`, `+0x0B = 0` — so a fresh
  boot is **style 0, inversion off**.
- **Persistence** [M LS_MakePlrSettings 0x1cd418:37-48]: the save packs 19 bits per slot —
  inversion 1, style 4, `+0x10` 4, then the eight 1-bit toggles and the 2-bit `+0x0C` — into the
  profile blob behind a 4-byte word; `LS_LoadPlrSettings` 0x1cd5b8 unpacks it; `Boot_LoadPTPData`
  0x1c8560:78 restores the boot value from `DAT_0031cab4`.

## The flow

- **Open (msg 0x4c)** [M 0x213cd0:10-38]: fill the style list, send the style byte
  (`DAT_002a38d6`), display the style diagram, and **stage the inversion** — `was_inverted
  (0x30d535) = PlayerSetting[+0x0]` — then label the Y-Axis row from the STAGED bit.
- **Toggle (msg 0x5e)** [M C_RBCONTROL_Handler 0x217908:24-33]: `was_inverted ^= 1`; the row's
  label becomes 0x206 when staged 1, 0x3bd when staged 0. (The game's toggle is one unconditional
  flip — no separate left/right values.)
- **Commit (msg 0x4b)** [M 0x217908:11-17]: `cn_modified_flag (0x30d527) = 1`,
  `Menu_ChangeControllerStyle(0, <list value>)`, **`PlayerSetting[+0x0] = was_inverted`**, and
  `Manager_SendMessage(0x5f)` — the page closes back to the wheel. The committed inversion takes
  effect on the next Input_Update tick (the live read above).
- **Back**: the handler has no 0x4d arm [M 0x217908] — the staged bit is simply discarded on the
  next open's re-stage; nothing is written.

Port flow (the existing option-page machinery): the page has two rows — the style spinner (values
0..7, Left/Right step) and the Y-Axis row (Left/Right or Confirm flips — the port maps the game's
single 0x5e flip onto the row's step, an equivalent observable [S] — the game's exact focus/✕
wiring inside the widget is not pinned). Confirm commits both rows into the option map and marks
the settings file for save (the port's cn_modified_flag equivalent); Back restores the enter
snapshot without a save.

## How the settings reach the game

Committed values live in the option map under the row controls (`0x1000_0025` style, `0x1000_00c9`
inversion) and persist in the settings file's new `[cn]` section (the port's option persistence;
the game packs its own into the codename profile — see above). `FrontEnd::cn_player_settings()`
reads the committed pair; `nf_view::apply_cn_pad(&mut PadConfig, &opts)` writes it onto the pad
config (`invert_look`, `style`) the driver's `map_inputs` consumes — the same values the game's
Input_Update and the style mapping read. The windowed host calls it beside the AV volumes'
boot apply (the pad config is built once in gamepad.rs `Backend::new` and `PadDriver.cfg` is
public).

Tests: `crates/nf-view/tests/cn_controls_live.rs` — the rows/labels/controls against the dump
[L], the staged-toggle/commit/discard flow [M 0x213cd0 + 0x217908], the file round-trip, the reach
onto `PadConfig`, and the sister (the other codename pages' rows and the wheel table unchanged).
