## Your task: `level-0d-operative-walk-4`, 0700000d can be walked to the end on Operative with real health

Branch from origin/s/level-0d-operative-walk-3, merge current origin/master, and read its docs/reports/level-0d-operative-walk-3.md plus the Operative ignore message. The in-bay #18 audit is solved and the band sweep reaches Operative; what still kills the walk is #18's inbound line. Take its levers in order: (a) the wait-for-crossing act, which fires once the bay mouth presents a clear box instead of burning one fixed cap; (b) the same act from the deck stand (d 29, the 7479 hit's own line); (c) the audit act at the probed bay-enter stand. Every added leg re-rolls the bay pair's patrol dice, so judge each lever over several real runs, as the report did. Do not repeat its measured dead ends. Helpers stay gated to 0700000d and the difficulty. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_0700000d_live binary and interactables_0700000d_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0d-operative-walk-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-09T12:35Z
