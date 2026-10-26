## Your task: `level-14-camera-view`, "Press Action for Camera View" works on 07000014

On 07000014 the security consoles prompt "Press Action for Camera View" but pressing Action does nothing. Find the console's activation chain [D placements near (33.71, 0.65, -18.11) + their triggers/scripts] and the game's camera-view mode it enters [M: the ThirdCam / security-camera viewer path - ThirdCam_Create 0x1bc510, ThirdCam_GetPos 0x1bc590, the Player_Activate arm that switches the view, and how it exits], then port the sim side: Action at the console switches the active view to the camera the game picks, input stays as the game routes it while viewing, and the view returns the way the game returns it. Expose the active view camera (position/orientation per tick) for the renderer; the overlay/draw side is done separately. Tests: Action at the console enters the camera view with the game's camera id [M + D]; the exit restores the player view.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-camera-view`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-26T19:32Z
