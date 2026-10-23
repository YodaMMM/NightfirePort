# Options census — every options row and its consumer

Census of every player-changeable option row: the profile byte the game writes, every game
reader of that byte, and the port's consumer on the real path. Started
2026-10-21, when the Auto Aim option did nothing (only 3 values were read anywhere; Auto Aim itself is
wired since — `sim.auto_aim` from 0x1000011b in nf-view's lib.rs).

All option bytes live in two per-player structs:

- **`PlayerSetting[slot]`** @ `0x2a38c8`, stride `0x158`, 4 slots [M action-symtab 002a38c8
  "PlayerSetting" 1376 bytes]. `Input_Init` 0x17edd8 memsets each slot then writes the defaults.
- **`MPSettings`** @ `0x2a47a0`, header + 8 player records of `0x30` [M Menu_StoreMPSettings
  0x2027d0 memcpy(0x2dec68, 0x2a47a0, 0x1dc); LS_LoadMPSettings 0x1ce5f0 pulls +0x28/+0x2c at
  `base + slot·0x30`]. `bootup` 0x1c7e78:48-52 writes the defaults into all 8 records.

The front-end pages write slot 0: `P_CNOPTIONS_Handler` 0x213f08:13-21 (Advanced), 
`P_CNMPOPTIONS_Handler` 0x2144d8:13-15 (MP), `P_CNAVOPTIONS_Handler` 0x214818:128-134 (AV)
each read the row control with msg 0x35 and store. `P_CNCONTROLS_Handler` 0x213cd0 +
`C_RBCONTROL_Handler` 0x217908 commit Controls. Persistence: `LS_MakePlrSettings` 0x1cd418 /
`LS_MakeMPSettings` 0x1ce508 / `LS_MakeGlobalSettings` 0x1cd9d0 bit-pack them into the PLRS /
MPSG / GSET save blocks (the port mirrors this in `nf_game::profile`).

The port's storage is `FrontEnd::opts` (keyed by row control id), persisted by the host's
settings file. The drain pattern is the auto-aim one: every frame the host copies the option
out of `fe.opts` into the consumer (`crates/nf-view/src/lib.rs`, `App::drain_options`), so the
value survives across missions like the game's profile globals.

## Advanced options (page 0x4000002d) — P_CNOPTIONS_Handler 0x213f08:13-21

| Row ctrl | Option | Byte [M] | Default [M] | Game reader(s) [M] | Port consumer |
|---|---|---|---|---|---|
| 0x1000011a | Vibration | PlayerSetting+0x09 (`DAT_002a38d1`) | 0 (memset; preset table says 1 [S]) | `Input_RumbleStart` 0x17f6b8, `Input_RumbleStop` 0x17f748: `if byte != 0` gates `psiInput_RumbleStart/Stop` | **None — no haptics output.** The port's gamepad backend (`crates/nf-view/src/gamepad.rs`) maps pad input only; nothing emits force feedback. The sim's rumble events (`world_fx.rs`) have no host consumer either. Future item. |
| 0x1000011b | Auto Aim (SP) | +0x01 (`DAT_002a38c9`) | 1 | `Player_AutoAim` 0x1a7c98:40-44 (SP branch, `MPSettings+0x180 == 0`); forced on for weapon classes -0x50/-0x4f (:47-49) | `sim.auto_aim` (drained per frame; `s/auto-aim-option`) |
| 0x10000124 | Crosshairs | +0x08 (`DAT_002a38d0`) | 1 | `HUD_UpdateCrossHair` 0x19e6d0:91-100 — sprite +0x38 = 0xff (hidden) when 0, 0x31 when on; `C_CHCHHUD_Handler` 0x215f48:22 also mirrors it and enables/disables all 22 HUD panes | `sim.crosshair` gates the HUD crosshair draw (`hud.rs`) |
| 0x10000125 | Crouch Toggle/Hold | +0x04 (`DAT_002a38cc`) | 1 (Toggle) | `Player_SSCrouch` 0x1924c0:84-105 — 0=Hold: stand when `Input_Action(8,1) == 0` (released); 1=Toggle: stand on press edge of action 8 or 7 | `sim.crouch_toggle` + host feeds held-level in Hold mode / edges in Toggle mode (`lib.rs` input fill, `player.rs` stand branch) |
| 0x10000126 | Manual Aim Toggle/Hold | +0x03 (`DAT_002a38cb`) | 0 (Hold) | `Player_Weapon` 0x1a24c8:45-63 — 0=Hold: aim bit (obj+0x60 bit0) cleared each frame, set while action 0x13 held; 1=Toggle: aim bit XORed on press edge | **None on the keyboard path.** The mouse aims directly (D-002) and its right button is the scope (`lib.rs` 5850). The pad aim action is the `gamepad` feature's `aiming`; a pad toggle is a future item. |
| 0x100001cb | Weapon Auto Switch | +0x0a (`DAT_002a38d2`) | 1 | `Player_EquipWeapon` 0x1a8a58:76-79 — `!= 0 ∧ Player_IsBetterWeapon` → picked-up weapon becomes current | `sim.auto_switch` (field existed; now drained from the row) — read at the pickup take (`drone.rs`) |
| 0x10000226 | Near-flash (label 0x5b7) | +0x0c (`DAT_002a38d4`, 2-bit) | 1 | `View_SetupRenderModes_Tweak` 0x1e5888:22-45 — when on, objects with render flag 0x80000 within 4 units get their tint multiplied by `FlashMod` (1 Hz pulse 0.5..1.0, `Check_Target` 0x1a7810:95-97) | `sim.near_flash` gates `scene::flash_mod` application (`scene.rs`) |
| 0x1000023b | Health-pane fade (label 0x5bd) | +0x0b (`DAT_002a38d3`) | 0 | `Player_Update` 0x193600:414-429 — 0 pins BLData+0x8bc to 1.0, non-0 decays it (1.0 − 0.0020833·MUL/tick); set 1.0 by `Player_HandlePain` 0x1902c8:27, `Pickup_Handler` 0x18e650:225, `Player_Init` 0x1910f8:127; `HUD_UpdateHealthPane` 0x19b9c8 multiplies the health/armor pane alpha by min(4·v, 1) | `sim.decay_gate` + `Bond::pain_alpha` (sim timer ported; the HUD alpha multiply is draw-side) |

## Multiplayer options (page 0x4000002e) — P_CNMPOPTIONS_Handler 0x2144d8:13-15

| Row ctrl | Option | Stored [M] | Default [M] | Game reader(s) [M] | Port consumer |
|---|---|---|---|---|---|
| 0x10000128 | Radar | `MPSettings+0x28` (slot record) | 1 (bootup 0x1c7e78:50) | `MP_GetRadarObjects` 0x189840:20-24 (empty blip list when 0), `HUD_RadarUpdate` 0x19f6e8:100-105 (pane alpha 0x1e vs 0xff) | `FrontEnd::mp_radar` (synced with the row both ways; the lobby row is the same storage — C_RBMPSETUP 0x20de58:112 writes it per slot) → the match settings' per-player `radar_on` → `hud.rs` radar draw |
| 0x1000011c | Health Handicap | `MPSettings+0x2c` | 0 (bootup 0x1c7e78:52) | `Player_Init` 0x1910f8:128-131 + `MP_ReSpawn` 0x185368:71-73: spawn health = 100 + value; `HUD_UpdateMPHealthPane` 0x19b630:68-72 (bar denominator); `P_MPCONFIRM_Handler` 0x20ec00:212-218 (confirm text) | `FrontEnd::mp_handicap` (synced with the row both ways; lobby `MP_HANDICAPS` row is the same storage) → match settings handicap → sim spawn health `100 + extra` |
| 0x10000129 | Auto Aim (MP) | PlayerSetting+0x02 (`DAT_002a38ca`) | 0 (Input_Init memset only) | `Player_AutoAim` 0x1a7c98:40-44 (MP branch, `MPSettings+0x180 != 0`) | `sim.auto_aim`, picked when the match is MP (scenario != 0) |

## AV options (page 0x40000031) — P_CNAVOPTIONS_Handler 0x214818

| Row ctrl | Option | Stored [M] | Default [M] | Game reader(s) [M] | Port consumer |
|---|---|---|---|---|---|
| 0x10000135 | Music volume | `MusicMasterVolume` (`SFXMusicSetVolume` 0x116cf8:14; slider×5 on commit :72/:80) | 70 (Defaults row :97) | the IOP mixer (batch cmd 0x15); `SFXInitialise` re-pushes | `apply_av_volumes` → `driver.music_master` (boot + per-frame drain) |
| 0x10000134 | SFX volume | `SfxMasterVolume` (`SFXSetVolume` 0x116d98:14; ×5 :74/:81) | 100 (Defaults row :98) | the IOP mixer (batch cmd 8) | `apply_av_volumes` → `driver.master` |
| 0x10000136 | Subtitles | `DAT_002a37c8` (GameState+0x60) | 0 (bootup 0x1c7e78:34); auto-on for non-English (`Menu_WillRunMiniMission` 0x202450) | `Sound_DoSubtitle` 0x1dde58:21 (drops captions when 0 unless forced), `Text_AddMsg` 0x1b0c30:148, `ResetMap_Load` 0x1d6360:40 | `sim.subtitle_track.enabled` (drained per frame) |
| 0x1000019a | Split Screen | `DrawInfo` 0x2a37c0 | 0 (Horizontal) | `Camera_CreateCameras` 0x1bb2b8:44 (viewport rects), `Camera_CalcViewAngles` 0x1baf70:35 (aspect ×0.5/×2), `HUD_ValidateXY` 0x19a250:35, `HUD_FixMP` 0x19b110:33, `MP_Start` 0x183a50:188 | **None — the port has no 2-player viewports** (one camera, one window). Future item. |
| 0x10000198 | Speaker mode | `SFXModeNumber` (`SFXSetMode` 0x117078:14, immediate) | 1 = Stereo (bootup 0x1c7e78:85 `SFXSetMode(1)`) | the IOP output config (batch cmd 0x1c) | **None — no mono/surround downmix in `nf-audio`.** The OS mixer owns the output. |
| 0x10000223 | Widescreen | `DAT_002a37c4` (+ mirror `switch_CONFIG_WIDESCREEN`, `C_CHCHWS_Handler` 0x216168:9-10) | 0 | `Camera_CalcViewAngles` 0x1baf70:29-32 — projection aspect 1.3333 (4:3) vs 1.7778 (16:9) | The menu draw aspect already reads it (`menu.rs` `aspect_for`); the 3D view's projection gate is draw-side (the port renders at the window aspect; the pillar seam is `gpu.rs`) |
| 0x10000137 | Screen Adjust (action) | → subscreen 0x40000047 | — | `psiSetScreenPos` 0x10b910 (GS display offset) | **N/A — PS2 display centering has no PC counterpart** (the window is the screen). |
| 0x100001a3 | Restore Defaults (action) | writes control values 0x214818:96-110 | — | the changed controls' own commit path | Wired: sets the port's AV row values to the game's Defaults (music 70, sfx 100, subtitles 0, split 0, speaker 1, widescreen 0). |
| 0x10000138 / 0x10000222 | action rows | subscreen jumps (0x214818:91-93/112-114) | — | page scripts | navigation rows, no profile byte — out of the value census. |

## Codename Controls (page 0x40000022) — P_CNCONTROLS_Handler 0x213cd0 / C_RBCONTROL 0x217908

| Row ctrl | Option | Stored [M] | Default [M] | Game reader(s) [M] | Port consumer |
|---|---|---|---|---|---|
| 0x10000025 | Controller style | PlayerSetting+0x0E (u16; `Input_ChangeControllerStyle` 0x17f5f0:11-13 writes it, clears the cached action axes) | 0 | `psiInput_MapInputs` 0x10cd80:61 `switch(+0xe)` — the 8 pad→action mappings; `Input_Update` 0x17ee88:38-39 (styles 2/5 stick compensation); `Menu_DisplayControllerStyle` 0x1f3558 (8-way diagram jump) | **None — the gamepad backend ships one fixed mapping** (`gamepad.rs` ports the style table's curves/ramps, not the switch). Future item. |
| 0x100000c9 | Y-Axis | PlayerSetting+0x00 | 0 (Normal) | `Input_Update` 0x17ee88:50-52 — negates the two analog look channels (action floats 4/5) when non-0; consumed by `Player_Aiming` 0x1a1d28:32/78-79 and `camera_freechase` 0x1ba1c8:111 | The host negates the look delta at the input choke point when the row is Inverted |

## The port's drain (App::drain_options, lib.rs)

Runs every frame next to the auto-aim drain it generalizes. Game-side, these are profile
globals read at the moment of consumption; the port's `fe.opts` is that profile, and the
drain is the read. Volumes commit on page exit in the game (0x214818:72-81) — the port
applies them per frame, which is the same net effect through the persisted profile.
