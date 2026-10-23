# HUD panes: PaneList, the create/update dispatch, and the Text queues

The whole-ELF scan found 18 ACTION.ELF functions with no mention in
`crates/*/src` (13 `HUD_*` pane functions + 5 `Text_*`). This document is the evidence map for
them; the Rust homes are `crates/nf-game/src/hud_pane.rs` (the OICW boot + nightsight state),
`crates/nf-formats/src/hud.rs` (the sprite tables), and `crates/nf-game/src/subtitle.rs`
(citations for the subtitle Text_* pair). Drawing is nf-view's.

## The pane-def table (`PaneList` 0x2be5b8)

Each HUD pane is a 28-byte definition in `.data` [L `work/ee_snapshot.bin` — file offset = EE
address; values below read from it at boot, and the `.data` bytes agree with the ELF image
[D `work/elf/ACTION.ELF`]]:

| off | meaning | example (OICWPane 0x2be140) |
|-----|---------|------------------------------|
| +0x00/+0x04 | screen-anchor/format words | 0x00000000, 0x01c00200 |
| +0x08 | `create(BL*, HUDPANE*, def*, obj*)` | 0x19cc90 `HUD_CreateOICWPane` |
| +0x0c | `update(BL*, HUDPANE*, obj*)` | 0x19cef8 `HUD_UpdateOICWPane` |
| +0x10 | `SprInfo` table | 0x2be008 `OICWSprInfo` |
| +0x14 | low half = sprite count | 7 |

`PaneList` 0x2be5b8 (22 slots, `HUD_Init` walks exactly 0x16) [M 0x199e88:44-46 + the 136-byte
symbol; L snapshot pointers]: slot 0 `AmmoPane` 0x2bcfb8 (create 0x1a0ed8, update 0x19a7f0,
8 sprites), 1 `HealthPane` 0x2bd470 (0x1a0f40 / 0x19b9c8, 24), 2 `MsgMissionStatusPane`
0x2bd538 (create 0x1a1028 / update 0x19c310; count word 3 on disc [D], 6 at runtime [L]), 3 `MsgObjectiveStatusPane` 0x319710 (BSS,
create 0x1a1070 / update 0x19c310, 6), 4 `MsgInfoStatusPane` 0x319810 (BSS, create 0x1a10b8 /
update 0x19c310, 5), 5 `AirPane` 0x2bd518 (update 0x1a11c8), 6 `SightPane` 0x2bd5e0 (create
0x1a0bb8 / update 0x1a12d0, 3), 7 `NightSightPane` 0x2bd8b8 (create 0x1a0bb8 / update 0x19d218,
6), 8 `LensFlarePane` 0x2bda10, 9 `RedeemerPane` 0x2bdc18, 10 `RCCarPane` 0x2bdd40 (create
0x1a0bb8 / update 0x1a15a0, 6), 11 `CameraPane` 0x2bd790 (create 0x199b70 / update 0x1a12d8,
9), 12 `BloodPane` 0x2bd030 (create 0x1a0bb8 / update 0x19a6a8, 2), 13/14 null, 15 `XrayPane`
0x2bde90 (create 0x199b70 / update 0x1a1410, 5), 16 `SecCamPane` 0x2bdfe8 (create 0x199b70 /
update 0x1a1538, 7), 17 `OICWPane` 0x2be140, 18 `RoninPane` 0x2be268 (create 0x1a0bb8 /
update 0x1a12d0, 6), 19 `LaserPane` 0x2be390 (create 0x1a0bb8 / update 0x1a12d0, 6),
20 `SpacePane` 0x2be598, 21 `MsgPickupStatusPane` 0x319910 (BSS, create 0x1a1140 / update
0x19c310, 5). The three BSS defs are built by the C++ static initialiser
`__static_initialization_and_destruction_0` 0x1a0058 [M]; their runtime create/update words
above are [L ee_snapshot.bin 0x319710/0x319810/0x319910]. MP uses a second table `MPPaneList`
[M 0x199e88:20-22; symbol `MPAmmoPane` 0x2be698 create 0x1a0ed8 [L snapshot]].

## Reachability chain [M]

