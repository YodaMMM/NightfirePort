## Your task: `level-14-skip-handback`, Countdown Part 1: where Bond lands after a skipped cut-scene

On 07000014, skipping the cut-scene that plays early in the level (the one ending near frame ~3000 of a fresh load) leaves Bond somewhere he should not be: a real play run ended up at (26.70, 1.03, 36.27, yaw 2.122) at frame 3046 - inside the wall beside the control-room window (the view clips through the wall) - and then at (26.58, 1.03, 48.57) at frame 3496. An earlier run on the same level put Bond outside the map after a cut-scene. Find which cut-scene(s) on 07000014 can be skipped, the game's skip path [M Script_* skip handling / the hand-back on skip - see the level-02-skip-handback test on master for the shape], and where the game places Bond after each skip [M/D, L if needed]. Fix the port's skip-path hand-back so it matches, and pin each one with a real-input test (load, play to the cut-scene, press skip, assert Bond's position/yaw against the game's). Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-skip-handback`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-10-29T08:13Z
