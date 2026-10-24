## Your task: `level-14-qclaw-climb`, 07000014: the trench landing without breaking the rides, then the Q-Claw climb and the infiltration walk

Start from `origin/s/level-14-trench-landing` (NOT merged; read its `docs/reports/level-14-trench-landing.md`). It makes the real east duct route walk green, but its `crates/nf-game/src/drone.rs` change turns the platform rides red on master's tree — each passes on master and fails with the branch:
`lifts_ride_live::the_paternoster_carries_bond_from_load`, the three `gondola_windows_live` tests, `zoe_gondola_realpath_live::zoe_rides_the_carriage_and_never_targets_bond`, `interact_0700004a_live` (the floor plates and the hint marker as the lift descends), both `walkreal_0700004a_satchel_live` tests, `walkreal_0700004a_live::ride_the_lift_from_the_spawn_and_walk_into_the_lobby`, `fix_wire2_live::the_fusebox_scenario_starts_crouched_inside_the_vent`, `walkreal_07000016_live` (2), `kiko_button_real_walk_live`, and `campaign_flow_live` (2).

1. Find what the branch's drone.rs change does to riding/carried bodies (lifts, gondola, paternoster, the wire), fix it so every test above is green again while the east route stays green [M]. Merge `origin/master` first (master has moved: the camera beam, the satchel draw).
2. Then the climb leg: aim the Q-Claw at grapple #626/#657 (45.30 / 40.78, 4.79, 42.86), fire, reel, E-open the mask-4 lid, out onto the mezzanine; rewrite leg 1a of `walk_the_infiltration_to_the_delta_exit` onto the east route and un-ignore it when it walks.
3. The full suite on the final tree: no test red that is green on master.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-qclaw-climb`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


