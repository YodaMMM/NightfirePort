## Your task: `capsule-cull-and-los-masks`, the game's head-on cull and the per-caller LOS masks, without the real-path regressions

On master both are written but held off by flags in crates/nf-game/src/world.rs: `HEAD_ON_CULL_ON` (the normalized-spine cull in `capsule_tri_contact`, [M Intersect_CylGeom 0x109da0:302-322]) and `LOS_MASKS_ON` (`line_of_sight`'s mask drops and back-face skip, [M Collide_Intersect 0x1ebaa8:300-319, Collide_Pick 0x1eb320:140-152]). Their unit tests in world.rs are ignored until this lands.

Measured with each flag on alone:
- The cull on broke interact_07000006_live (the guardhouse E door: Bond never settles on the door's floor), interact_07000009_live (the body-armour walk-over), level_end_live (07000016 straddle door), walkreal_07000008_live (the laser burn never writes the bolt's ch 134), gun_turrets_live (the turret's rounds never reach Bond), bathhouse_shout_mixer_live, and the mp_net_slots_post_move_live digest.
- The masks on broke ai_live (`guard_takes_cover_like_the_game`: the state log diverges after 0x67) and zoe_gondola_realpath_live (an ally round aimed at Bond).

1. For each failing test, find the triangle or mask drop that changes the outcome (print the culled/dropped faces along the failing segment or capsule) and compare with the game's rule. Either the port's reading of the game is off, or a stand-in elsewhere depended on the old behaviour; fix whichever it is, with the citation.
2. Turn both flags on (then remove them). Every test above, the world.rs unit tests and exit_aim_01_live (the 01 stairwell descent) must pass.
3. No test may be weakened to get there; a pin that contradicts the decompile is fixed with the address.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/capsule-cull-and-los-masks`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

