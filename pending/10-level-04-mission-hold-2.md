## Your task: `level-04-mission-hold-2`, the mission-complete hold freeze without breaking two other tests

Start from `origin/s/level-04-mission-complete-early` (merge it, then origin/master). It folds `Mission_Update` case 7's `GS_PausePlayer(1, 0)` [M 0x182c60:123-130] into the sim's input gate (`mission.internal == 7`). Merged with master these two go red (each green on master, confirmed by re-running without the branch):
- `walk_07000013_live::the_front_entrance_completes_the_mission`
- `civilian_no_shoot_live::the_trigger_fires_under_the_mark_and_the_civilian_takes_the_hit`

1. Find what each test does during the case-7 hold and what the game does there [M — GS_PausePlayer's exact effect (input only? the weapon/fire? the trigger?), which mission internal states run case 7's arm (success and fail), the hold's 240 ticks]. The civilian one likely shoots/fires during a fail hold; the 13 walk likely acts after the completion. Fix the port's gate to the game's exact effect, or, where the game really freezes the player, make the test act the way a player can.
2. Both green plus the branch's own 04 test; run the full suite on the merged tree and report the total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-04-mission-hold-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
