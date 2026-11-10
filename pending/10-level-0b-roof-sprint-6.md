## Your task: `level-0b-roof-sprint-6`, the 0700000b roof sprint can be walked on 00 Agent with real health

Branch from origin/s/level-0b-roof-sprint-5, merge current origin/master, and read its docs/reports/level-0b-roof-sprint-5.md plus the 00 Agent ignore message. The Agent walk is green. For 00 Agent, its first lever is the moving-target accuracy: the walk's [0b-stall] prints show Bond read as standing still while he runs. Check first whether origin/s/onfoot-moving-accuracy (the game's move-vector read for that term [M 0x179740, 0x179a48:34-41]) is on current master and covers this; if the port still reads a running Bond as still there, fix that in the game's way and test it red on the old code. Then the P1/P2 long-range toll. The blast range belongs to explode-hurt-radius and the 11 corpse fire to corpse-fire. Helpers stay gated to 0700000b and the difficulty. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run walkreal_0700000b_live and interactables_0700000b_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0b-roof-sprint-6`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

