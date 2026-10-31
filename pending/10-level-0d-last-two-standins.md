## Your task: `level-0d-last-two-standins`, 0d: the crane-to-yard chain fully on real input

level-0d-jib-walk-off (merged; read its report and `probe_crane20`) made the deck walk-off, the catwalk walk and the dock E #460 real input, and left two narrowed stand-ins in `walkreal_0700000d_live::walk_arrival_to_the_c5_door`: (1) the catwalk -> bay exit, (2) the beam boarding + the west ride on SP #115's shuttle (137.54 <-> 177.22 over 3000 ticks [D]) and the drop into the yard at x 137.54. Find the game's way for each [D the catwalk/roof/bay collision incl. dynamic objects, the beam's script-entity collision and its timing; M the script-entity carry Control_InheritVelocity 0x133e88 / FeetOnPoint 0x194188], walk both on real input and delete both writes. Fix any port gap with evidence. Run walkreal_0700000d_live and interactables_0700000d_live; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0d-last-two-standins`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

