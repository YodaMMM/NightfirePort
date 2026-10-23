# The mission movies (every SP row's briefing and outro through the window's own path)

**Port status: EXTENDED (mission-movies-e2e, 2026-10-19).** The briefing timeline was pinned for
The Exchange only (briefing_plays_live); this doc covers EVERY single-player
campaign row: which rows wrap to a briefing/outro movie, the state-6 hold, the worst-case-clock
contract, and the no-movie rows. Facts are tagged [M] code (function + address + decompile line),
[D] disc, [L] EE snapshot (address + value), [S] suggested.

## 1. The two switches that decide a row's movies

**The briefing wrap** — `ResetMap_LevelToLoad__FUiScSc` 0x1d3800 wraps a story level to its 0x71
briefing code when param_3 (no_intro) is 0: identity on the low byte for 0x0700001..0x07000016,
plus exactly two special cases — 0x0700001b → 0x07100018 (the Equinox briefing is movie 18;
0x0710001b is not on the disc [D /MOVIES/30_FPS]) and 0x070004a → 0x071004a. The switch is
compiled code — 22 `lui rt,0x710` words at 0x1d392c-0x1d3a40, no jump table [L ee_snapshot scan].
The DRIVING branch comes FIRST [M 0x1d3800:104-108]: a 0x09xxxxxx hash pushes GameFlow state 9
(the DRIVING.ELF handover, `psiLaunchDriving` 0x120af0) with GameState+0x8 = the hash and NO wrap,
no movie — the driving rows never enter the loader's movie phases.

**The outro switch** — `Mission_Update__Fv` 0x182c60 case 6 maps the level the mission ended on
to `LevelToEndTo` (GameState global 0x30cbac): its own 0x73 code for 0x0700004/08/0b/0d/0f/13/16
(0x16 → 0x7300017 is the one non-identity entry) and 0x070001b, the front-end map 0x0700048 for
everything else. The bytes 0x182e60-0x182eb4 are lui/ori pairs, no table [L ee_snapshot].

**Which level ends a mission** — the `MissionData` records group the story levels into missions
[L ee_start_05.bin 0x2a4350, ported as `nf_game::mission::LEVELS`]: the parts are

| mission | parts in order | end level | outro |
|---|---|---|---|
| 0x07000001 Double Cross | 01 02 03 04 | 0x07000004 | 0x07300004 |
| 0x07000005 The Exchange | 05 06 07 08 | 0x07000008 | 0x07300008 |
| 0x07000009 Night Shift | 09 0a 0b | 0x0700000b | 0x0730000b |
| 0x0700000c Chain Reaction | 0c 0d | 0x0700000d | 0x0730000d |
| 0x0700000e (no campaign row) | 0e 0f | 0x0700000f | 0x0730000f — NOT whitelisted |
| 0x07000011 Phoenix Fire | 11 12 4a 13 | 0x07000013 | 0x07300013 |
| 0x07000014 Countdown | 14 15 16 | 0x07000016 | 0x07300017 |
| 0x0700001b Equinox | 1b | 0x0700001b | 0x0730001b |

