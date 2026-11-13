## Your task: `level-02-castle-walks-hearing-2`, the 02 castle walks green on Operative and 00 Agent under the hearing fold

Branch from origin/s/level-02-castle-walks-hearing and merge origin/master; read docs/reports/level-02-castle-walks-hearing.md on that branch. d1 (walkreal_07000002_live::walk_the_castle_all_objectives_real_health) is green there - keep it green. d3 (::walk_the_castle_all_objectives_real_health_00agent) is still ignored with the next lever in its ignore reason: carry on from the bunker-ambush lure and the quiet-discipline thread-local. Real input only, never lower damage or health, no top-ups; helpers gated to 07000002 and their difficulty. Remove the d3 ignore when green, otherwise push the ledger. Run walkreal_07000002_live, interactables_07000002_live and the 07000001->02 carry walks.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/20-level-02-castle-walks-hearing-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

---
claimed-by: sandbox4 at 2026-11-12T19:44Z

---
## Report (done, 2026-11-13T02:21Z)

Closed by the driver: the report is on `s/level-02-castle-walks-hearing-2` (docs/reports/level-02-castle-walks-hearing-2.md); the worker didn't run finish.sh.

---
Merged (squash, docs dropped): master 80ebf43c9.
