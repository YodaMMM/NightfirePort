# The level-exit sequence (the screen fade + the music fade-out)

Riding the 09 lift out, the level end did not match the game's: the game fades the screen to
black, the port cut with a short stall. The port froze the world for the
exit (`ExitHold`, docs/systems/exit-hold.md) but drew no fade and never ran the music's exit arm
on most levels — a freeze, then a hard cut. This doc is the game's whole
exit sequence with the evidence; the port now follows it.

## 1. The exit trigger's tail (the LOAD_LEVEL trigger)

`Trigger_Activate` 0x1b2318, the type-10 arm (disasm 0x1b24b8-0x1b24f4):

| step | game | evidence |
|---|---|---|
| cancel gates | returns if ch 96/97/98/99 stand | [M 0x1b2318:64-75] |
| channel logic | the gate channel (+0x26) → success ch (+0x2a) / fallback (+0x2c) | [M 0x1b2318:76-107] |
| 1 | `Music_Event(3, 1)` — the end-sting event onto the board | [M 0x1b24bc-0x1b24c0] |
| 2 | `GS_PauseGame(1)` — the world pauses | [M 0x1b24c4-0x1b24c8] |
| 3 | `ResetMap_LevelToLoad(ext+0x20, 1, 0)` — warm, queues state 3 (the load) under the fade | [M 0x1b24cc-0x1b24d8; 0x1d3800 pushes state 3] |
| 4 | `GameFlow_PushState(300.0f, 7, 0xff)` — the fade push | [M 0x1b24dc-0x1b24ec: f12 = 0x43960000 = 300.0f, a0 = 7, a1 = 0xff] |
| 5 | trigger +0x38 = 99 (spent); `Player_RamSave` when `glb_blokes != 0` | [M 0x1b24f0-0x1b250c] |

## 2. The screen fade

`GameFlow_PushState` 0x1c9f38: a state-7 push **runs `Camera_SetFade(6, duration, colour, 1)`
first** and pushes state **8** instead [M disasm 0x1c9f7c-0x1c9f90]. If the fade object refuses
(a fade already in flight) the push is ABORTED — no state 8 [M 0x1c9f88 `beqz $v0` → return].

`Camera_SetFade` 0x1bbdd8 (disasm 0x1bbe20-0x1bbf30), on `glb_viewer[6]` (0x2e88b0 + 6·4 = the
pointer GameFlow polls [M 0x1c9d8c `lw v0,-30520(v0)` = 0x2e88c8]):

- rate `+0x22c = (60.0 / duration) · (1/60) · (alphaByte/255)` — every caller passes colour
  0xff, so the rate is **1/duration per tick** [M]; `FRAME_RATE_MUL = 60/FRAME_RATE` = 1 at 60 Hz
  [M GS_SetRefreshRate 0x17d2b8:15].
- alpha `+0x228` starts 0.0 (rate ≥ 0, fade-out) or 1.0 (rate < 0, fade-in) [M 0x1bbe8c-0x1bbebc].
- colour bytes `+0x230..+0x232 = colour>>24/16/8` (0,0,0 = black), current alpha byte `+0x233`,
  mode `+0x234`, active flag `+0x235 = 1` [M 0x1bbefc-0x1bbf2c].

