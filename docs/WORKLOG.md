# WORKLOG

## level-04-mission-complete-early (2026-10-25, sandbox expiring ~22:47 UTC)
Branch `s/level-04-mission-complete-early` off origin/master (24084ef2). Task: 07000004 (Double
Cross courtyard) — killing the ninja completes the mission ("Mission Complete" box, right), but the
outro cut-scene never plays and the HUD jumps to "NEW OBJECTIVE / Destroy Mayhew's computer" during
normal gameplay. Drive the real level with real input to the ninja kill; log per tick the mission
records' state, every objective write + HUD message (text id, requester), the script players that
should start (outro/next scene: trigger channels, states), and the level-end path [M mission
complete path, objective writers 0x1827d8 family, end-of-level cut-scene start (SP/script triggers
on the completion channel), GameFlow level-end state]. Fix so the kill leads into the game's outro
exactly as the game does; real-input test from the kill: outro script runs, gameplay HUD down
during it, level ends/transitions as the game's.
Plan: reuse sandbox work/ + ISO + nf-queue; branch pushed at start; fast suite runner in background
early; census the mission-complete path in the decompile (SP trigger on the completion channel ->
cut-scene start) and the 0x1827d8 objective writers; reproduce with a real-input drive test;
reproduce before fix; tests pin only [M]/[D]/[L]-evidenced behaviour; report + finish steps.

### level-04-mission-complete-early — progress (2026-10-25 ~22:15 UTC)
The kill→outro path mapped from the decompile: ch 0x54 → the #46 mux → ch 100 → `Mission_Update`
case 6 (banner 240, LevelToEndTo 0x0730004 [M 0x182c60:85-122]) → case 7 `GS_PausePlayer(1,0)` per
tick [M :123-130] → case 4 `ResetMap_LevelToLoad(0x0730004,0,0)` [M 0x1d3800] → the movie branch
(SP_Create(0x600095d) + state 6 + `psiStartBackgroundMovie(0x0730004)` [M 0x1d6360]) → phase 2 to
the hub 0x0700048 [M 0x1d3560] → the results page at the load tail [M 0x1d6360:160-163]. The level
has NO in-level outro SP [D two SPs only; both scripts' streams decoded — nothing watches 0x54/0x64].
FIXED: the case-7 hold freeze (the port let Bond walk/shoot under the banner — the user's "HUD and
gun still up"); the mission_hold input gate in scene.rs's step_bond. The "Destroy Mayhew's
computer" banner = the game's cold-boot case-0 announce queue [M 0x1827d8:41-54 + [L 0700000b], the
banner drains ~45 s — the early kill caught it mid-queue]; pinned: no 0x04000007 write on the kill
path. The slim image's 07300004.PSS is a 0-byte stub [D] — the state-6 pop is the game's own
BGFMVPlaying==0 arm there. New real-input test `the_kill_leads_into_the_games_outro` (the hold
pins, mission_end, the FE outro flow to the results). Green: walkreal_07000004_live (4), the
objective/carry/chain/cutscene batch (52), fix_death_loop_07/fix_mission/doublecross/reward/ptp/
level_end (26); nf-game+nf-formats+nf-audio 1091 passed / 4 failed (the slim-ISO disc-golden set:
disc_golden ×3 + identity_disc). The 806-binary suite: the build green (254 s), the run needs
~45-60 min foreground — this sandbox kills background jobs; the constraint + the covered bins are
in the report. Report: docs/reports/level-04-mission-complete-early.md.
