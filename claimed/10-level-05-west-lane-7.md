## Your task: `level-05-west-lane-7`, 07000005: past the step shelf's kink, then the notch grab and the wire ride

Master now walks the west lane green through the band gap (the band's south-west continuation, the z 44.5-45 corridor, the x -20 seam, the cable ledge and the step's spine start: `walkreal_07000005_live::the_cable_legs_the_gap_and_the_ledge_step`). The one blocker is the step shelf's kink at about (-23.5, 33, 46.5): `Room_Outerwall_Corner` (instance 13) carries a 45-degree ramp (33.0 -> 34.0, base line (-23.39, 47.09) -> (-26.03, 44.44), normal (-0.52, 0.68, 0.52)) across the spine, and every walk and hop so far lands on the y-29 roof west of the ramp base. Read `docs/reports/level-05-west-lane-6.md` on `origin/s/level-05-west-lane-6` first (merged to master; the report itself is not on master) and the probe `crates/nf-view/examples/probe_gap_05.rs`.

1. **The kink.** Find how the game's player gets past it [M]: does `Player_FeetOnPoint` 0x194188 / `Player_Collision` 0x194df0 ground the pick on a 45-degree ramp (the slope limit, the step-up), does the capsule push slide along it, or does the route leave the spine earlier (round the ramp from its east, a lower shelf)? Dump the ramp's triangles and the capsule's contacts along the spine. Then walk it with real input only.
2. Un-ignore `the_notch_grab_and_ride_parked` and `the_notch_holds_the_walk_and_the_grab_needs_its_zone`; the grab and ride legs are written and height-checked.
3. If time remains: the south ledge after the ride, the window-ledge crawl #35 (-34.10, 37.28, -31.65) and the exit door #112.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-05-west-lane-7`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.



---
claimed-by: sandbox1 at 2026-10-24T08:22Z
