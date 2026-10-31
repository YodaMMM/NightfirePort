## Your task: `level-0d-boarding`, 0d: the last placement in the crane chain

`walkreal_0700000d_live::walk_arrival_to_the_c5_door` keeps one placement: the boarding of SP #115's gantry (the ride west and the drop into the yard are real input now). Read the merged level-0d-beam-boarding history in the test comments and probe_crane20. Settle the boarding from the data [D the gantry's script-entity collision per tick of the 3000-tick shuttle; the roof/catwalk edges it passes; M Control_InheritVelocity 0x133e88 / Player_FeetOnPoint 0x194188], walk it on real input and delete the write; if only a live run can settle it, pin the exact question with a sister. Run walkreal_0700000d_live and interactables_0700000d_live. Commit and push your branch after each step so a reset loses nothing. Run the named binaries and quote every total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0d-boarding`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-31T21:57Z