- `Player_Init` 0x1910f8 calls `HUD_Init` 0x199e88 — every on-foot level load, SP and MP.
- `HUD_Init` 0x199e88:24-70 allocates the HUDINFO at `BL+0x7e0` (0x2c8 bytes), then for each of
  the 22 slots: def non-null and `def->create` non-null → enable bytes `entry+0x24 = 1`,
  `entry+0x08 = def`, and calls `def->create(BL, entry+8, def, obj)`; afterwards, if
  `entry+0x14` is non-null it calls the update once.
- `HUD_Update` 0x19a088 runs each frame (`param_2` = the local player object): it calls
  `HUD_UpdateCrossHair` 0x19e6d0, then **`HUD_MonitorNightSight` 0x19e9d8 directly** [M
  0x19a088:15], then walks the 22 entries: disabled panes get every sprite's layer forced to
  0xff (hidden) [M 0x19a088:22-38]; enabled panes get each sprite's layer restored from
  `def->sprinfo + 0x8 + 0x2c·i` (SpriteInfo+8 = layer) [M 0x19a088:47-58]; in both cases the
  entry's update fn (`entry+0x14`) runs.

So every function below is **reached** on every on-foot level (SP and MP), unless its own body
says otherwise. Nothing here is debug-only or driving-only.

## The 13 HUD functions

### `HUD_CreateShrink` 0x1a0bb8 (332 B) — reached; ported as pane geometry helper

The shared create for 6 panes (Blood, Sight, NightSight, RCCar, Ronin, Laser — the L-snapshot
create words). Body [M 0x1a0bb8]: `HUD_CalcWidthHeight` 0x19a2c0-family width/height into the
create tag, `HUD_CreateDefault` 0x199b70-family link, then for each of `def->count` sprites:
if the viewer's screen height (`viewer+0x10c`) < 512 the sprite's w (`+0x2c`) halves and its x
(`+0x24`) shifts by `−create_tag_x/2`; if `viewer+0x110` < 448 the h (`+0x2e`) halves and y
(`+0x26`) shifts by `−create_tag_y/2` — the panes re-centre themselves on sub-fullscreen
viewports. Ported as `hud_pane::shrink_to_viewport` [M] with the same guards.

### `HUD_CreateAmmoPane` 0x1a0ed8 (104 B) — reached; cited

[M 0x1a0ed8] = `HUD_CalcWidthHeight` + `HUD_CreateDefault` only. The pane's runtime look lives
in `HUD_UpdateAmmoPane` 0x19a7f0 (the port draws it: `crates/nf-view/src/hud.rs` ammo section,
`HudTables::ammo` = `AmmoSprInfo` 0x2bce58). Cited at `crates/nf-formats/src/hud.rs`
(`HudTables::ammo` field doc).

### `HUD_CreateHealthPane` 0x1a0f40 (228 B) — reached; cited

[M 0x1a0f40] = CalcWidthHeight + CreateDefault, then zeroes each of the 24 sprites'
`+0x24/+0x26/+0x2c/+0x2e` (position/size) — the table's own geometry wins — and resets
`ThirdIconTimer` and `CrouchIconTimer` to 0. The port draws this pane (`HudTables::health`,
nf-view hud.rs health section; timers are the port's damage-glow/pain state
`Drones.health.flash`). Cited at `crates/nf-formats/src/hud.rs`.

### `HUD_CreateInfoStatusPane` 0x1a10b8 (132 B) — reached; cited

[M 0x1a10b8] = CreateDefault; then the status header at `pane+4` gets count 0, max 1, format
0x10; in MP (`MPSettings+0x384`) `HUD_FixMP` re-colours and shifts the sprite row +0x48. This
is the in-game "info" status line (MsgInfoStatusPane, slot 4). The port's status line is
`nf_game::mp::hud::status_line` + the pause objective state; the SP pane is the objective
ticker. Cited at `crates/nf-game/src/mp/hud.rs` (status_line) and
`crates/nf-formats/src/hud.rs`.

### `HUD_CreatePickupStatusPane` 0x1a1140 (132 B) — reached; cited

Identical shape with max 6 [M 0x1a1140] (the pickup queue line: "Picked up …"). Same homes as
above.

### `HUD_UpdateSightPane` 0x1a12d0 (8 B) — reached; a genuine no-op

