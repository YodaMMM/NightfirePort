## Your task: `real-health-00agent-02-6`, 07000002 can be walked to the end on 00 Agent with real health

Branch from origin/s/real-health-00agent-02-5, merge current origin/master, and read that branch's report in the queue's blocked/50-real-health-00agent-02-5.md plus the ignore message on `walk_the_castle_all_objectives_real_health_00agent`. Take its levers in order: first the sniper duel at the lane mouth (if the aimed stand gets no window before shot 2, run the probe pass along the lane printing `exposed_aim(9)` per u), then the health headroom (the k=0/k=1 gap of one volley against the closing #7), then the crossing ammo (verify the lip-drop take lands and the fight runs on 15-27 rounds). Do not repeat runs that report lists as measured dead ends. Real input only, no top-ups, never lower damage or health, and gate the changes on difficulty 3 so the other walks stay byte-identical. Remove the ignore when green. If the walk is still red, push the ledger and next lever in the ignore message. Run the whole walkreal_07000002_live binary and walkreal_07000003_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-02-6`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-07T05:00Z
