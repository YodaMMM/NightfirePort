## Your task: `level-04-mission-complete-early`, 07000004: killing the ninja completes the mission, but the outro cut-scene never plays and the objective jumps to "Destroy Mayhew's computer"

Real play on 07000004 (Double Cross, the courtyard; Bond near (-9.45, 5.47, 36.49)): killing the ninja is what completes the mission (the "Mission Complete" box) - that part is right - but the cut-scene that should follow does NOT play, and the HUD shows "NEW OBJECTIVE / Destroy Mayhew's computer" at the same moment, during normal gameplay (HUD and gun still up).

1. Drive the real level with real input to the ninja kill and log, per tick: the mission records' state, every objective write and HUD message (text id, requester), the script players that should start (the outro / next scene: their trigger channels, states), and the level-end path [M - the mission complete path, the objective writers (0x1827d8 family), the end-of-level cut-scene start (SP/script triggers on the completion channel), GameFlow's level-end state]. Find why the outro does not start and why the next objective is shown in gameplay.
2. Fix it so the kill leads into the game's outro exactly as the game does (the cut-scene plays; the next objective appears where the game shows it, not over gameplay); real-input test from the kill: the outro script runs, the gameplay HUD is down during it, the level ends/transitions as the game's.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-04-mission-complete-early`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox6 at 2026-10-25T20:43Z

---
## Report (done, 2026-10-25T22:19Z)

