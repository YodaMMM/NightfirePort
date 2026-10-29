## Your task: `capsule-cull-and-los-masks-13`, the head-on cull and LOS masks without new reds

`origin/s/capsule-cull-and-los-masks-12` merged with master (8dc61a92) turns these RED, reproducibly when re-run alone (green on master): walkreal_07000002_live::walk_the_castle_all_objectives_to_the_exit, walkreal_07000003_live::walk_the_full_route_and_complete_every_objective, fix_npc_doors_live::the_escort_opens_the_interior_doors_on_his_route, walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit. Continue from -12 (merge `origin/master` first). Split the two changes (the head-on capsule cull with the corrected direction test; the LOS masks / masked_scan) and find which moves each red, then what the game does at that step [M Intersect_CylGeom / Collide_Intersect 0x1ebaa8; L if needed]. Fix the port where it is wrong; re-derive a test only with evidence. Land only what keeps the full suite free of new reds; a part that cannot be made green stays out with its evidence. Run the FULL suite on the final tree and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/capsule-cull-and-los-masks-13`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-10-29T21:40Z