The switch's keys are exactly the eight end levels [M]; completing on a NON-end part (early
completion, e.g. on 0x07000011 itself) hits the default arm → the front-end map, no outro [M
0x182c60:93-95; pinned by reward_after_outro's sister].

## 2. The campaign rows and their movie codes

`sp_level` 0x2df2e0, 12 × 0x18 bytes [M `C_SBNFMAP` 0x20b8d8:18-21; values in menu.rs `CAMPAIGN`]:

| row | mission | start level | briefing | end level | outro |
|---|---|---|---|---|---|
| 0 Paris Prelude | driving | 0x09000001 | none — state 9 handover | — | none |
| 1 The Exchange | 0x07000005 | 0x07000005 | 0x07100005 | 0x07000008 | 0x07300008 |
| 2 Alpine Escape | driving | 0x09000005 | none | — | none |
| 3 Enemies Vanquished | driving | 0x09000006 | none | — | none |
| 4 Double Cross | 0x07000001 | 0x07000001 | 0x07100001 | 0x07000004 | 0x07300004 |
| 5 Night Shift | 0x07000009 | 0x07000009 | 0x07100009 | 0x0700000b | 0x0730000b |
| 6 Chain Reaction | 0x0700000c | 0x0700000c | 0x0710000c | 0x0700000d | 0x0730000d |
| 7 Phoenix Fire | 0x07000011 | 0x07000011 | 0x07100011 | 0x07000013 | 0x07300013 |
| 8 Deep Descent | driving | 0x09000002 | none | — | none |
| 9 Island Infiltration | driving | 0x09000003 | none | — | none |
| 10 Countdown | 0x07000014 | 0x07000014 | 0x07100014 | 0x07000016 | 0x07300017 |
| 11 Equinox | 0x0700001b | 0x0700001b | 0x07100018 | 0x0700001b | 0x0730001b |

**The no-movie rows, listed:** the five driving rows (0x09000001/05/06/02/03 — their codes are
not in `UsingMovie` and never wrap; the game hands over to DRIVING.ELF in state 9 [M
0x1d3800:104-108]; the port, which has no DRIVING.ELF, shows its driving notice instead and never
requests a load [S port choice]); mission 0x0700000e's end 0x0700000f →
0x0730000f, which is IN the outro switch but NOT in the whitelist, so its "outro" exits state 6
on frame 1 [M 0x107eb8 + 0x1082e8 — the explicit routing entry exists for exactly that]; and any
non-end part through the default arm (the map, no movie at all).

## 3. The whitelist and the player

`UsingMovie__FUi` 0x107eb8 — the 23 hashcodes `psiStartBackgroundMovie__FUiScUi` 0x1081a8
accepts; it returns without opening anything for any other code, and `strFileOpen`'s failure path
would hang, so the gate keeps state 6 short for codes with no file. Briefings 0x07100001/05/09/0c
/11/14/18; outros 0x07300004/08/0b/0d/13/17/1b; the specials 0x07330048/0x07350048/0x07360048
(not on the disc)/0x07370048/0x07380048/0x07390048/0x073a0048/0x073b0048/0x073f0048 [M 0x107eb8;
ported verbatim as `movie::USING_MOVIE`]. The game plays from `cdrom0:\movies\30_FPS\%8.8x.pss;1`
[D]. `BGFMVPlaying` lives at 0x30c69c [L]; `psiMovieFinished` is `BGFMVPlaying == 0` [M 0x1082e8].

## 4. The loader phases and the state-6 hold

`ResetMap_Load__Fv` 0x1d6360: phase 0 records GameState+0x10 = the movie code for the 0x71/0x73
wrapped codes, starts the movie (`psiStartBackgroundMovie`, `MovieVolume` = the site's SFX
volume) and pushes GameFlow state 6 with no frame budget of its own [M 0x1d6360:101-111]. Phase
2 resolves with `GetLevelToLoadFromMovieCode__Fv` 0x1d3560: 0x07100018 → 0x0700001b is the one
0x71 special case, the enumerated 0x73 codes route to the front-end map, everything else strips
the middle byte; when the resolve lands on the map, GameState+0x1c = 0x40000036 (the results
page) [M 0x1d6360:173-177]. Phase 3 loads; its tail runs `MenuManager_Create` — the queued
results page opens only there, i.e. AFTER the outro [M 0x1d6360:160-163].

`GameFlow_Main__Fv` 0x1c98c8 case 6 holds while `BGFMVPlaying` is set and pops when it clears
(psiMovieFinished) or the skip action 0x19 newly presses (→ `psiStopBackgroundMovie` → pop) [M
0x1c98c8 case 6, Input_Action(-1, 0x19, 1); 0x1082e8]. With no real player (the headless
countdown model) the pop rides the frame countdown instead.

**The decode-ahead.** The port starts the player whenever a briefing is pending — during the
84-frame fade the menus push ahead of the load (`Camera_SetFade` push 60.0/84.0 [M disasm
0x183054-60, 0x20ba50-5c]; the abort-while-fading guard 0x1c9f88/0x1c9fa8) — so state 6 opens on
its first frame with BGFMVPlaying already 1. Without it, one redraw's worst-case 8 front-end
ticks could carry the flow from the fade through phase 0's state-6 push to its pop before the
host ever saw ST_MAPLOAD — the briefing skipped (The Exchange, after the
gameflow-state-stack change; fixed by starting the decode ahead of state 6 whenever a
briefing is pending).

## 5. The window's own path (what the tests drive)

The front end walk: boot → the title → START → P_ESTHERO → Main → NightFire → Play without
saving → the box → difficulty → the Single Player list (`C_SBNFMAP`) → a row's Confirm. The
confirm reads sp_level+0x0c and calls `ResetMap_LevelToLoad` [M 0x20b8d8:21-24]; C_SBNFMAP pushes
the 84-frame fade AFTER the load request [M 0x20ba50-5c]. The window's `RedrawRequested` order is
the movie glue first, then up to 8 front-end ticks (the worst-case clock the live tests run).

The end sequence: the sim's mission layer reports the banner end as Event::LevelEnd{next =
LevelToEndTo} [M 0x182c60 case 4's ResetMap_LevelToLoad + GameFlow_PushState(60.0, 7, …); port:
nf-game/src/mission.rs state 6 → internal 4]; the window's handler (`mission_ended`, the same
call the App makes from `pending_mission_end`) captures the ended level, queues GameState+0x1c =
0x40000036 [M 0x182c60:72] and requests the load, then pushes the 60-frame fade [M
0x183054-60]. A FAILED mission (MissionState 2/3) opens the 0x40000042 fail menu OVER the paused
mission instead — no load request, no movie, nothing loads until the menu's own row [M
0x182c60:77-80, P_ENDMISSION_Handler 0x216618].

The port's glue contract: `nf_view::mission_movie_glue` runs once per redraw before the front-end
ticks; in state 6 it feeds `movie_playing` from the one BGFMV slot and starts `movie_code` when
the slot is empty [M 0x1081a8 replaces the slot's content]; while a movie-wrapped load is pending
it starts the decode-ahead and yields the slot from any looping page backdrop [M
psiStartBackgroundMovie replaces the playing movie]. Tests: `crates/nf-view/tests/
briefing_plays_live.rs` (every row, `--features nf-view/movies`), with the headless state machine
pinned by `movie_playback_flow.rs` and the end-order by `reward_after_outro.rs`.
