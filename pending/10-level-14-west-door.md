## Your task: `level-14-west-door`, Countdown Part 1 past the facility's west double door to the Delta exit

On 07000014, after the vent-drop hand-back (now on master: Bond lands inside the facility), walkreal_07000014_live `walk_the_infiltration_to_the_delta_exit` wedges at the facility's west double door at (-52.2, 1.2, 40.1): two door records, kind 2, state 0x2, pos01 0.00, ch_open 0, ch_lock 0x85, lock_flag 0. Find how the game opens it [D the placements writing channel 0x85 and their triggers; M the door's lock/unlock and use paths, Door_Update / the lock channel read] - a switch, an objective step, a script, or an approach side the port lacks - and port the missing link. Then walk legs 2..6 of the test on real input to the Delta exit #944 and un-ignore it (or re-ignore on the next exact blocker with its evidence). Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-14-west-door`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

