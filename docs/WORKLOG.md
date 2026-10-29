# WORKLOG

### level-01-end-fade (started 16:40 UTC, 2026-10-29)

Task: the level-end screen fade. The port cuts straight to the next level when the exit
trigger fires (reported on 01, Double Cross Part 1). Find the game's level-end fade
([M] Trigger_Update 0x1b1a78 case 10 / ResetMap_LevelToLoad, the Fade_* / HUD fade pane it
starts, its length and colour), port it for every on-foot exit that uses the same path, and
pin it: on the 01 real-input walk's exit the fade starts on the exit tick, runs the game's
frame count, and the next level loads after it. Full suite, quote the total line.

Plan: branch pushed first; then the decompile read (Trigger_Update case 10, Camera_SetFade
0x1bbdd8, ResetMap_LevelToLoad 0x1d3800, FadeClr.147 0x30cbc0), the port change in nf-game,
the pin test on the 01 walk, chunked full suite, report.

Done (16:40-17:42 UTC): the read pinned the whole chain [M]: `Trigger_Update` case 10 (0x1b1a78:162-166)
→ `Trigger_Activate` type-10 tail (0x1b2318:89-92, disasm 0x1b24b8-0x1b24ec: `Music_Event(3,1)`,
`GS_PauseGame(1)`, `ResetMap_LevelToLoad(next,1,0)`, `GameFlow_PushState(300.0f,7,0xff)`) → the push
converts state 7→8 through `Camera_SetFade(viewer 6, 300.0, 0xff, mode 1)` [M 0x1c9f38 disasm
0x1c9f7c-84: a0=6, f12/a1 passthrough, a2=1] arming level 0, rate (1/300)·(0xff/255), colour
R=G=B=0 black [M 0x1bbdd8 disasm 0x1bbe20-54]; `Game_Draw` draws psiFadeView per active viewer
[M 0x1e6f50 passes 0..4 and 0..10]; `Camera_UpdateGlbVars` advances + clamps [M 0x1bada0:16-27];
state 8 pops onto state 3 and the level loads [M 0x1c98c8:173-189]. The sim side was already on
master (mayhew-end-transition's ExitHold, 300 frames, both Trigger_Activate arms + the mission
60-frame arm). The MISSING piece was the draw: `script_overlay_hud`'s chain ran
`view_fade.or(exit).or(spawn)` and 07000001's parked intro/meet script fade-in (alpha 0, never
cleared — the hand-back clears only a viewer-0 fade) shadowed the exit fade — the game's fade
object is PER VIEWER (exit = viewer 6 [M a0=6], script op-0x13 = viewer 4), a parked pane draws
byte 0 and cannot cover the exit pane, so the chain is now exit → script → entry (scene.rs, the
one-line fix + the per-viewer cite). Pinned on the 01 real-input walk
(`walkreal_07000001_live::the_exit_fade_starts_on_the_exit_tick_runs_the_games_300_frames_then_the_level_loads`):
hold 300/300 + alpha 0 at the fire, world + clock frozen, 200 left at +100 with the pane black at
alpha 100/300 in the drawn HUD, cleared at 300 with the request intact, 07000002 loads warm at
start #0. All 5 tests in the file green (the walk body became the shared `walk_to_the_exit`
helper, `stop_at_fire` keeps the walk's mission-monitor block identical).
