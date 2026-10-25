## Your task: `level-4a-doors-and-cutscene`, 0700004a: lift doors go dark; the cut-scene's satchel and floor

Real play on 0700004a (Bond 3.84, -66.29, -1.86, yaw -1.578, frame 2592; and the earlier black frame at (3.61, -54.06, -2.58)): the large black shape is one of the lift/elevator doors — every lift door turns dark as the ride goes on. Separately, in the level's cut-scene (now that the cut-scene camera keys are right, branch origin/s/cutscene-camera-base): the satchel the guard is holding is missing, and Bond stands on nothing.

1. The doors: find how the lift doors are lit in the game [M — their glists' light flags, the cel ambient walk View_SetupRenderModes_Tweak 0x1e5888, whether they ride the cab (and the cab's light) or the shaft's cels] and why the port's go dark as the cab descends (an ambient sampled at the wrong position/cel, a moving object losing its cel). Fix; headless renders of the ride before/after.
2. The cut-scene: which actor holds the satchel (the guard's held-object/attachment in the script stream), why it doesn't draw, and what floor the game shows under Bond in that shot (a script entity, a cel only drawn by the script, a camera at the wrong height) [M + D]. Fix; renders.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-4a-doors-and-cutscene`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
