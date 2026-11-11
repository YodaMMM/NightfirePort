## Your task: `guard-slide-03-01-repairs`, 07000003's and 07000001's Agent walks are green with the guard slide rule everywhere

Branch from origin/s/guard-slide-gate-sweep, merge current origin/master, and read its docs/reports/guard-slide-gate-sweep.md, items 1 and 2 of "What is left". The slide law is the game's and stays. First the 07000003 repair: diagnose the wedge and re-take the lane exit, gated to 07000003 and the test's difficulty, following the ledger on the test. Then continue the 07000001 Agent repair: re-time the Mayhew sister per leg, then the 900-tick step-out, tracing the 13556 leg with NF_RF_TRACE. One level per tool call. Real input only, no top-ups, never lower damage or health. Remove each ignore when green; if a walk is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000003_live and walkreal_07000001_live binaries and their interactables binaries.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-slide-03-01-repairs`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-11T14:09Z
