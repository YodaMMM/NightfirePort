## Your task: `level-0b-roof-sprint-8`, the 0700000b roof sprint can be walked on 00 Agent with real health

Branch from origin/s/level-0b-roof-sprint-7; if origin/s/los-mask-repairs exists, merge it too (it repairs the guard sight masks this walk runs under); then merge current origin/master. Read docs/reports/level-0b-roof-sprint-7.md and the 00 Agent ignore message in walkreal_0700000b_live. Sprint-7 pinned the rocket: 0x2e homes (+0x6C = 0x32200, no gravity), refires every 60 ticks, and spawns on the crosshair ray [M 0x1a4bf0:226-245]; one traced round killed two guards. Take its next levers: aim the round through the crosshair (turn the view to the target before firing, since the walk's turn input does not move the spawn ray) so the lock lands on the #6/#7/#8/#9 cluster, then finish the deck. Real input only, no top-ups, never lower damage or health; helpers gated to 0700000b and difficulty >= 3. Remove the ignore when green, otherwise push the ledger. Run walkreal_0700000b_live and interactables_0700000b_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-0b-roof-sprint-8`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
