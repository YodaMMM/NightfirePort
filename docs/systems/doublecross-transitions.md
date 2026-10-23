# The Double Cross level transitions (mission 0x07000001, parts 07000001→02→03→04)

Playtest 1017: "all of the doublecross level transitions are missing." This doc is the chain and
the between-levels sequence with the evidence; the port now runs the real path on every hop.
Tags: **[M]** code (function + address + decompile line), **[D]** disc, **[L]** EE snapshot,
**[S]** suggested. Decompile files live in `work/decomp/ACTION.ELF/` (citation `0xaddr:lines`);
disc measurements were taken with `nf-extract` (raw dumps: the task's `/tmp` scratch, not committed).

## 1. The chain [D]

`MissionData` groups the story levels into missions [L ee_start_05.bin 0x2a4350, ported as
`nf_game::mission::LEVELS`]: **Double Cross = 0x07000001, parts 01 02 03 04, end level
0x07000004, outro 0x07300004** [M 0x182c60 case 6]. The disc's exit graph (`nf-extract
placements <iso> <L>.bin e8`, every 0xE8 param 0 on the disc scanned):

| level | exit (0xE8) | next (param 0) | gate [D] |
|---|---|---|---|
| 07000001 part 1 | #18 "Load Point" | 0x07000002 | auto ch 0x10 — Touch #180 (out 0x10, mask 0x18, filter skin 0x4F: Bond AND the contact on the plate) |
| 07000002 part 2 | #206 "Load Point 2" | 0x07000003 | key gate 0x4C (AND #480 = [0x48, 0x4b]) — E-key only (param 2 = 0) |
| 07000003 part 3 | #451 "Load Point" | 0x07000004 | key gate 0x52 (AND #435 = [0x4e, 0x51]) — E-key only |
| 07000004 part 4 | **none** | — | the mission ends in the mission layer (§5) |

**No wrapper/interlude level is in the chain.** Nothing on disc or in ACTION.ELF loads
0x07000041/43/46/49 [D: every 0xE8 param-0 + every kind-7 script's event-0x12 scanned — event
0x12 appears exactly twice, 07000006→0x07000007 and 0790004a's pack script →0x0720004a; the
decomp has no loader constant for any of them — the only ELF hits are the MapMusic table rows at
0x24fe0c-0x24fef0 and AnimDebugCreate's comparisons [M]]; `ResetMap_LevelToLoad`'s wrap switch
has no case for them [M 0x1d3800:27-101] and `ResetMap_LevelCode2Img` drops them into the
default arm [M 0x1d5258]. They are never-entered wrappers (43/49 are 77 KB, 2 placements [D]).

The exits are pure trigger chains: **zero event-0x11/0x12 in all four Castle script banks** [D
`nf-extract nis`] — no 07000005-style EndLevel script exists; the in-engine pieces of a hop are
the auto-play intro scripts of the NEXT level (§4), not an exit cut-scene.

## 2. The exit trigger's tail (every hop) [M]

`Trigger_Activate` 0x1b2318 type-10 (disasm-verified 0x1b24b8-0x1b250c; docs/systems/level-exit-fade.md §1):
`Music_Event(3,1)` → `GS_PauseGame(1)` → `ResetMap_LevelToLoad(next, 1, 0)` →
`GameFlow_PushState(300.0f, 7, 0xff)` (the 300-tick fade; a state-7 push runs
`Camera_SetFade(6, dur, colour, 1)` and pushes state 8 [M 0x1c9f7c-0x1c9f98]) → spent latch +
`Player_RamSave`. The key gate (param 1) locks the E-path when its channel is off [M :76-90].

## 3. The wrap and the loader phases [M]

`ResetMap_LevelToLoad__FUiScSc` 0x1d3800 (param 3 = no_intro; the exits pass 0 [M :111 — the
only no_intro=1 callers are P_ENDMISSION's restart rows 0x216618:16,20]): after the DRIVING
branch and the MP arm, the **wrap switch** [M :27-101; jump table 0x305030, 74 slots for
0x07000001-0x0700004a, 24 `lui 0x710`/`ori` pairs]:

- **0x07000001→0x07100001, 02→0x07100002, 03→0x07100003, 04→0x07100004** — identity on the low
  byte through 0x07000016 [M :29-94];
- 0x0700001b → 0x07100018 (the Equinox briefing is movie 18; 0x0710001b is not on the disc [D
  /MOVIES/30_FPS]) [M :95-97]; 0x0700004a → 0x0710004a [M :98-99];
- every other code (the wrappers, the front end 0x07000048, the 0x0720/0x0730 movie codes)
  loads unwrapped. Tail: `GameState+8 = wrapped; GameFlow_PushState(0, 3, 0xff)` — the loader
  (state 3) is pushed at trigger time, under the fade [M :102-103].

`ResetMap_Load__Fv` 0x1d6360 phases: **phase 0** loads the common stage pack 07000500, runs the
script-hash switch (0x7100001→0x600095a, 05→0x6000961, 09, 0c, 11, 14, 18; the 0x73 outros —
**0x7100002/03/04 have no case** [M :40-99]) and pushes **state 6** + starts the background
movie [M :101-111]; **phase 2** resolves (`GetLevelToLoadFromMovieCode` 0x1d3560), **phase 3**
builds the loading screen and loads.

`UsingMovie__FUi` 0x107eb8's whitelist (23 codes, ported as `nf_view::movie::USING_MOVIE`)
accepts the briefings **0x07100001/05/09/0c/11/14/18** and the outros 0x07300004/08/0b/0d/13/17/1b
[M :8-85]. **0x07100002/03/04 are NOT whitelisted**: `psiStartBackgroundMovie` 0x1081a8 returns
without touching `BGFMVPlaying` [M :29-30], so state 6 pops on its first tick [M 0x1c98c8 case 6
pops when `BGFMVPlaying == 0`]. **The game plays no movie on a Castle hop.**

`GetLevelToLoadFromMovieCode__FUi` 0x1d3560: 0x72 link-movie cases (0x7200001→0x7100002 …
0x720004a→0x7100013 [M :6-95]), the 0x73 cases → 0x07000048 [M :24-50], default =
`GameState+8 & 0xff00ffff` (strip the middle byte) [M :97-98]. So the wrapped hop
0x07100002 → **0x07000002**: the straight load stands.

**Net sequence on a hop**: exit trigger → end sting + pause → 300-tick fade to black →
(wrapped load, no movie, no state-6 hold) → loading screen → archive load → the entry fade
arms (§7: on the player's viewer — DRAWN only when no script camera owns the view, so the
cut-scene openings show no fade-up; part 3's camera-free load does) →

## 4. The next level's intro [M/D]

- **The 45-tick fade-in from black** [M `Player_Init` 0x1910f8:45 — `Camera_SetFade(player,
  -45.0f, 0xff, 1)`, 0xc2340000 = −45.0 in f12; negative rate = fade-in, alpha starts 1.0 and
  falls 1/45 per tick [M `Camera_SetFade` 0x1bbdd8 disasm 0x1bbe8c-0x1bbefc; the rate math in
  docs/systems/level-exit-fade.md §2]]. `ResetMap_Load` phase 3 runs `Player_Start` on every
  level load [M 0x1d6360:152], so the fade arms before the first gameplay frame (the loading
  screen ends at phase 3's tail [M :169-170]).
- **The auto-play intro scripts** [D]: each part's mode-1 ScriptPlayer plays at load — part 1
  SP #277 (0x06000087, pack 07800001; writes ch 0xB — the contact's enable), part 2 SP #400
  (0x06000084), part 3 — (mode-0/64 players only: 0x06000563/0x06000080 wait on triggers),
  part 4 SP #3 (0x06000083). Ported since fix-cutscenes (`load_with_carry` merges the type-0xC
  packs; docs/systems/levels.md §5).

## 5. part 4's end (the mission layer, not a level exit) [D/M]

part 4 has **no 0xE8** [D §1]. The end chain: intro 0x06000083 (500 ticks, mode 1) fires
`0xD [0x38,1]` → SP #2 plays 0x06000082 (830 ticks), whose last event `0xD [0x53,1]` enables
the mission record's only objective "Defeat the assassin" (channel 0x54, enable 0x53) [L
mission/c07000004.rs]. The assassin is the level's only drone — "oDrone" placement 1, **death
channel param 12 = 0x54 = the objective's channel** [D placement probe]. The kill raises ch
0x54 → objective complete → success ch 100 → `Mission_Update` internal 6 sets `LevelToEndTo =
0x0730004` [M 0x182c60 case 6] → after the 240-tick banner, internal 4 posts
`Event::LevelEnd{0x07300004}` [M case 4]. The outro movie 0x07300004 plays (whitelisted), then
phase 2 resolves it to the front end 0x07000048 with the results page queued (0x40000036) [M
0x1d6360:173-177] — the end sequence is pinned by `reward_after_outro`/mission-movies (docs/systems/mission-movies.md).

## 6. The port

- **The wrap on the exit path** — `nf_game::triggers::wrap_intro` (the 0x1d3800 switch verbatim);
  the window's drain (`nf-view/src/lib.rs`, the `level_exit` arm) now routes the exit through
  wrap → `resolve_exit_code` (the compressed phase-2 resolve that already lived there). On a
  Castle hop the wrapped code is not whitelisted, so nothing holds and the raw part code loads —
  the same net code as before, now by the game's route. Deviation: a trigger exit whose wrap
  hits `USING_MOVIE` would start a briefing movie in the game; **no retail exit does** [D §1 —
  the 0xE8 param-0s are 02/03/04/06/07/08/0a/0d/12/4a/15/07300013], so the port resolves it
  straight through and logs [S — live-check requested: poke a warm exit to a briefing'd code in
  PCSX2 and watch state 6 + the movie].
- **The 45-tick entry fade** — `Drones::spawn_fade` (`(left, total)`, armed `Some((45, 45))` at
  every level load — the port's `Player_Init` stand-in; the game arms it in `Player_Init` on
  every `Player_Start` [M §4]), ticked in `Drones::tick` (the exit hold's documented deviation:
  the game advances its fade on the render path), read by the draw code through
  `Drones::spawn_fade_view()` — the same `ViewFade` the script fade and the exit fade produce.
  `Scene::script_overlay_hud` chains it last: script fade → exit fade → entry fade — one
  overlay, as the game draws one `psiFadeView` per viewer [M 0x1e6f50:203-209]; a script that
  owns the fade during an intro overwrites the same object in the game [S: the port keeps the
  three sources separate and prefers the script's].
- The loading screen (Double Cross art 0x030000d2, the part's objective label 0x4000006/8/a/d,
  the hint tables) is the existing `nf_view::loading` mirror of `ResetMap_LevelCode2Img` [M
  0x1d5258]; the exit hold + fade + sting are mayhew-end-transition/level-exit-fade (unchanged).

## 7. Tests

`crates/nf-view/tests/doublecross_transitions_live.rs` (ISO-gated where they read the disc, each
with a sister that can fail):
1. part 1's real route (intro → contact → the AND-touch plate) fires the exit and the sequence
   runs: exit 0x07000002 → wrap 0x07100002 → not whitelisted → resolve 0x07000002 → the Double
   Cross loading group → the entry fade falls 1.0 → 0.0 over 45 ticks. Sister: Bond alone on the
   plate fires nothing (the AND needs the contact).
2. part 2's exit: key gate 0x4C locked (E does nothing), gate on → E loads 0x07000003 through
   the same wrap/resolve; entry fade on 07000003. Sister: locked E fires no load.
3. part 3's exit: gate 0x52 → E loads 0x07000004 through the wrap/resolve; loading group +
   objective label. Sister: without the gate the exit stays locked.
4. part 4: the intro chain enables the assassin objective (ch 0x53 via the two scripts), the
   kill raises 0x54, the mission ends to `LevelToEndTo 0x07300004`, resolved to the front end.
   Sister: a non-end level maps to the default arm (0x07000048, no outro) — pinned on 0x07000011.
5. The table pins: wrap switch rows for the whole Double Cross row set + the unwrapped codes;
   the movie-less wrap keeps the straight load (no `USING_MOVIE` hit) — the sister for "a level
   that has none keeps its straight load"; no Double-Cross exit references the wrapper codes
   0x07000041/43/46/49 [D].

## 7. The entry fade is on the player's viewer, not the cut-scene (doublecross-transitions-2)

The main agent held 779's merge on a render comparison (2026-10-19): the port faded 07000002's
opening cut-scene up from black over 45 frames; the PS2 (GDB build, the level loaded through
goto_level.py's push = `ResetMap_LevelToLoad(0x07000002, 0, 1)`, captures timed by
GameState+0x34) shows the loading screen at +1/+2 and the same cut-scene at FULL brightness
from +3 (mean 19.4 at +3 → 21.2 at +30, drift only). Both are right about their own object —
the fade exists, but the port drew it on the wrong viewer:

1. `Camera_SetFade` 0x1bbdd8:9 arms the fade on `(&glb_viewer)[param_2]` — one fade object PER
   VIEWER (`+0x228` alpha, `+0x22c` rate, `+0x230..33` RGBA, `+0x234/35` state). `Player_Init`
   0x1910f8:44-45 passes the PLAYER's viewer id (`Camera_Enable(param_1, 1, 1, 0)` right
   before) — viewer 0 in single-player.
2. `Script_CameraStart` 0x1d9140 — the cut-scene camera — runs `Camera_Enable(v, 0, 0, 0)` for
   v 0..6 and `Camera_Enable(4, 1, 1, 0)`: the gameplay viewers' draw flag `+0x2a` goes 0 [M
   Camera_Enable 0x1b9920:31-32 stores param_2→+0x29, param_3→+0x2a] and viewer 4 (the script
   camera's) is drawn. `Game_Draw` 0x1e6f50:186-212 draws each DRAWN viewer's sprites and then
   its own `psiFadeView(+0x230..33)` only when that viewer's `+0x235` is set — viewer 4's fade
   bytes are never armed by Player_Init, so **no fade over the cut-scene** [the L capture].
   (Script fades DO arm on viewer 4 — `Script_FadeStart` 0x1db3d0:10 `Camera_SetFade(-f, 4,
   0xff, 0)` — the port's existing `view_fade`.)
3. The raw state keeps advancing: `Camera_UpdateAll` 0x1bbd68 → `Camera_UpdateGlbVars`
   0x1bada0:16-35 (`+0x235 == 1`: `alpha += rate · FRAME_RATE_MUL`, → 2 → 0) runs
   unconditionally in `Game_Run` 0x1c9790:43 every game tick, cut-scene or not; [L]
   FRAME_RATE_MUL = 1.0 (ee_snapshot 0x0030d0d8 = `0000803f`), so the -45 arm is exactly 45
   ticks. `Camera_PushStates`/`PopStates` 0x1bbc60/0x1bbcd0 snapshot only +0x29/+0x2a, so no
   cut-scene end resurrects it — by the time gameplay returns the fade is expired (a cut-scene
   shorter than 45 ticks would hand the remainder back to the player viewer; none of the
   Castle intros is that short [D §4 — 500/830-tick scripts]).
4. Where the game DOES draw it: a load with no auto-play intro. part 3 (07000003) has
   mode-0/64 script players only [D §4], its port sim runs no script camera for the whole
   window, so viewer 0 stays drawn and the fade-up shows for its 45 ticks [M 0x1e6f50 draws
   every +0x2a viewer's fade]. part 1/B/D open under their auto-play intros (SP #277/#400/#3
   [D §4]) — hidden, like 02.

**Port shape (unchanged draw code)**: `Drones::spawn_fade` still arms (45, 45) at every load
and ticks 1/tick; `spawn_fade_view()` now returns None while a script camera owns the view
(`scripts.camera().is_some()`) — the getter the draw overlay reads (scene.rs chains script →
exit → entry). Tests pin both halves: `doublecross_transitions_live::assert_entry_fade_runs`
walks 46 ticks per part — the raw ramp exactly (45−n, 45) with expiry at 45, the view None
under the cut-scene camera (01/02/04, the PS2-pinned case) and the exact drawn ramp on
part 3 (the game-draws-it case).
