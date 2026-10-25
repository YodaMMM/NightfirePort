## Your task: `level-04-mission-complete-early`, 07000004: "Mission Complete" shows as the last objective is given

Real play on 07000004 (Double Cross, the courtyard; Bond near (-9.45, 5.47, 36.49), around frame 1836): the screen shows "NEW OBJECTIVE / Destroy Mayhew's computer" at the top AND a centred message box "Mission Complete" at the same time, during normal gameplay (HUD and gun up). The mission is not complete — the computer has not been destroyed.

1. Drive the real level with real input to that point and log the mission records' state, the objective writes, and every message the HUD's message box shows per tick (the text id, who requested it). Find who requests "Mission Complete" there [M — the mission/objective handlers (the 0x1827d8 objective writer family, Mission_* complete paths), the script events that print messages, the level's mission records [D]] and why the port fires it now (a premature complete condition, an objective counted done at creation, a script message mis-mapped).
2. Fix it so the message shows only when the game shows it (after the computer is destroyed / the real completion); real-input test: at the objective handout no "Mission Complete" message; after the real completion it appears.

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
