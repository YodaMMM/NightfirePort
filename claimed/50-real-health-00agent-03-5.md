## Your task: `real-health-00agent-03-5`, 07000003 can be walked to the end on 00 Agent with real health

`walk_the_full_route_and_complete_every_objective_real_damage_00agent` in crates/nf-view/tests/walkreal_07000003_live.rs is still ignored. Read two reports first: the "Report from the worker" in the queue's blocked/50-real-health-00agent-02-03-4.md (on origin/s/real-health-00agent-02-03-4, not merged), and docs/reports/door-walk-03b.md on origin/s/door-walk-03b. Branch from origin/master, then merge origin/s/door-walk-03b if it is not on master, and take the difficulty-3 work from s/real-health-00agent-02-03-4 only where it still helps. door-walk-03b turned the Agent sister green with two changes gated on `difficulty >= 2`: the resupply arms before the rescue, each enable plate and each servant walk, and the east-lawn saddle pre-clear at (23.5, 6.3, 8.4). The earlier 00 Agent runs predate both, so re-run on that tree first and read the new death ledger before changing anything. The levers left from the blocked round, in order: start the corridor exchange at door 9 from high health, cutting the mid-courtyard #9 fight (32.4 at 00 Agent); then the (35.6, 8.7) door stall and the exit-lawn #16 volley, which are still unexercised at this difficulty. If the ledger shows a far-shooter bleed, door-walk-03b names `stand_shooters_down` as the ready lever. Real input only, no top-ups, never lower damage or health, and keep the Agent and easiest walks byte-identical by gating on difficulty 3. Remove the ignore when green. If the walk is still red, push the ledger and the next lever in the ignore message. Run the whole walkreal_07000003_live binary, walkreal_07000002_live, walkreal_07000004_live and interactables_07000003_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-03-5`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-06T22:15Z
