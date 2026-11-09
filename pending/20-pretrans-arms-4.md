## Your task: `pretrans-arms-4`, every level's walks stay green with guard root motion zeroed as in the game

Branch from origin/s/pretrans-arms-3, merge current origin/master, and read its docs/reports/pretrans-arms-3.md. The PreTransCallback root zero is in and matches the game; six walks went red on it. Repair them one level per tool call, the -2 round's way: per-level walk repairs with the trace, no sim changes. 01's lever is spelled out in the report; for 05, 09, 0c and 11, replay their wedge dumps against the shifted approach paths. Then re-run 01 and 14, the -2 round's repair levels. Helpers stay gated to their level and difficulty. Real input only, no top-ups, never lower damage or health. Remove each ignore when green; if a walk is still red, push the ledger and next lever in its ignore message. Run the ai binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/pretrans-arms-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

