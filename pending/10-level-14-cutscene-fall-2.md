## Your task: `level-14-cutscene-fall-2`, the cut-scene base reset without breaking other levels' scripts

Start from `origin/s/level-14-cutscene-fall` (merge it first; its report: `Script_CameraStart` 0x1d9140 resets the script base SCRIPTINFO+0x00 to identity [M Mat_IdentityT at 0x1d916c], so a script with a camera op keys world space afterwards). Merged with origin/master these go red, each green on master (and green on master + the urn fix):
- interact_07000001_live (the armour pickup + channel 13; the E key on the entry sliding door)
- exit_aim_01_live::the_exit_resolves_to_the_gate_plate_and_e_is_refused_there
- escort_hide_findrun_live::the_hides_exit_reanchors_the_mission_route_before_the_lead_resumes
- zoe_gondola_realpath_live::zoe_rides_the_carriage_and_never_targets_bond (08)
- walkreal_07000008_live (the ally fights only the guards; the gondola ride to the end)
- cutscene_fall_probe_07000014::probe_camera_flags_of_pinned_scripts (the branch's own probe)

1. Find which scripts on 01/08 your change re-keys (camera-flag scripts whose entities/objects were authored as local offsets) and what the game really does for them [M — when exactly the identity reset happens relative to the entity streams' first keys, whether it applies to the entity interps or only the camera interp (case 4 vs case 1 base-concat flags at 0x1da054 / 0x1d9b30), which SCRIPTINFO the reset hits when a player runs several scripts]. Fix the port to the game's exact rule.
2. Everything above green, the 14 cut-scene camera stays in the room and Bond is back on the floor after the hand-back; run the FULL suite on the merged tree and report the total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-cutscene-fall-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
