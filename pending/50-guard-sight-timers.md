## Your task: `guard-sight-timers`, the per-guard sight timers behind the spotted / lost-you barks

The sight-bark latches in `crates/nf-game/src/drone.rs` (~line 17975, `DroneVision_ProcessDroneSight` 0x176a28: NewSight after 60+ frames of sight, LostSight after 120+ unsighted frames) run on the port's `seen_ticks` / `unseen_ticks`, which the comment marks as a stand-in for the game's per-drone sight timers ("[S] fold, 0c/0d only"). Find the game's per-drone sight timer fields and how they count [M 0x176a28 and its callers, speech.md §2.3], port them as the fields the barks read, and make the barks fire on every on-foot level the way the game fires them, not only 0c/0d. Pin with a real-input test: a guard on a fixed trigger spots Bond and barks at the game's frame, then loses him and barks again. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-sight-timers`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

