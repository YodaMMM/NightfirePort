## Your task: `real-health-00agent-03-captors-7`, 07000003's 00 Agent full-route walk completes every objective

Branch from current origin/master.

Master: death t74135 at the east lawn saddle (23.18, 5.50, 7.20): Bond arrives with the P2K 3+13 (the only rounds left; every 0x12/0xe/0x1c drop near the route expires on its lifetime before a loot pass), the saddle wake sweep spends them, and #7 (0x5000039, 0x12, hp 4) at (11.4, 10.1, -16.1) lands 1.8 a hit from 26.5 u. Measured: a wider loot radius, a dry-wheel cycle skip, the saddle armour (behind the door (22.3, 10.9)), stands from (14..20, 0) - no gain; the captors-5/6 branch kills #7 but dies earlier (t67488). Goal: rounds that survive to the saddle (or the saddle crossed without the sweep) and the walk's 00 Agent test passes.

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

**Branch:** `s/real-health-00agent-03-captors-7`.

---
claimed-by: sandbox1 at 2026-11-15T05:34Z

---
## Report (done, 2026-11-15T06:51Z)

