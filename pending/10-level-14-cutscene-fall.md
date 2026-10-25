## Your task: `level-14-cutscene-fall`, 07000014: the broken cut-scene camera and the fall below the map after it

Spots from real play (Bond root x, y, z, yaw, pitch; frame since load):
- the broken cut-scene: (40.87, 2.98, 40.85, -3.179, 0.210) at frames 1931 and 2300 — the cut-scene camera is inside rock, the rest of the frame a flat blue void;
- the camera-view console: (33.71, 0.65, -18.11, -7.579, 0.220) frame 9909 — "Press Action for Camera View" shows, nothing happens;
- the panels: (54.21, 1.03, 46.15, -8.344, 0.097) frame 5739 — a vent/hatch opening with nothing beneath it; (-88.58, -1.10, 46.15, -0.333, -0.041) frame 25538 — a wall panel with nothing behind it;
- the fall: (-49.65, 0.65, 36.85, -6.979, 0.006) frame 20889 (black screen) then (-107.07, -70.14, 73.28, -4.003, 0.595) frame 21799 — Bond below the map after a broken cut-scene, objective "Sabotage base security and escape from room".

1. The cut-scene at (40.87, 2.98, 40.85): find which script plays there (the trigger/SP, its camera stream) and why the port's camera sits inside rock with a void around it [M — the script camera's base compose (`Script_UpdateCamera` 0x1db518, the SP's base matrix, the parent chain), the camera key interpolation]; fix it.
2. The fall: after the cut-scene near (-49.65, 0.65, 36.85) Bond ends below the map at y -70. Find the cut-scene's hand-back (the marker entity that re-enables Bond, `Player_Enable` 0x195498 — the matrix origin placed directly [M :57-68]) and why the port's hand-back pose lands off the floor; fix it [M].
3. Real-input tests from the trigger to the hand-back: the camera stays in the room, Bond is back on the floor, the walk continues.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-cutscene-fall`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
