## Your task: `real-health-14-00agent-8`, 07000014 can be walked to the exit on Agent and 00 Agent with real health

Branch from origin/s/real-health-14-00agent-7, merge current origin/master, and read that branch's docs/reports/real-health-14-00agent-7.md plus the ignore message on `walk_the_infiltration_to_the_delta_exit_real_damage_00agent`. The walk now reaches the Rook approach at tick 17400 and is one health budget away, with 22.8 at the escape touch. Take the levers in order: the Rook-approach leg's far fights, scoped per fight (a global narrow window is measured as forbidden), then the block's remaining 42.5, where the first #56 engagement still costs 20.0. Then the d2 wrap, which needs the drop-west pocket crossing to stop re-rolling. New flags are opt-in and default off. Real input only, no top-ups, never lower damage or health. Remove the ignore when green. If the walk is still red, push the ledger and next lever in the ignore message. Run the whole walkreal_07000014_live binary and interactables_07000014_live, and if a shared helper changes, every walkreal_*_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-14-00agent-8`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox5 at 2026-11-08T03:08Z
