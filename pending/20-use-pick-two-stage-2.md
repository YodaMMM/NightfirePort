## Your task: `use-pick-two-stage-2`, the game's use pick without breaking the E presses

Continue `origin/s/level-03-locked-door-use-volume` (merge `origin/master` first; read its report). It replaced `Drones::player_use_triggers`' ±1.0 AABB stand-in with the game's two-stage pick (bounds-sphere pre-gate [M Collide_PickObj 0x1eae80:219-228] + the use point vs the trigger glist [M Collide_Intersect 0x1ebaa8:186-190 -> Intersect_PointGeom 0x1e92d8]). Merged onto master the full suite went 17 red, all E-use paths: campaign_flow_live, doublecross_transitions_live, guards13_wake_live, guards13_warm_probe, interactables_0700000b_live, interactables_07000016_live, movie_trigger_live, use_key_census_live, walk_07000012_live (the override console ch 0x49), walkreal_0700000b/0c/11/11_route/12/13/41_live, walkreal_probe13. So legitimate presses no longer register. Find what differs from the game: the use point itself (head + look x 1.0 [M Player_Activate 0x1a5170:100-108]), the [S] parts of the point test (the flags byte / one-sided window, the on-triangle rule — Intersect_PointGeom may test distance to the triangle, not the projection), the bounds sphere (obj+0x80/+0x8C are the MODEL's sphere, not the collision AABB's), and the trigger classes that are not glist hits at all. Fix it so 03's locked_door_refuses_the_e passes AND every binary above is green. Also: no comment may point readers at docs/reports or name who runs a live check. Run every binary above plus nf-game; quote each total.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/use-pick-two-stage-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

