## Your task: `level-16-finale-pit-3`, 16: the player survives the finale and the mission completes

Read `blocked/10-level-16-jetpack-escape-2.md` on the queue branch (its whole report) and `walkreal_07000016_live::walk_the_chasers_defeat_completes_the_mission` (ignored). The mission cannot be finished on real input: the ch-0xf3 hurt-volume window of script 0x0600090a stream 6 kills Bond. Find how the shipped game gets the player through that window [D the script streams, the hurt volume's gates, the items/vehicle the finale hands the player; M the trigger/script classes involved and the player state they set], fix the port gap with that evidence, and un-ignore the walk so the mission completes on real input. Run walkreal_07000016_live, interactables_07000016_live and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-16-finale-pit-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

