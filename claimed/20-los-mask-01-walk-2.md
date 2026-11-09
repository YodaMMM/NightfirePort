## Your task: `los-mask-01-walk-2`, every level's walks stay green with the game's line-of-sight masks

Branch from origin/s/los-mask-01-walk, merge current origin/master, and read its docs/reports/los-mask-01-walk.md. Take "What's left" in order. First the two unledgered 08 failures: one captured rerun names them, then re-tune the gated descent. Then the walkreals it did not run (09, 0a, 0b, 0c with barrelrow and eastroute, 0d, 11 with route, 12, 13, 14, 15, 16, 1b, 41 with wall, 4a with satchel), one level per tool call, and repair any walk the masks shift, gated to its level. Expect masked-timeline wedges on routes with timed door or nav moves. Then the ai, turret and glass binaries one `-p nf-view --test <name>` at a time, which avoids cargo's name clash with nf-game's ai_live. The mask rule is the game's and stays. Real input only, no top-ups, never lower damage or health. Remove each ignore when green; if a walk is still red, push the ledger and next lever in its ignore message.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/los-mask-01-walk-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-09T18:47Z
