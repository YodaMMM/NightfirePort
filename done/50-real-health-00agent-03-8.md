## Your task: `real-health-00agent-03-8`, 07000003 can be walked to the end on 00 Agent with real health

Branch from origin/s/real-health-00agent-03-7, merge current origin/master, and read that branch's docs/reports/real-health-00agent-03-7.md plus the ignore message on `walk_the_full_route_and_complete_every_objective_real_damage_00agent`. Take its next lever: make `return_fire` stand down while `self.dmg.resupplying` is true, so a mid-grab Bond finishes the pass's four nearest-first attempts and the leg's re-fight on #3 runs with a real magazine. Then the hip-cone miss at 5.3 u: have the stand close distance while dry, or open the moat answer one volley earlier to shave #17's residual. After that the corridor cluster, the (35.6, 8.7) door stall and the exit-lawn #16 volley, which are still unexercised on 00 Agent. Standing pre-emption is proven worse. New flags are opt-in and default off. Real input only, no top-ups, never lower damage or health. Remove the ignore when green. If the walk is still red, push the ledger and next lever in the ignore message. Run the whole walkreal_07000003_live binary, walkreal_07000002_live, walkreal_07000004_live and interactables_07000003_live, and if a shared helper changes, every walkreal_*_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-03-8`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-08T01:26Z

---
## Report (done, 2026-11-08T03:07Z)

Closed by the driver: the report is on `s/real-health-00agent-03-8` (docs/reports/real-health-00agent-03-8.md); the worker didn't run finish.sh.
