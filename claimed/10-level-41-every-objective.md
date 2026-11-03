## Your task: `level-41-every-objective`, complete every 07000041 objective on real input

No walk pins every 07000041 objective in order. walkreal_07000041_live reaches the end, but no test completes each objective as a player does. Read the level's objective list and what completes each one: the objective records, the channels and the scripts [D the level file, M the objective handlers]. Then extend walkreal_07000041_live, or add a sister test, that plays the whole mission on real input from the fresh start. Every objective completes in the game's order. Each completion is checked on the channel or objective state the game reads, not on Bond's position. Bond reaches the level end alive, with no teleports and no forced fields. Where a step can't be done the way a player does it, fix the port against the decompile, and fix the whole class. Coordinate with the pending 30-level-41-exit-event-2 by staying off the exit event and naming it in the report if the end trigger is the blocker. Run walkreal_07000041_live, interactables_07000041_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-41-every-objective`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-03T04:05Z
