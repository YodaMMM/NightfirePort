## Your task: `real-damage-routes-2`, rebase the 03/08/11/14 real-damage routes onto master

`origin/s/real-damage-routes-08-11-14-03` (finished, worker) conflicts with master in `walkreal_07000001_live.rs` and `docs/UNKNOWNS.md` (master since gained enemy-stats-4, stealth-los-4, hud-start-2 and walker changes: keep_off/back_off, prefer_head, sidestep_decays, rise_watch, bridge_wedged_nav). Merge `origin/master` into it, resolve keeping both sides' intent (UNKNOWNS: unique U- ids), re-run its real-damage variants for 03/08/11/14 against master's guards (seven-bone sight, decoded stats) and fix the walker where they now fail — never an assertion. Run every walkreal_* binary and nf-game; quote the totals.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-damage-routes-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-01T17:35Z
