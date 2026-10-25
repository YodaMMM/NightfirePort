## Your task: `cutscene-4x3-strips`, cutscene side strips never show the gameplay camera

Master builds the cut-scene band at the viewer's aspect (16:9 with the AV widescreen option on, 4:3 off). With the option OFF on a 16:9 window, the 4:3 cut-scene band is drawn in the centre but the side strips still show the GAMEPLAY camera's view (07000004, `--bond -9.45,5.47,36.49,0.871,-0.135`, the Mayhew-computer cut-scene around frame 1836).

1. Make the strips outside the cut-scene band plain black during any script-camera frame (the band is the game's whole picture; nothing else is drawn there), with the option on or off. Check the setting path too: that the AV options' Widescreen row really sets the viewer aspect the band reads, through the real menu.
2. A test that during a script-camera frame nothing from the gameplay camera reaches the strips, for both settings; headless renders of the repro frame named in the report.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/cutscene-4x3-strips`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

## Lead (measured headless on 8de76e31a, check it)
The F12 frame shows the gameplay HUD and the first-person gun, so no script camera is live in it, yet the letterbox band + "Mission Complete" draw: the scene script overlay (`scene_overlay_up` -> `Scene::script_overlay_hud`, lib.rs ~2047-2089) runs on a different flag from `script_camera()`, so the band outlives the camera. Find the game rule for when the letterbox/overlay ends relative to the camera end [M Camera_Enable 0x1b9920, the script overlay ops] and tie them together.
