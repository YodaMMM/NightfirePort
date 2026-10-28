## Your task: `capsule-cull-and-los-masks-10`, the unconditional cull + LOS masks with the four last reds closed

Continue `origin/s/capsule-cull-and-los-masks-9` (merge `origin/master`). Its report bisects the remaining reds to their movers (cull / masks / both) and measures the searchlight chain (the harness Bond starts in the under-croft and relied on the old push-up through the courtyard floor the game's head-on cull skips [M Intersect_CylGeom 0x109da0:302-322]). Close them one by one from the game's behaviour [M], re-deriving a test only where it leaned on the old non-game rule, with the cite: walkreal_07000001 `following_mayhew_down_completes_the_level` (cull), fix_npc_doors `the_escort_opens_the_interior_doors_on_his_route` (masks), mp_perf_16 `sixteen_participants_fixed_run_digest_is_pinned` (both - a digest re-pin only with the mover named), fix_searchlight_remainder `a_courtyard_head_tracks_bond_until_the_lost_limit` (cull - start Bond where the game's player stands). Double Cross Part 1 must still be finishable. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/capsule-cull-and-los-masks-10`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

