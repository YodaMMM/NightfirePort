## Your task: `real-health-00agent-01-glass`, 07000001 can be walked on 00 Agent with rounds passing glass as in the game

Branch from origin/s/glass-pierce-2, merge current origin/master, and read its docs/reports/glass-pierce-2.md, item 3 of "What's left". With through-glass hits, the 07000001 00 Agent sister's atrium timeline shifted: guard #5 (shotgun 0x1c) at d 1.6 lands 76.6 in 8 hits, and Bond dies at 3623 at (6.49, 6.47, 13.48). Apply the family of repairs that worked for 08 in that report: plan the 01 fights with glass in the guards' sight, and back off out of #5's close range. Helpers stay gated to 07000001 and difficulty 3, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green. If the walk is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000001_live binary, glass_pass_every_level_live and dead_pane_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-01-glass`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-08T06:19Z

---
## Report (done, 2026-11-08T07:17Z)

