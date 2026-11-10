## Your task: `level-0d-agent-ladder-top-4`, Countdown can be walked past the ladder top on Agent with real health

Branch from origin/s/level-0d-agent-ladder-top-3, merge current origin/master, and read docs/reports/level-0d-agent-ladder-top-3.md and the Agent walk's ignore message in walkreal_0700000d_live. The death moved from tick 8110 to 13291: the ride, deck and descent pass. The settled law: the west-roof pair g32/g33 cannot be killed from the ladder top (torso lines blocked, head boxes graze at d 63). Take the report's next levers in order: avoid the pair's lines instead of trading with them, or close to d ~31 where the head line lands. Real input only, no top-ups, never lower damage or health; helpers gated to 0700000d and the difficulty. Remove the ignore when green, otherwise push the ledger. Run walkreal_0700000d_live and interactables_0700000d_live.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-0d-agent-ladder-top-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
