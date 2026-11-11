## Your task: `real-health-00agent-16-1b-4`, 07000016 and 1b can be walked on 00 Agent with real health

Branch from origin/s/real-health-00agent-16-1b-3, merge current origin/master, and read that branch's docs/reports/real-health-00agent-16-1b-3.md plus the two 00 Agent ignore messages. For 1b, decode which gate each missile's branch read consumes, and at what tick after launch, from the missile scripts' p0 streams [D], then re-balance the per-console caps against those real read times. Its (e) shape is one missile short with health to spare; the latch ledger to beat is in the report. The Drake phase's y > 9 spawn-band re-weight comes after. For 16, try the cheap lever first: enable the d1-gated `finish_fights` and `level_fights` for d3's ring legs, or raise the ring-gated clear_hostiles cap, then verify the chaser, pit and finale phases whole. Do not repeat its measured dead ends. Real input only, no top-ups, never lower damage or health. Remove each ignore when green. If a walk is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000016_live and walkreal_0700001b_live binaries.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-16-1b-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-11T03:09Z