[M 0x1a12d0] the body is a bare `jr ra` (8-byte function). The Sight/Ronin/Laser panes have no
per-frame update; their sprites stay as created. Recorded here; nothing to port.

### `HUD_UpdateNightSightPane` 0x19d218 (412 B) — reached; ported in `hud_pane::NightSightFx`

Runs only while the pane's enable byte (`pane+0x1c`) is set and the viewer mode
`viewer+0x236 == 1` (nightsight on) [M 0x19d218:34]; otherwise clears the enable byte (the
pane parks itself). While `DAT_002d88c0+0x29` (the pause/freeze flag) is clear [M :37]:
- sprite 0's colour word = `Rand(0x18) + 7 | 0xff0000` — the green tint pulses [M :39-40];
- every other tick (`Scrl_247` bit 0) sprite 0's v = `Rand(sprite->h − 1)` — the scanline
  jitter [M :43-47];
- sprite 1's h/v = `max(1, 28 + (BL+0x920) · (1/1800) · 200)` — the brightness step grows with
  the fade-in timer [M :63-79].
In MP the Light_Create at :49-51 is skipped (only SP adds the light). The port keeps the exact
arithmetic but the HUD's own `Rand_Rand` calls are the pane's private stream — the same
decision as the redeemer pane ("a HUD draw must not move the game's draws"): a per-frame hash,
not the sim RNG. Documented as the port choice in `hud_pane.rs`.

### `HUD_UpdateXRayPane` 0x1a1410 (296 B) — reached; ported in the same `NightSightFx`

Same shape with mode `== 2` (x-ray), tint `Rand(0x18) + 7 | 0xffffff00` (blue), and the v
counter wraps at 0x1f instead of re-rolling [M 0x1a1410:17-30].

### `HUD_UpdateSecCamPane` 0x1a1538 (100 B) — reached; ported (data) in `NightSightFx::sec_cam_tint`

[M 0x1a1538] while enabled: `viewer+0x236 = 0` (leaving the sec-cam view resets the goggle
mode) and the sprite row gets colour word `0x7f7f7f78` when `GameState+0x34 & 0x1f < 0x10`
(NTSC/PAL field counter low) else `0x7f7f7f00` — the interlace flicker of the security-camera
black-and-white wash.

### `HUD_UpdateCarPane` 0x1a15a0 (256 B) — reached; ported as `hud_pane::car_pane_layers`

