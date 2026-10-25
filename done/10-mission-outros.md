## Your task: `mission-outros`, the end-of-mission cut-scenes do not play (Night Shift 4a, Double Cross 04)

Two real-play reports on the current build:
- 0700004a (the end of Night Shift): "end of level cut-scene played but end of mission cut-scene didn't play".
- 07000004 (the end of Double Cross): killing the ninja completes the mission ("Mission Complete" box), but the cut-scene that should follow does not play, and "NEW OBJECTIVE / Destroy Mayhew's computer" shows over gameplay instead. (Task 10-level-04-mission-complete-early covers 04's objective text; coordinate: this task owns the outro.)

1. Drive both levels' endings with real input and log GameFlow from the completion: the mission-complete path, the level-end vs mission-end distinction, what the game plays at a mission's last level (the outro NIS / the outro movie / the debrief) [M — GameFlow's level-end states, the mission records' end flag, the outro script/movie start, the movie table], and what the port does instead.
2. Port the mission-end path so each mission's outro plays as the game's does after its last level (and the level-only end plays on the others); real-input tests for both endings (the outro's script/movie starts, gameplay HUD down).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/mission-outros`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox6 at 2026-10-25T22:23Z

---
## Report (done, 2026-10-25T22:56Z)