`Camera_UpdateGlbVars` 0x1bada0 (every frame, `Game_Run`'s tail `Camera_UpdateAll` [M 0x1c9790]):
`alpha += rate · FRAME_RATE_MUL`, the byte clamps to 0..255; while 0 ≤ alpha ≤ 1 the flag stays 1,
crossing the end sets **2**, the next tick clears it to 0 [M 0x1bada0:16-35]. Linear.

`GameFlow_Main` 0x1c98c8 case 8: while `viewer6->+0x235 == 1` the state stays; the tick the flag
leaves 1 it pops [M 0x1c98c8:173-189]. The load (state 3, `ResetMap_Load` 0x1d6360) sits UNDER
state 8 on the stack, so **the level loads only after the fade finishes** [M stack order;
`ResetMap_LevelToLoad` pushed state 3 first]. `Game_Draw` 0x1e6f50 draws the fade per viewer
while `viewer->+0x2a != 0 && +0x235 != 0`: `psiFadeView(R+0x230, G+0x231, B+0x232, A+0x233)` — a
full-screen quad [M 0x1e6f50:203-209].

**The trigger exit fades to black over 300 ticks (5 s at 60 Hz)** [M 0x1b24dc + §1].

The fade-in on the next level: `Player_Init` 0x1910f8:45 calls
`Camera_SetFade(player, -45.0f, 0xff, 1)` — 0xc2340000 = −45.0 in f12 [M disasm] — a 45-tick
(0.75 s) fade up from black on every spawn.

### The other durations (for contrast, all [M] call sites)

| caller | duration | colour |
|---|---|---|
| `Trigger_Activate` type 10 (level exit) 0x1b2318:112 | 300.0f | 0xff black |
| `Trigger_Activate` type 0xd (movie) 0x1b2318:61 | 300.0f | 0xff |
| `Mission_Update` case 4 (mission complete) 0x182c60:74 | 60.0f (0x42700000) | `FadeClr_147` — 0xff black on success, 0xff0000ff **red** on the fail arm [M 0x182c60:27,38] |
| `MP_Update` 0x184bd0:67 | 60.0f | 0xff |
| `Trigger_Update` 0x1b1a78:197 | 30.0f | 0xff |
| the quit/pause menu rows (P_ENDMISSION 0x216618:29 et al) | 80.0f | 0xff |

## 3. The music fade-out

`Music_Event(3,1)` [M 0x1de328] writes board slot 3 (`EventList` 0x2d34f8, slot k = +4k; slot 3 =
0x2d3504). `UpdateMusicalEvents` 0x114e70 consumes the board each frame and dispatches the level's
own updater — **but only while `MusicWaitForEnd` (0x30c8e0) is clear** [M disasm 0x1153c4-8: `lw
v0,-32144(gp)`; `bne v0,zero,0x115554` skips the whole per-level switch]. Once an end-wait stands,
the board never requests another section this level.

`UpdateMFX_03TowerA` 0x113558 (07000009), the shared exit block [M :9-33]:
`e[3] || e[8]` (the trigger exit / the mission-success pulse 0x182c60:31) → scan the MapMusic
record's +0x24 combat set for `MusicLastJump`: hit → jump `+0x14` (end_combat), miss → `+0x18`
(end_calm), **instant**; then `SFXFadeDown(0)` and `MusicWaitForEnd = 1`. The same block is in
every on-foot updater except 06's (`UpdateMFX_01CastleB` 0x113060 has none — its SFXFadeDown :37
rides the 0x6000071 NIS event): MayhewA-D 0x111ed8/0x112270/0x112730/0x112bf8 (01-04), TowerB/C
0x1137b0/0x113a20 (0a/0b), PowerStationA1/A2 0x113c18/0x113fc0 (0c/0d), Tower2A/B/C 0x14250/
0x14348/0x14640 (11-13), EvilBase/Silo/BaseC 0x14728/0x14a38/0x14b38 (14-16), Tower2Elevator
0x14528 (4a). (05 CastleA keeps its own copy inside its updater 0x112d78:104-115; 07/08/1b are
ported in music.rs.)

`SFXFadeDown` / `SFXFadeUpdate` [M SFX.IRX 0x10e0 / 0xf8c, docs/systems/sound.md §5.5]: the fade
is time-based — `FadeVolume = 10000 − 5·(t−t0)` ms, so a full bus fade lasts **2 s**. Flag 0 does
NOT copy `FadeVolume` into `FadeMusicVolume`, so **the SFX bus fades while the streamed music does
not follow the fade** [M ES_CombineVolumes 0x1300 vs ES_MusicSetVolume 0x8218; sound.md §5.5] —
the game's audible "music fades to black" is the outro marker playing out (then the stream ends)
over the fading world SFX and the 5 s screen fade; the next level's start restarts the track
[SFXStopMusic + SFXStartMusic, M 0x114e70:43-54].

The death/fail path stays separate [M 0x182c60:21-42]: ch 98 posts `Music_Event(6,1)`, sets
`FadeClr_147 = 0xff0000ff` (red), kills Bond (internal states 3→7→4) and ends in the mission-fail
MENU — no exit fade push, no level load; the board's e[6]/e[7] arms jump the record's on_death /
on_fail markers [M 0x114e70's arms; the port's music.rs e[6]/e[7]].

## 4. The port

- `nf-game/src/music.rs`: `Music::exit_block` — the shared exit arm for every previously unported
  level (01-04, 09-0d, 11-16, 4a), dispatched under the `MusicWaitForEnd` gate [M §3]. The
  07/08/1b handlers moved under the same gate (the decompile's dispatcher gate; their internal
  checks guard blocks the dispatcher already excludes).
- `nf-game/src/drone.rs`: `Drones::exit_fade_view()` — the exit hold as the game's viewer fade
  (`alpha = 1 − left/total`, black) for the draw code; the trigger exit's `ExitHold` (300) came
  from mayhew-end-transition. The mission-complete path now holds the 60-tick fade [M 0x182c60:74]
  when its `LevelEnd` targets an 0x0700 level (the host's own drain gate).
- `nf-view/src/scene.rs`: `Scene::script_overlay_hud` draws `view_fade` or the exit fade — one
  overlay, as the game draws one `psiFadeView` per viewer [M 0x1e6f50:203-209]; a push is aborted
  while a fade is in flight so the two never overlap [M 0x1c9f88].
- `nf-audio` (unchanged): `fade_down(false)` ramps the SFX bus over 2 s [M §5.5]; the music stream
  keeps its gain — the game's own split.

Deviations: the fade ticks on the fixed 60 Hz sim tick instead of the render frame (the game
advances it in `Camera_UpdateAll` on the render path — same 300 count at 60 Hz); the mission
path's ~240-frame mission-text display (0x182c60 case 6/7 `TimeOut`) is not modelled as a hold —
the port's 60-tick fade starts at the `LevelEnd` event.
