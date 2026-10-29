## Your task: `level-05-every-objective-2`, The Exchange castle: the cable crossing on real input in the full walk

`walkreal_07000005_live` walks the objective chain; its cable route `walk_the_west_ledge_and_cross_the_cable` was `#[ignore]`d. The cable now has the game's jump grab, hand-over-hand ride and third-person camera (cable_third_person_live, fix_wire2_live on master). Un-ignore the cable route: walk the west ledge, jump-grab the cable at the icons, ride it hand over hand to the far end and drop off, on real input, then continue the mission. Fix any port gap the real leg hits [M/D]. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-05-every-objective-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

