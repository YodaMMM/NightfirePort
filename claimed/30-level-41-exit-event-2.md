## Your task: `level-41-exit-event-2`, the 41 exit event without breaking the wrapper chain

Continue `origin/s/level-41-exit-event` (merge `origin/master` first). On master it turns `interactables_07000041_live::the_wrapper_ends_into_the_space_station_after_the_chain` red. Find how the game's 41 exit event and the wrapper's end-into-the-space-station chain coexist [D the 41 scripts/channels; M the level-exit / LOAD_LEVEL path], fix the port and keep the wrapper test green without weakening an assertion. Run every *07000041* binary, the campaign/hand-off binaries and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-41-exit-event-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-03T06:13Z
