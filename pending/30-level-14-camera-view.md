## Your task: `level-14-camera-view`, the security-camera monitors' camera view

On 07000014 (and every level with monitor consoles, kind 0x30 `Monitor_Create` 0x1b08b0 — docs/systems/cameras-turrets.md) the prompt "Press Action for Camera View" appears at a console, but pressing Action does nothing: the camera view is not ported.

Spots from real play (Bond root x, y, z, yaw, pitch; frame since load):
- the broken cut-scene: (40.87, 2.98, 40.85, -3.179, 0.210) at frames 1931 and 2300 — the cut-scene camera is inside rock, the rest of the frame a flat blue void;
- the camera-view console: (33.71, 0.65, -18.11, -7.579, 0.220) frame 9909 — "Press Action for Camera View" shows, nothing happens;
- the panels: (54.21, 1.03, 46.15, -8.344, 0.097) frame 5739 — a vent/hatch opening with nothing beneath it; (-88.58, -1.10, 46.15, -0.333, -0.041) frame 25538 — a wall panel with nothing behind it;
- the fall: (-49.65, 0.65, 36.85, -6.979, 0.006) frame 20889 (black screen) then (-107.07, -70.14, 73.28, -4.003, 0.595) frame 21799 — Bond below the map after a broken cut-scene, objective "Sabotage base security and escape from room".

1. Port the camera view [M — `Monitor_Activate` 0x1b0058 / `Monitor_Update` 0x1afcc8, the CameraList 0x319930 (`Sensor_Init` 0x1b0868 and `Sensor_Create`'s join at 0x1af010:135-137), the view switch to a sensor's camera (the camera matrix, FOV, the overlay `SecCamSprInfo` HUD panes already parsed in hud.rs), cycling cameras, leaving the view, the sensor's monitor-view hide latch SENSOR+0x60]. The draw shows the scene from the selected camera with the game's overlay.
2. Tests: Action at a console enters the camera view (the active camera = the console's first listed sensor), cycling moves to the next, leaving restores Bond's view; headless renders of the view named in the report.

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
