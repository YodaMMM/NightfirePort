# audit-nis-ops — the script (NIS) op census and the port's gaps

Branch `s/audit-nis-ops` (cut from origin/master b2cd660b3). Tool: `nf-extract nis-census <iso>`
(committed) — walks every type-7 entry's stream bytes with the game's own per-op lengths and dumps
one row per op-0x12 event site.

## 1. The census [D]

Every archive with type-7 entries on the disc: **60 archives, 5,263 streams** (26,824 commands),
every stream walked to its exact end with `Script_Run` 0x1d8548's per-op lengths — **0 truncated
walks, 0 opcodes outside the game's switch** (the default arm never fires on disc data). The 20
switch ops {04,05,06,07,09,0A,0C,0D,0F,12,13,14,15,16,17,18,1A,1B,1D,1E} all occur.

The 22 on-foot SP archives (`ResetMap_LevelCode2Img` 0x1d5258's story set + 07000041) carry
3,521 End / 59,830 Wait / 59,830 Marker / 3,660 Entity / 2,193 EntityEnd / 25 AnimStart /
23 AnimEnd / 18 CameraStart / 14 CameraEnd / 500 Event / 2 Fade / 4 Sprite / 326 Sound /
318 SoundEnd / 71 Light / 71 LightEnd / 128 SubScript / 126 SubScriptEnd **/ 0 Text**.

- **Op 0x1E (Text)** never appears in an on-foot SP level: all 81 sites live in 07000500 (79 —
  the intro-movie/LevelLoadingScreens pack) and 07000999 (2). The port's handler (the subtitle
  track + `Request::Text`) is exercised by `fix_script_commands_live`'s 07000500 test.
