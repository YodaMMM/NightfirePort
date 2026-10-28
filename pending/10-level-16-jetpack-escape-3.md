## Your task: `level-16-jetpack-escape-3`, the Phoenix base escape re-checked on today's master

`walkreal_07000016_live::walk_the_chasers_defeat_completes_the_mission` is ignored (read its ignore message: the pulse-1 survival and the script stream clock's absolute block). Master has moved a lot since that round (the general kind-1/kind-2 hand-back rules, the cut-scene pack anims, the vision-bit and placement fixes). Re-run the walk on master, find the current exact blocker, and fix it from the game's behaviour [M the script ops and the escape's channels; D the level data] - real input only. Un-ignore the test or re-ignore on the next exact blocker with its evidence. Run the FULL suite and quote its total line.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-16-jetpack-escape-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

