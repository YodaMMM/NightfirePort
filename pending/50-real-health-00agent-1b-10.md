## Your task: `real-health-00agent-1b-10`, 0700001b's 00 Agent walk reaches Drake

Branch from current origin/master.

Master 2051dc6ef: death t11002 at (48.54, -6.85, -3.09) in the dead window before the tail consoles' holds - the 40-hp astronauts (0x5000078, 0x6a) at 7-13 u; 29.0 of the 100 is the space laser (record 107, radius 20, the target locked at the charge start). The level holds no pickups. Measured worse: the kite's tap gate at clip > 10 / > 60 (the merged > 36 stays). Goal: the walk's 00 Agent test passes.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.


- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

**Branch:** `s/real-health-00agent-1b-10`.
