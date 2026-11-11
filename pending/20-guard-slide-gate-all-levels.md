## Your task: `guard-slide-gate-all-levels`, the guard slide rule applies on every level as in the game

Branch from origin/s/guard-stuck-0c-14, merge current origin/master, and read its docs/reports/guard-stuck-0c-14.md, item 2 of "What is left". The guard slide law is the game's on every level, but the port's gate still carries a level list because the 07000008 walk shifts without it. Re-tune the 07000008 walks for the law, gated to 07000008 and the difficulty, then drop the level list so the law runs everywhere. Run every walkreal_*_live one level per tool call and repair any other walk that shifts, gated to its level. Real input only, no top-ups, never lower damage or health. Run the whole walkreal_07000008_live binary, the ai binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-slide-gate-all-levels`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