[M 0x1a15a0] the RC-car pane (slot 10; the RCCar gadget's viewer). While enabled and
`BL+0x878` (the current vehicle/gadget object) non-null: if the object's cel group
(`obj+0xd8`) equals the hash table entry 0x2000194 (`hashtable_hashcode_to_celglist` — the
RC car's cel), the 5 sprites' layer bytes (`+0x38`) become `[0xff, 0xff, 0x27, 0x27, 0x27]`
and sprite 0's colour `0x0020ff`; otherwise `[0x27, 0x27, 0xff, 0xff, 0xff]` and `0x2000ff`
— the two views (car-cam vs normal) swap which sprites are on the deep layer.

### `HUD_CreateOICWPane` 0x19cc90 (612 B) — reached; ported as `hud_pane::OicwBoot`

[M 0x19cc90] CalcWidthHeight + CreateDefault + the same sub-viewport shrink as CreateShrink;
then (SP only) `OICW_mode = 0; OICW_timer = 0x96` and the five text sprites get their
boot lines: `"COMMAND.COM\n"`, `"LOAD BIOS\n"`, `"MEMORY SET\n"`, `"SYSTEM STATUS\n"`, and a
fifth from `DAT_0030ccc8` = `"OK\n"` [D the ELF image] (the `Sprite_SetText` calls at
0x19cdd4-0x19ce50). In MP:
`OICW_timer = 0xfdd5` (a negative — mode 2 immediate steady state), `OICW_mode = 2`.
Ported: `OicwBoot::new(mp: bool)` with those exact constants and the five line strings.

### `HUD_UpdateOICWPane` 0x19cef8 (800 B) — reached; ported as `OicwBoot::tick`

The boot animation [M 0x19cef8], SP only (`MPSettings+0x384 == 0`):
- pane disabled and `OICW_mode != 2` → reset to mode 0, timer 0x96 [M :15-19];
- pane enabled sets `viewer+0x236 = 0` [M :21];
- **mode 0** (boot list): timer counts down from 0x96; line k (of 5) is on layer 0x1c (visible)
  while `k < 5 − timer/30` (`iVar5 = 5 − OICW_timer/0x1e` at :53); the block rises — line k's
  y = `408 − 17·(visible − k)` (the y starts at `408 − 0x11·visible` and advances 0x11 per
  line, :53-58), x = 0x32 (50); while exactly 2 lines show (`k == 1`, `timer % 8 < 4`) the
  "LOAD BIOS" line drops to layer 0xff for those ticks [M :60-71]; at timer < 1 → mode 1,
  timer 0x1e [M :80-84];
- **mode 1** (flash all): every 4 ticks (`timer % 4 < 2`) all 5 lines flip between 0xff and
  0x1c [M :24-45]; at timer < 1 → mode 2 [M :46-50];
- **mode 2** (steady): all 5 lines at 0xff [M :52-64].
Ported as a pure state machine over the five lines: `OicwBoot::tick(&mut self, pane_enabled:
bool, frame: u32)` + `lines()` getter for the draw code (text, x, y, alpha per line). The
blinking uses the boot timer itself, so no RNG and no wall clock.

### `HUD_MonitorNightSight` 0x19e9d8 (820 B) — reached; ported as `hud_pane::MonitorNightSight`

The goggles' flow controller, called directly from `HUD_Update` [M 0x19a088:15]. Guards [M
0x19e9d8:18-41]: BL non-null; `BL+0x94a` (HUD active) set; viewer state `player+0xf4 == 1`
(the on-foot view); SP only; weapon class ≠ 0x12 (no goggles while carrying the item); the
viewer `+0xf6` (gadget sub-state) in 0..5 only. Then [M :43-101]:
- if the current weapon is the goggles (class `']'` = 0x5d at `weapon+0x62` high byte): while
  the zoom pane (BL+0x7e8's `+0xf4`) is off, force mode 1 (nightsight) [M :45-56], set the
  item's `+0x63 = +0x64` pair and `BL+0x961 = 1` (the "was goggles" latch), clamping item
  classes 'G'/'\\'..'_' to 1 [M :60-66];
- else on the toggle action (`Input_Action(player, 0xd, 4)` — the select-button action) and
  the weapon's `+0x60` bit 0 clear: cycle `viewer+0x236` 0 → 1 → 2 → 0 [M :72-101], and for
  each transition clear/set the night pane object (`BL+0x7e0` chain `+0x102/+0x104`) and the
  x-ray pane (`+0x202/+0x204`) enable words [M :79-97];
- the fade timer `BL+0x920`: grows by `3·FRAME_RATE_MUL` toward 1800 while the pane is off
  (`sVar3 == 0`), decrements by FRAME_RATE_MUL while a mode is up [M :103-112]; at ≤ 0 with a
  mode still set, clear the mode and both pane enable chains [M :114-136].

Ported as `MonitorNightSight` over explicit inputs (`goggles_equipped`, `toggle_action`,
`zoom_pane_on`) producing `mode` (0/1/2) + `fade` — the state the pane updates and the
renderer consume. When this was written the port had no goggle item wiring, so the struct was delivered as the flow +
getters; it is wired since (docs/systems/goggles.md).

## The Text functions

### Subtitle queue pair — reached; cited in `crates/nf-game/src/subtitle.rs`

- `Text_Init(char)` 0x1b1428 (228 B): zeroes `SubTitleMsg`, creates the 3 subtitle sprites
  (`SubTitleSpr/1/2`, `Sprite_Create2(0x319940)`), `DList_Init(0x319970, 0x20, 0)` (the 32-node
  TXT_MSG free list), and the 0x80-entry circular receive buffer at 0x2bf140; the `param_1`
  warm path re-locks `ReceivedMsgs` strings instead of re-initing [M 0x1b1428]. Called by
  `ResetMap_GameInit` 0x1d5080 [M grep] — every level load, SP and MP.
- `Text_DelMsg(TXT_MSG*)` 0x1b1640 (192 B): unlocks the message's string (per the type switch:
  type 1/4/6 unlock; type 4 also clears `SubTitleStr/2` if it is the active subtitle [M
  0x1b1640:12-30]) and returns the node to the free list via
  `DList_RemoveFromInUse2Free(0x319970, msg)` [M :36]. Called by `Text_FlushAllSubtitles`
  0x1b15c8 and `Text_UpdateMsg` 0x1b11d8 [M grep] — the subtitle expiry/skip paths.

The port's `SubtitleTrack` (subtitle.rs) implements the same queue under other names:
`SubtitleTrack::new`/`from_script` = `Text_Init`'s reset (the 3 sprite slots are the port's 3
`SubtitleLine` draw slots), `SubtitleTrack::flush` = `Text_FlushAllSubtitles` →
`Text_DelMsg`'s unlock-and-free of every queued message, and the `push` dequeue path =
`Text_UpdateMsg`'s delete-on-expiry. Cited with name + address in subtitle.rs doc comments.

