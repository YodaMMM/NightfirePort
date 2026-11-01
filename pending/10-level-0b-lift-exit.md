## Your task: `level-0b-lift-exit`, 0b: off the lift without a teleport

`walkreal_0700000b_live`'s full walk teleports Bond about 6 m after the lift: stepping off lands him in a pit whose rim is 1.35 m high while his jump rises about 0.77 m, and the dock's south edge drops him down the shaft (see the test comments and `origin/s/walk-census` for the measured stops). Suspect: the walk rides the lift's ROOF while the game carries the player INSIDE the car. Find the game's own route off the lift [D the lift car's collision and the shaft geometry; M the mover/lift class and how it carries the player — Player_FeetOnPoint 0x194188, the platform ride], fix the port gap with the address cited, and delete the teleport. Run walkreal_0700000b_live, interactables_0700000b_live, every *0700000b* binary and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0b-lift-exit`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

