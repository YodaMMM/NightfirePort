## Your task: `fusebox-wire-geometry-2`, land the fusebox wire contact on the real cable geometry

Start from `origin/s/fusebox-wire-geometry` (merge it, then origin/master). It replaces the port's 0.5-radius wire sphere at the wire entity's translation (a stand-in, 1.5 m below the visible cables on 07000014) with the wire OBJECT's own bounds through the live entity matrix [M Collide_PickObj 0x1eae80 picks the object; FuseBox_Update 0x1adfe8:88-123 reads its obj+0xD0 chain]. Real play confirms it: lasering the visible cables on 14 now cuts them.

Merged with master these go red (each green on master), because they aim the beam at the old sphere spot, not at the cables:
- fix_fusebox_cut_live::the_07000014_sabotage_cuts_through_the_real_laser ("the laser burns box 1's group-0 wire")
- walk_07000014_live::the_facility_chain_runs_to_the_delta_sector_exit
- fusebox_06_live::the_alarm_bundle_raises_ch55_and_never_hurts
- fusebox_06_live::the_laser_cut_through_the_searchlight_bundle_kills_the_lights

1. For each, aim the real input at the wire object's visible geometry (its world bounds — the branch's `wire_geo` gives them) the way a player would, and keep every assert. Check the 07000006 fuseboxes resolve their wire geometry too (their scripts/hashes [D]); if a box has none, say why and what the game collides there.
2. All four green plus `fusebox_geometry_07000014_live`; run the full suite on the merged tree and report the total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/fusebox-wire-geometry-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-10-25T20:54Z
