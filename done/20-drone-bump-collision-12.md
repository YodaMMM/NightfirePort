## Your task: `drone-bump-collision-12`, the bot-capsule arming without the regressions

`origin/s/drone-bump-collision-11` merged with master (8dc61a92) still turns these RED in the full suite, reproducibly (green on master): fix_npc_doors_live::the_escort_opens_the_interior_doors_on_his_route, walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit, walkreal_07000002_live (TIMEOUT 600 s in walk_the_castle_all_objectives_to_the_exit), walkreal_07000001_live::walk_from_the_spawn_to_the_exit_with_the_contact, escort_follow_07000001_live::escort_at_walking_speed_keeps_the_games_distance_and_the_exit_opens, fix_party_mission_live::the_07000008_safe_chain_works_in_play, playtest_spots_live::spots_07000008_safe_bolts_in_reach, walkreal_07000011_live::walk_spawn_to_the_lower_floor_load_point, pickups08_live (4 tests), fix_safe08_live::the_safe_pickups_take_after_the_safe_opens, speech_facial_fallback_live::a_present_script_talker_plays_through_the_facial_and_advances. Continue from -11 (merge `origin/master` first). Split the branch's changes (the kind-word arming + every-stance Jointy push, the straddled-cel pick filter, the door-zone opponent gate, the boundary-clamp removal) and find which one moves each red; for each, find what the game does at that step [M/D/L]. Fix the port where it is wrong; re-derive a test only with evidence. Land only what keeps the full suite free of new reds - a part that cannot be made green stays out with its evidence in the report. Run the FULL suite on the final tree and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/drone-bump-collision-12`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-10-30T02:03Z

---
## Report (done, 2026-10-30T02:51Z)

Closed by the driver: the report is on `s/drone-bump-collision-12` (docs/reports/drone-bump-collision-12.md); the worker didn't run finish.sh.
