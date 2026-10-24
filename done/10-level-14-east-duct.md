## Your task: `level-14-east-duct`, 07000014: the east duct mouth, then the infiltration walk

`walkreal_07000014_live::the_east_duct_route_reaches_the_stairwell_slot` and `walk_the_infiltration_to_the_delta_exit` are ignored. The real-input east crawl wedges at the duct mouth: both lanes (z 63.7 and z 65.4) stop at x 40.98 with head_bump true, crouched and grounded — the mouth's lintel sits at crawl-head height in the port's world (read the two ignore reasons in full).

1. Dump the triangles of x 40.9..41.6, y 3.1..3.9, z 63.0..66.0 and name their owner objects and glist flags. Is the lintel structural wall, or a flagged label/lid object the game's player collision skips [M `Collide_Pick` 0x1eb320, `Player_Collision` 0x194df0 and its crouch capsule]? Compare the crouched capsule's head height with the game's [M].
2. Fix the cause [M] (not a test-side teleport), pin it with a test that fails before the fix, and un-ignore the east-duct leg; then take the walk on to the slot crossing and the exit as far as it goes, un-ignoring what goes green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-east-duct`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox1 at 2026-10-24T17:06Z

---
## Report (done, 2026-10-24T18:08Z)

Closed by the driver: the report is on `s/level-14-east-duct` (docs/reports/level-14-east-duct.md); the worker didn't run finish.sh.
