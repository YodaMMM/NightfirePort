## Your task: `level-14-lid-and-climb`, 07000014: the duct lid opens like the game, then the Q-Claw climb and the infiltration walk

Master has the rides fix and the east duct route parked on the lid: read `docs/reports/level-14-qclaw-climb.md` on `origin/s/level-14-qclaw-climb` (merged; the report is not on master). The real-input route stops at the trench lid — the lid's collision after it opens is the open question the report names.

1. Find how the game opens and clears the lid [M — the lid's script/SP, its key frames, what collides after the open (the glist the collision pick sees, the script entity's own pose)], port it so the walk drops past it the way the game's player does, and un-ignore `the_east_duct_route_reaches_the_stairwell_slot`.
2. The climb: aim the Q-Claw at grapple #626/#657 (45.30 / 40.78, 4.79, 42.86), fire, reel, E-open the mask-4 lid, onto the mezzanine; rewrite leg 1a of `walk_the_infiltration_to_the_delta_exit` onto the east route and un-ignore it when it walks.
3. Every ride test (lifts, gondola, paternoster, the 4a lift, the wire, 16's chain) stays green; the full suite on the final tree.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-lid-and-climb`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


