## Your task: `level-05-west-lane-3`, 07000005's west lane, the two creepwall crawls and the cable notch on the real walk

`walkreal_07000005_live::walk_the_west_ledge_and_cross_the_cable` is ignored. What is established (the test's doc comment holds the fine collision map): the ladder balcony is a closed perch (the map's own parapet [D]); the real lane runs south from the courtyard's west strip (x 28-56, z 92) along the castle wall (x 24-48, z 44-92), over the tower top and down its west slope; "Wall Hug Short" #299 (25.79, 31.78, 67.76) and "Wall Hug Window" #35 (-34.10, 37.28, -31.65) are CreepWalls (class 0x3F, `CreepWall_Create` 0x196fc8, crawl substate 7 [M 0x1a5170:66-97], the port's creepwall.rs); the wire #251 (-23.49, 34.47, 6.26) crosses an 8-unit notch in the cable ledge (z 4-10), and the action-edge grab [M Player_HandleJump 0x1916b0:126-147] reaches it standing at the notch edge (icons #277/#300/#310). The first two legs walk green.

1. Walk the strip south of z 92: the 29.3 to 30.3 lip at x 42-44 wedges the walker. Find whether the game's step-up clears it (cite the player's step rule [M]) and fix the port if it doesn't; otherwise take the sidestep route.
2. The crawl through #299 with the real E press, the lane-to-ledge link, the notch grab, the ride, the #35 window-ledge crawl and the exit leg, real input only.
3. Un-ignore the test with a sister that can fail. If a live number is needed, name the PCSX2 read for a fresh boot and stop there.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-05-west-lane-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

