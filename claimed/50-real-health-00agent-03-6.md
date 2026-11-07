## Your task: `real-health-00agent-03-6`, 07000003 can be walked to the end on 00 Agent with real health

Branch from origin/s/real-health-00agent-03-5, merge current origin/master, and read that branch's report in the queue's blocked/50-real-health-00agent-03-5.md plus the ignore message on `walk_the_full_route_and_complete_every_objective_real_damage_00agent`. The walk now reaches t404475. Its main losses are #17's point-blank burst (60.0 in 4 volleys, ticks 14353-14576) and #9 (28.8, ticks 7734-7845), both inside the fights' 240-tick post-fight cooldown. Standing pre-emption is already proven worse, so do not repeat the runs that report lists as reverted. Build the lever it names: an opt-in walker.rs fights flag that re-engages at once when the cooldown would delay refighting a guard within 8 u who is hitting Bond. It must default off, so every other walk stays byte-identical. Then read the new ledger and take the corridor cluster, the (35.6, 8.7) door stall and the exit-lawn #16 volley in the order the ledger gives. Real input only, no top-ups, never lower damage or health. Remove the ignore when green. If the walk is still red, push the ledger and next lever in the ignore message. Run the whole walkreal_07000003_live binary, walkreal_07000002_live, walkreal_07000004_live and interactables_07000003_live, and because walker.rs changes, every walkreal_*_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-03-6`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox3 at 2026-11-07T02:53Z
