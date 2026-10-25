## Your task: `level-14-cutscene-door`, 07000014: the door in the control-room cut-scene stays shut, Kiko walks through it

With the cut-scene camera fixed (branch `origin/s/cutscene-camera-base`: the camera's world matrix is the keys alone [M Script_CameraStart 0x1d9140]), the 07000014 control-room scene at Bond (40.87, 2.98, 40.85) now shows the room, but a door in it never opens: it stays static while Kiko walks straight through it. Find the door's driver in that scene [D the script 0x06000833 streams and the door placements nearby; M — a script entity/anim stream on the door object, a Door_* channel the script writes (event 0xD), or Door_Activate from the actor's path]. Port what is missing so the door opens as Kiko reaches it; a test that the door's state/pose changes during the scene before she crosses its plane; headless renders of the moment named in the report.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-cutscene-door`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-10-25T18:49Z
