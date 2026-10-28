## Your task: `level-14-guards-at-locked-door`, Countdown Part 1's guards heard at a door outside the sabotage room

On 07000014, after the capture scene and during the panel/fusebox sequence, the player hears guards trying to open a door outside the room. That is all that was observed - whether they should come through, wait, or stay out is not known. Once both west fuseboxes are cut (ch 0x87/0x88 -> AND #717 -> 0x8C) the spawners #720/#1333 start on 0x8C [D]. Find what the game's spawned guards do [M the spawner's spawn, their route/scripted states, a drone's use of a locked door (Door_Activate by a drone, the lock channels), their barks] and compare with the port; port any missing link. Real-input test from the fusebox cuts: the guards' spawn, states and positions over time, the door interaction, and what changes after the E-use cascade opens Door_Slide_11A/B. Report what the game does as [M]-cited fact and anything unproven as [S]. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-guards-at-locked-door`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

