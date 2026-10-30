## Your task: `level-03-locked-door-use-volume`, the mask-4 trigger's use-pick volume on 03

`interactables_07000003_live::locked_door_refuses_the_e` is ignored (level-03-interactables, merged; read its report's gap section): the port's use-pick against the mask-4 trigger beside the locked door does not match the game's reach. Find the game's use-pick volume for that trigger [M Player_Activate 0x1a5170:100-137, the object's collision/glist bounds; D the placement] and fix the port's stand-in, then un-ignore the test. Run every interactables_* and interact_* binary; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-03-locked-door-use-volume`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-30T16:23Z
