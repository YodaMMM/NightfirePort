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
