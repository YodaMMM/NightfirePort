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
