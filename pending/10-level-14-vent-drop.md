## Your task: `level-14-vent-drop`, Countdown Part 1's vent drop carries Bond into the facility

Read `docs/reports/level-14-full-walk-4.md` (on master). `walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit` now walks green to the mezzanine (41.88, 2.79, 39.03) and objective 1, then stops: the TouchOnce #301 (out 0xd9, centre (40.72, 3.10, 39.01)) fires, the SP #276 cut-scene (0x06000833, 1400 ticks) runs and writes ch 0x47, but Bond is left on the mezzanine, behind a 3.5-3.7 wall at z 38.21 he cannot jump [M apex 0.78], with no nav route on. Find how the game moves Bond into the facility here [M the scene's player ops: SetPosRot / Player_Enable hand-back (Script_EventHandler 0x1d94c0 case 6, Script_Free 0x1da880:44-48), the touch's out channel 0xd9's consumers, any teleport/spawn marker] [D the pack's streams and events] and port it. Then continue the walk (legs 2..6) to the exit on real input; un-ignore when green, else back to `#[ignore]` with the exact blocker.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-vent-drop`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