- **Op 0x13 (Fade)**: 2 SP sites — 07000001 and 07000006, one each.
- **Op 0x14/0x15 (Sprite)**: 4 SP sites (07000001 ×2, 07000004 ×2, no SpriteEnd — the sprite
  lives until the stream/script dies); 07900007 carries the 14/14 pair. The port renders the
  sprites (`scene.rs script_overlay_hud` → `hud::script_overlay`), pinned by
  `fix_script_commands_live` (07000001's letterbox bars are op-0x14 sprites).
- **Op 0x18 (Light)**: 71 SP sites; every kind byte on disc is 0 or 1, so the port's raw `kind:
  b2` equals the game's stored `b2 == 1` boolean [M 0x1cc3d8:23 + the census]. The initial rgb
  (255, 2, 3) is the game's own `Light_Create` arguments [M 0x1db1b0 call + 0x1cc3d8:19-21].
- **Event kinds** [D 851 sites]: 0x03 ×303, 0x04 ×36, 0x05 ×35, 0x06 ×31, 0x07 ×29, 0x08 ×1,
  0x09 ×1, 0x0A ×1, 0x0B ×23, 0x0C ×65, 0x0D ×195, 0x0F ×119, 0x11 ×10, 0x12 ×2. Kinds 0x0E,
  0x10 and 0x13 **never occur on the disc**. All 17 game kinds have port handlers; the drain's
  `_ => {}` only ever sees kinds the game's own switch also ignores.

## 2. What the audit found and FIXED (fades)

The port decoded and dispatched every op correctly (the fix-script-commands census stands), but
three fade behaviours were unmodelled or misgated:

1. **`Script_Play`'s ScriptCam latch [M 0x1db588:24-26]** — a camera-flag script's play sets the
   global `ScriptCam` to its hash; `Script_Update`'s near-end fade gate [M 0x1da998:44-79] reads
   it. The port carried `script_cam` as a per-script flag set only by event 0x13, which **never
   occurs on the disc** [D census] — so the near-end fade (`Camera_SetFade(40, 4, 0xff, 0)`
   every tick from `length − 20`) could never fire. Fixed: `Script::start` latches `script_cam`
   when the header camera flag is set; all 18 camera-flag SP scripts are non-looping [D census],
   so the latch cannot outlive its script.
2. **The skip's view fade-out [M 0x1dac78:49-57]** — `Script_FFwd` arms
   `Camera_SetFade(min(remaining, 45), 4, 0xff, 1)`: the screen fades to black over up to 45
   ticks while the scene keeps playing; the catch-up fast-forward starts only at black
   (Script_Update's loop guard), then the tail fades the gameplay viewer back in
   [M 0x1da998:101-110]. The port skipped the fade-out entirely. Fixed: `ffwd` pushes
   `Request::Fade { ticks: fade_len, viewer: 0, mode: 1 }`.
3. **The refuse gate + the completion model [M 0x1bbdd8:14, 0x1bada0]** — the port refused ANY
   second fade while one ran, and parked a completed fade at its end alpha forever. The game:
   the gate reads the fade TYPE byte (+0x234) — a type-0 fade (op 0x13, the near-end) never
   blocks, a re-arm overwrites it mid-ramp (which is why the near-end's every-tick re-arm holds
   the veil near 1/40 alpha); a type-1 fade refuses only on its own viewer. The state machine
   (+0x235): ramping 1 → parked 2 (one drawn tick) → idle 0, and an idle object draws nothing.
   Fixed: `Request::Fade` carries `mode`; `ViewFade` carries `mode` + `state`; the drain
   refuses only a type-1 re-arm on the same viewer; the advance runs unclamped with the
   strict [0,1] exit (a fade-in landing exactly on 0.0 holds — the observed parked 07000001
   intro fade). The script viewer's fade also now draws only while a script camera holds
   viewer 4 enabled [M 0x1d9860:71-76 `Camera_Enable(4,0,0,0)`] — the intro→story hand-off
   (07000004) has no camera-free tick, so the edge-based hand-back clear alone let the intro's
   ramp darken the story scene.

New tests `crates/nf-view/tests/nis_fade_lifecycle_live.rs` (real load + sim path, no
teleport): the near-end window of BOTH 07000004 camera scripts (0x06000083 500 ticks,
0x06000082 830 ticks — rate 1/40, mode 0, the re-arm veil < 0.1, the completion before the
story's window); the skip (mode 1, rate 1/45, the peak alpha ≥ 0.999, the tail's −22 fade-in on
the gameplay viewer, the story loads after the skipped intro).

## 3. The ops report table

| Op | SP count | Port status | Notes [M] |
|---|---|---|---|
| 04 End | 3521 | full | the hold-on-end / loop-restart / stop arms match 0x1db420 |
| 05 Wait | 59830 | full | |
| 06 Marker | 59830 | full | flag 2 at length [M 0x1d8548:43-51] |
| 07 / 09 Entity | 3660 / 2193 | full | the create arms, the base-compose rules and the hash-0 carve-outs are pinned by the walkreal batteries |
| 0A / 0C Anim | 25 / 23 | full | the frame tick, the op-1 sounds, the precache [M 0x1d9a08 case 2]; the footstep raycast's dust `Effect_Create` is **not modelled** (visual-only) |
| 0D / 0F Camera | 18 / 14 | full | the letterbox, the identity base, the HUD freeze |
| 12 Event | 500 | full (all 17 disc kinds) | the handlers match 0x1d94c0; kinds 0x0E/0x10/0x13 ported but never occur |
| 13 Fade | 2 | **fixed** | see §2 (1, 3) |
| 14 / 15 Sprite | 4 / 0 | full | rendered; killed with the stream |
| 16 / 17 Sound | 326 / 318 | full | the per-tick reposition + the finished-sound drop ride `script_sounds` [M 0x1d9a08 case 5] |
| 18 / 1A Light | 71 / 71 | full | the kind byte and the initial rgb are the game's own values [M 0x1cc3d8] |
| 1B / 1D SubScript | 128 / 126 | full | the sub ticks on its start tick [M 0x1d9a08 case 7]; the persistence condition matches 0x1db148 |
| 1E Text | 0 (SP) | full | 81 sites, all in 07000500/07000999; pinned on 07000500 |

## 4. Verification

- `cargo test --offline --release -p nf-game --lib`: **661 passed / 0 failed** (5 ignored).
- Fade-adjacent binaries: `doublecross_transitions_live` 6/0, `fix_cutscene_letterbox_live` 4/0,
  `fix_kimiko_handback_live` 1/0, `fix_script_commands_live` 4/0 (the parked-fade assertion
  updated to the completion model — the drawn result is identical), `movie_over_scene_live` 2/0,
  `nis_fade_lifecycle_live` 2/0 (new).
- The view_fade-touching walkreal batteries: `walkreal_07000001_live` 8/0, `walkreal_07000009_live` 8/0.
- **Not run** (the window did not fit): the rest of the walkreal battery and the nf-formats /
  nf-audio suites. The `Request::Fade` variant change touches every crate that compiles it —
  `cargo build --release -p nf-game -p nf-view -p nf-extract` is clean, and the full suite should
  be run at merge time. `nf-formats` was not modified.

## 5. Live checks for the main agent

1. **The near-end fade's visible weight** [M-derived]: during 07000004's intro (0x06000083) at
   clock 480..500, the type-0 re-arm should hold the veil at ~2.5% alpha (invisible). Poke
   nothing; watch `glb_viewer[4]` +0x228/+0x233 during the window and after the script stops
   (the ramp should never reach the screen — viewer 4 is disabled with the camera stream).
2. **The skip's black hold** [M-derived]: skip 07000004's intro mid-scene; the screen should
   fade to black over ≤45 ticks while the scene keeps playing, then the gameplay fades in over
   22 ticks. PCSX2: press the skip at ~5 s in and compare the fade timing.
3. **The intro→story hand-off** [M-derived]: 07000004's intro end should NOT darken the story
   scene (the port now gates the script fade on the live script camera).

## Usage

Context window, input/output tokens and cost are not visible to me in this environment (no
token or credit figures are exposed to the sandbox).
