## Your task: `cutscene-kind31-frame-3`, the general frame rule without breaking the gondola and the escorts

Start from `origin/s/cutscene-kind31-frame-2` and read its report (merge origin/master). Its one-line rule (`keys_alone = camera_started && parent < 0` for every unparented case-1 stream [M 0x1d9a08 case-1 0x1d9c0c, Script_GetInterp 0x1da838, Script_CameraStart 0x1d916c]) turned five tests red on the full suite, reproducible alone: walkreal_07000008_live `walk_the_gondola_ride_to_the_end` and `the_ally_fights_only_the_guards_through_the_noise_path`, zoe_gondola_realpath_live `zoe_rides_the_carriage_and_never_targets_bond` (all three: "the cable-car floor plate fired ch 4 [D #236 mask 8]" fails), mayhew_hold_live `the_escort_holds_at_the_ally_wait_node_until_the_ambush_clears` ("never parked in AllyLeadMissionWait 0x21"), escort_hide_findrun_live `the_hides_exit_reanchors_the_mission_route_before_the_lead_resumes` (route index 2, want 1). Find which unparented stream in which camera script the rule now reads keys-alone where the game composes (or vice versa) - e.g. a scene's camera op tick vs the stream's op tick, a SubScriptStart base reset [M 0x1db050:28-38], SP_SetPosRot [M 0x1db920] - and fix the rule to what the game does; all five green plus the 01/14 battery from frame-2's report.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/cutscene-kind31-frame-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

