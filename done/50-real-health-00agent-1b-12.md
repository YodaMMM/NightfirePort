## Your task: `real-health-00agent-1b-12`, 0700001b's 00 Agent walk defeats Drake

Branch from current origin/master.

Master fa191d448: all eight consoles latch (k7 9337, k1 9854, k2 10026) with Bond at 19.5 health; the walk dies at t13331 at (8.06, -2.34, 7.19) in the Drake phase (the assert at walkreal_0700001b_live.rs:1984). Read the 1b-11 ignore ledger on master. Goal: the walk's 00 Agent test passes.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.


- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

**Branch:** `s/real-health-00agent-1b-12`.

Main 11-15 (on fa191d448, NF_DMG_TRACE): after the console run (19.5 health at frame ~10026) the leftover astronauts chip 5.0 a hit (#15 at (14.4, 0.3, -0.6) d 10.4, #18 at (9.8, 4.7, 3.4) d 8.2) and Drake (#19, skin 0x50000ba, weapon 0x30, 113 hp, at (4.7, 2.3, 0.5) d 8.8) lands one 9.52 that ends it at t13331 - the assert "Bond survives Drake's fight". Bill totals: #10 30.0 (6 hits), the laser 14.7 (5, the dodge on), #9 10.0. The astronaut tail before Drake is the budget.

---
claimed-by: sandbox7 at 2026-11-15T09:35Z

---
## Report (done, 2026-11-15T11:09Z)

Closed by the driver: the report is on `s/real-health-00agent-1b-12` (docs/reports/real-health-00agent-1b-12.md); the worker didn't run finish.sh.

---
## Report from the worker (blocked, 2026-11-15T11:19Z)

