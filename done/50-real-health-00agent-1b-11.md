## Your task: `real-health-00agent-1b-11`, 0700001b's 00 Agent walk latches all eight consoles and reaches Drake

Branch from current origin/master.

Master e21422e86 (the opt-in walker laser_dodge): Bond SURVIVES the console run at 15.9 health with k5 (3079), k4 (4621), k3 (6211), k8 (6774), k6 (9083 - 1309 past its 7774 deadline) and k7 (11935) latched; k1's first hold is answered at 12187 and missiles m1/m2 launch at ~12376 (deploy [3,3,...], mission status 2) - the consoles STUCK guard ends the leg and the console-1 latch assert fails. Goal: k1/k2 latched before their missiles (time recovered upstream - k6's late latch, the k1 dead-window shelter), then the Drake phase; the walk's 00 Agent test passes.

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

**Branch:** `s/real-health-00agent-1b-11`.

---
claimed-by: sandbox1 at 2026-11-15T07:52Z

Main 11-15 (on e21422e86): k1 before k7 (orders [5,4,3,8,6,1,2,7] and [5,4,3,8,6,1,7,2]): Bond dies at t10424 at (13.96, -1.35, -7.52) on the way to k1 - worse; k7-then-k1/k2 stays.

Main 11-15 (on e21422e86): laser_dodge radii - target 20 u: only k5 latches (the dodge eats the console work); hostile gate 6 u: death t9463. The merged 15/10 stays.

---
## Report (done, 2026-11-15T08:56Z)


Main 11-15 (merged fa191d448): on master all eight consoles latch (k7 9337, k1 9854, k2 10026) at 19.5 health; death t13331 at (8.06, -2.34, 7.19) in the Drake phase.
