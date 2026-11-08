## Your task: `guard-pane-drain-2`, the 07000001 walks stay green with guards breaking glass as in the game

Branch from origin/s/guard-pane-drain, merge origin/s/real-health-00agent-01-glass and current origin/master, and read both reports. guard-pane-drain made guard rounds break the glass sheets they pass, which shifts the 01 d2 sister's ledger, and real-health-00agent-01-glass repaired the 01 00 Agent walk without that change. With both in, run every walkreal_07000001_live test and repair any red walk the way the 01-glass report did, gated to 07000001 and the difficulty. Then run every walkreal_*_live binary one level per tool call, because guard rounds now break sheets on 06, 08, 0a and 12 too [D the census], and repair any walk that shifts, gated to its level. Real input only, no top-ups, never lower damage or health. Remove each ignore when green; if a walk is still red, push the ledger and next lever in its ignore message. Run glass_pass_every_level_live, dead_pane_live, the bullet binaries and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/guard-pane-drain-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-08T09:58Z
