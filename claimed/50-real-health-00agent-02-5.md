## Your task: `real-health-00agent-02-5`, 07000002 can be walked to the end on 00 Agent with real health

`walk_the_castle_all_objectives_real_health_00agent` in crates/nf-view/tests/walkreal_07000002_live.rs is still ignored, and no session has worked on it beyond one repro. Read the 02 sections of the "Report from the worker" in the queue's blocked/50-real-health-00agent-02-03-4.md. That repro died at t6833 at (-31.35, 6.46, 8.40): 66 hits for 100, of which #7 (skin 0x500003a, weapon 0x1c) landed 88.8 in 32 hits at d 3.5, fought with a dry gun, and the log read "geisha 1 stand-off 0 shotgun: 0x1c not carried". Branch from origin/master and repeat the repro first, because the guards' door and radius work may have moved it. The walk's own 00 Agent tactic is to scoop a dropped shotgun and kill a 12-hp captor with one tap, so establish from the decompile and the placement [M]/[D] what #7 and the earlier captors actually drop and when, instead of relying on the walk's timing. Then reach the geisha-1 room with a loaded gun: scoop an earlier drop deliberately, or re-order the stand-off so a drop lands before the duel. The dry-gun fix in docs/reports/door-walk-03b.md on origin/s/door-walk-03b (resupply arms before each risky leg, along covered paths only) is the same failure on 07000003, so reuse its shape. Real input only, no top-ups, never lower damage or health, and gate the changes on difficulty 3 so the other walks stay byte-identical. Remove the ignore when green. If the walk is still red, push the new ledger and the next lever in the ignore message. Run the whole walkreal_07000002_live binary and walkreal_07000003_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-02-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-06T21:14Z
