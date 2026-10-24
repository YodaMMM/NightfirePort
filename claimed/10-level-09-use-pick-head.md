## Your task: `level-09-use-pick-head`, the use key picks from Bond's head, not the render camera

`interact_07000009_live::the_load_point_e_route_exits_with_objective_2_incomplete` is ignored: the port's use pick runs from the RENDER CAMERA (scene.rs passes `camera` as the head), while the game's `Player_Activate` 0x1a5170 builds it from `Player_GetHeadPos` [M :101-106 — the use point = head + view]. At the 07000009 lift doors the camera hangs ~2.5 behind Bond, so the 1.0 use sphere never spans the exit box (z 10.33..12.49 [D LOAD_LEVEL #63]) and the E press never fires.

1. Make the use pick originate at the game's head position [M `Player_GetHeadPos`, `Player_Activate` 0x1a5170:101-106] for every use (doors, switches, pickups, ladders, wall-hug, lifts), not the camera. Search for the other callers that pass the camera as the head and fix those too.
2. Un-ignore the test. Every interact_* and walkreal_* test green on master must stay green (several walks press E; re-derive any pin that only held because of the camera origin, with the evidence).

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-09-use-pick-head`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-10-24T22:51Z
