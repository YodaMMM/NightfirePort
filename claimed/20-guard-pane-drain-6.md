## Your task: `guard-pane-drain-6`, the 07000001 00 Agent walk stays green with guards breaking glass as in the game

Branch from origin/s/guard-pane-drain-5, merge current origin/master, and read its docs/reports/guard-pane-drain-5.md plus the d3 ignore message. The south-door peek has no exposed stand, so take its lever: the west door #2 peek, the one door pair on the guards' own nav path, with the stage and angle the report gives. If the report's own guard on it still holds, take the east perch instead. Then the escort phase once the AND fires. Helpers stay gated to 07000001 and difficulty 3, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000001_live binary, glass_pass_every_level_live and dead_pane_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-pane-drain-6`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-09T08:49Z
