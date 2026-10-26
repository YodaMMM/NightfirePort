## Your task: `level-0c-display-hatch-4`, which body queries the glass reject applies to

Start from `origin/s/level-0c-display-hatch-3` (merge it, then origin/master). Its blanket body reject of mat-0x40 faces (world.rs `capsule_tri_contact`: `if mat & 0x40 != 0 { return false; }`, from `Collide_Filter` 0x1ed888:7-9 with the 0x800 flag) was NOT merged: on the full suite it broke walkreal_0700001b_live `the_boundary_wall_stops_the_float_and_passes_bullets` (OBJECT_Boundary_1, mat 0x40, pinned body-solid), walkreal_07000001_live `walk_from_the_spawn_to_the_exit_with_the_contact`, walkreal_07000008_live `walk_the_gondola_to_the_briefcase_and_back`, walkreal_07000011_live `walk_spawn_to_the_lower_floor_load_point`, fix_npc_doors_live `the_escort_opens_the_interior_doors_on_his_route` and mp_perf_16_live's digest. So the game does NOT drop every 0x40 face from every player query. Find exactly which query carries 0x800 [M: every caller of Collide_Intersect / Intersect_CylGeom with the player — Player_Collision 0x194df0's four collide-flag writes :205/:229/:257/:282 and what reads them; the zero-G float substate; the walk vs the step/ground probes], and which 0x40 faces the game treats as body-solid (the 1b boundary) vs walkable-through (the 0c skylight-lantern glass, [L 2026-10-26]). Port that rule only, keep all six of those green, and keep the 0c roof-walk test green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-display-hatch-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox6 at 2026-10-26T21:39Z

---
## Report (done, 2026-10-26T22:12Z)

