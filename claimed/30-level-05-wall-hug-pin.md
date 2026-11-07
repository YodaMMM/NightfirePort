## Your task: `level-05-wall-hug-pin`, the 07000005 wall-hug plate swaps the gate guards while the combat music plays

`the_wall_hug_plate_swaps_without_the_cut_scene_while_the_combat_music_is_on` in crates/nf-view/tests/spawner_kill_live.rs is ignored. Its message says the fix's own assertions pass (no freeze, SP #298 refused state 5, `npc_enabled` stays up) and that the failure is test setup: the spawned gate pair reads hp 10 at spawn and takes a stray death during the yard wait. First find what kills them. If it is the port's own guards or a real rule of the game, read that rule and say so, because hp 10 at spawn may itself be wrong: check the spawner's record against the placement [D] and the health the drone init gives it. Fix the port if it is wrong; otherwise shorten the wait or use a quieter spot in the yard. Then pin the visible result through real input: with combat music on, the plate swaps the pair for the placed pair behind the gate with no cut-scene and no freeze, and the game's order of channel writes holds. Remove the ignore. Run the whole spawner_kill_live binary, walkreal_07000005_live, interactables_07000005_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-05-wall-hug-pin`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-07T12:03Z
