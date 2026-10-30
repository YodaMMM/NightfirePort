## Your task: `level-0c-real-crossings-2`, Chain Reaction Part 1: the real crossings on top of the green walk

`level-0c-every-objective-blockers-2` (merging now) makes the whole 0c walk green on today's master with the #283 ride re-derived. `origin/s/level-0c-real-crossings` made the hinge-burn leg REAL (the hinge locks burned through held fire, the trench drop fires plate #274) but its #283 landing went red. Start from origin/master (with blockers-2 in), port the real-crossings branch's hinge-burn leg onto it, then the remaining wire crossings (#284, #282) and descents on real input, keeping blockers-2's #283 leg. The walk must stay green. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0c-real-crossings-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

