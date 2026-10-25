## Your task: `level-14-east-walkway`, 07000014: walk the infiltration by the east walkway and stairwell

Start from `origin/s/level-14-lid-and-climb` (merge it, then origin/master; its report and seven probes have the [D] room layout: two parallel walkways, the east stairwell x 44.5..46, z 58 -> 51, and the [M] proof that the mask-4 trench lids are one-shot SPs that never open). Note: `panels_07000014_live::the_route_lid_use_spawns_the_ladder_art_beneath_the_opening` (on master) says an in-duct use press spawns the ladder art; reconcile the two readings first [M SP_Update 0x1dbda8, SP_LoadScript 0x1dbc08] and fix whichever test or doc is wrong.

1. Rewrite leg 1a of `walk_the_infiltration_to_the_delta_exit` onto the east walkway with real input: the start nook (x 39..41.5, z 63..67) connects only to the west walkway in the port — resolve the west -> east crossing (is there a gap, a jump, a crouch, a grapple point in the game? [D placements + M]), descend the east stairwell, corridor east to x 49, south to the mouths' lane.
2. The mezzanine entry from inside the east duct's south mouth; un-ignore `the_east_duct_route_reaches_the_stairwell_slot` (its slot-drop leg becomes the stairwell descent), then the infiltration walk once it walks green.
3. Run the full suite on the final merged tree and report the total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-east-walkway`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-10-25T15:31Z
