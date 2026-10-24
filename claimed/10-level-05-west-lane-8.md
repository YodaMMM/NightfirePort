## Your task: `level-05-west-lane-8`, 07000005: the kink walked, then the notch grab and the wire ride

Read `docs/reports/level-05-west-lane-7.md` on `origin/s/level-05-west-lane-7` (merged to master; the report itself is not on master) and its probe `crates/nf-view/examples/probe_kink_05.rs`. It settled from the decompile how the game's player takes the 45-degree ramp at the step shelf's kink (-23.5, 33, 46.5): the capsule push is slope-blind and carries him up the ramp [M `Intersect_CylGeom` 0x109da0, `Collide_Intersect` 0x1ebaa8, `Player_CollisionHandler` 0x1949a8], and the feet ray keeps him grounded (normal.y 0.68 > 0.5) [M `Player_FeetOnPoint` 0x194188, `Player_Collision` 0x194df0]. No port divergence was found; the fall to the y-29 roof came from the test walker's stuck sidestep shoving Bond west off the 1-unit strip.

1. Make the walk take the kink the way a player would: steer along the ramp strip (the spine) with the real inputs, without the sidestep that drops him; if the port still stalls, log the per-tick pushes against the game's [M] rules above and fix the port.
2. Un-ignore `the_notch_grab_and_ride_parked` and `the_notch_holds_the_walk_and_the_grab_needs_its_zone`; the grab and ride legs are written and height-checked.
3. Then the south ledge after the ride, the window-ledge crawl #35 (-34.10, 37.28, -31.65) and the exit door #112, un-ignoring what goes green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-05-west-lane-8`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.



---
claimed-by: sandbox6 at 2026-10-24T15:48Z
