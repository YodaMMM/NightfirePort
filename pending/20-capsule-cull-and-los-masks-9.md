## Your task: `capsule-cull-and-los-masks-9`, the unconditional cull + LOS masks with the full suite green

`origin/s/capsule-cull-and-los-masks-8` (the per-caller LOS masks [M Collide_Intersect 0x1ebaa8:300-319] and the head-on cull [M Intersect_CylGeom 0x109da0:302-322] made unconditional, the GT turret track ray from the muzzle push to the target's world sphere [M 0x17f9c8:232-278], GT spread draws [M 0x12c5a0:160-180], the ext+0xd0 muzzle offset) merged onto master gives five reproducible reds in the full suite (each green on master): walkreal_07000001_live `following_mayhew_down_completes_the_level`, fix_npc_doors_live `the_escort_opens_the_interior_doors_on_his_route`, mp_perf_16_live `early_out_segments_match_the_full_search` and `sixteen_participants_fixed_run_digest_is_pinned`, fix_searchlight_remainder_live `a_courtyard_head_tracks_bond_until_the_lost_limit`. Merge origin/master into the branch, then for each red find what the game does at that moment [M the caller's mask, the cull arms] and fix the port, or re-derive the test with the decompile cite (a digest re-pin only with the named mover and its cite). Also: `line_of_sight_masked` takes the dynamic read lock and then calls `masked_scan`, which takes it again - make it one call. Drop tools/test/run_suite_resume.sh (environment-specific). Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/capsule-cull-and-los-masks-9`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