### M_TEXTBOX trio — reached through the front end's control factory

- `Text_Init(M_TEXTBOX*, NEW_CONTROL*, M_CONTROL*)` 0x222b80 (2688 B): the type-0xc control
  constructor (0x9b0 bytes) — text state, script slots, the char queue at `+0x478/0x488` with
  the count at `+0x9ac` [M 0x224098's writes]. Created by the control factory
  `Manager_SendMessage` 0x21c858 when a page-def control's type byte is 0xc [M 0x21c858:206-208].
- `Text_SetupChar` 0x224098 (116 B): appends one typed character `{char, 4 words, flag}` at
  `+0x488 + 0x14·++count` [M 0x224098:12-22] — the typewriter queue. Called by `Text_Init`
  0x222b80 [M grep].
- `Text_SendMessage(M_TEXTBOX*, msg, a, b)` 0x223600 (456 B): the control's message switch —
  0x14 `Script_ExitLoop`, 0x16 `Script_Play` (typewriter start), 0x21 `Script_Select` into the
  7 default slots at `+0x7c`, 0x2b set char flag, 0x39/0x3b/0x3d return handles, 0x47 free
  (Label_SendMessage + `Component_Free` ×4 + `Script_DelAllScripts` + `Menu_Free`), 0x53
  show/refresh [M 0x223600:26-90]. Called by the generic router `__Menu_SendMessage` 0x1f1c18
  and by `TextBox_Update` 0x2237c8 [M grep].

Reachability: the factory arm is [M]; whether a shipped page instantiates one is a disc data
question. The disc census in `crates/nf-formats/tests/hud_pane_census.rs` parses the kind-8
menu def (08000002 in 07000048.bin, the file the port's `menudef.rs` already reads) and counts
`ctype == 0xc` controls [D]. The census found none, and the trio is recorded unreached in
`docs/port-map/unreached.md` ("M_TEXTBOX: no shipped page builds one").

## What the renderer must draw (not drawn here)

1. **Nightsight pane** (slot 7): two full-screen sprites — 0 the tint/scanline (colour pulsing
   `Rand(0x18)+7 | 0xff0000`, v re-rolled on odd ticks), 1 the brightness step at
   `max(1, 28 + fade·200/1800)` — while `mode == 1`.
2. **X-ray pane** (slot 15): same two sprites with the blue tint `0xffffff00` base and the
   v counter wrapping at 0x1f, while `mode == 2`.
3. **OICW boot pane** (slot 17): the five text lines at x = 50, y = `408 − 17·k`, alpha per
   `OicwBoot::lines()` (the COMMAND.COM boot list, the flash, the steady state).
4. **RC-car pane** (slot 10): 5 sprites whose layer bytes swap between
   `[0xff,0xff,0x27,0x27,0x27]` and `[0x27,0x27,0xff,0xff,0xff]` by the cel-group test.
5. **Sec-cam wash** (slot 16): the full-screen `0x7f7f7f78 / 0x7f7f7f00` interlace alternation.
6. `HudTables` gains `oicw / nightsight / xray / secam / sight / rccar / blood / ronin / laser
   / camera` sprite tables (from `OICWSprInfo` 0x2be008 etc.) — the tables the draw code needs.
