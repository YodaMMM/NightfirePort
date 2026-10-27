## Your task: `level-4a-cutscene-autoaim`, 0700004a: the level's cut-scene, and auto-aim while riding the lift

Two real-play reports on 0700004a (the Night Shift lift ride):
1. The level's cut-scene "is busted". Play the level from its start (fresh load, no --bond) and log the intro/scripted cut-scenes: which scripts run [D the level's SPs and the cut-scene pack], the camera per tick (position, fov — a near-zero fov key draws a flat void; a camera inside geometry draws nothing), the actors, the hand-back. Compare with the game [M Script_CameraStart 0x1d9140, Script_UpdateCamera 0x1db518, Script_Interp case 4 0x1d9a08, the actor entity streams] and fix what differs. Headless renders of the cut-scene frames named in the report.
2. "Auto aim doesn't play too well with shooting from the elevator": with Bond riding the moving cab, auto-aim tracks badly. Find the game's auto-aim with a moving shooter [M the auto-aim target pick and lead, Player_* aim assist, whether it uses the cab-relative or world velocity] and fix the port; a real-input test riding the cab and shooting a guard.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-4a-cutscene-autoaim`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-10-25T17:23Z

---
## Report (done, 2026-10-25T18:46Z)

