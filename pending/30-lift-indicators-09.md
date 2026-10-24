## Your task: `lift-indicators-09`, 07000009's garage lifts: the floor indicators above the doors and the call buttons are not drawn

At `--bond -3.45,-375.98,20.61,-6.124,-0.083` on 07000009 (the parking-garage lift lobby), the real game [L fresh boot, PCSX2] draws a floor-indicator panel showing "L" above EACH of the two lift doors; the port draws neither, and the lifts' call button is missing too. No placement on map 39 within x -14..8, y -380..-368, z 18..34 is the indicator or a door (only OBJECT_Lights, a drone, an alarm switch player #263, an AI point), so these are script-created entities, objects on another map, or parts of the lift SS objects.

1. Identify each missing piece: the indicator panels (and their "L" display — an animated texture or a script-driven frame), the call buttons. Where they come from on the disc (map/placement or the script entity and its stream) and how the game creates and draws them [M + D].
2. Make the port create and draw them where the game does (data/creation only — if a draw-path change is needed, keep it minimal and say so). Test: at that spot the scene's draw list contains both indicators and the button at the game's positions; sister: a placement the game never draws is still absent.
3. The render comparison against PCSX2 happens before merge (name the frame and view).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/lift-indicators-09`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

