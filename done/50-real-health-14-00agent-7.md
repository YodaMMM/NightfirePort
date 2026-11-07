## Your task: `real-health-14-00agent-7`, 07000014 can be walked to the exit on Agent and 00 Agent with real health

Branch from origin/s/real-health-14-00agent-6, merge current origin/master, and read that branch's docs/reports/real-health-14-00agent-6.md plus the ignore message on `walk_the_infiltration_to_the_delta_exit_real_damage_00agent`. Take "What's left" in order. First the fights' pick question: the corridor block's hunters (skin 0x5000087 and the 0x5000090 south block) are never picked by `DamageLog::shooters` or the lane pick, while they chase Bond and burst 30.0-65.0 in the cooldown gaps. Add a pick-level trace that prints why the nearest 3.6 u shooter is not a candidate, and fix the pick behind an opt-in flag that defaults off, so every other walk stays byte-identical. Then the d2 wrap, once the drop-west pocket crossing stops re-rolling. Keep the head-shot law that cut the trench guard's trade from 65.0 to 15.0. Real input only, no top-ups, never lower damage or health. Remove the ignore when green. If the walk is still red, push the ledger and next lever in the ignore message. Run the whole walkreal_07000014_live binary and interactables_07000014_live, and if a shared helper changes, every walkreal_*_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-14-00agent-7`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox4 at 2026-11-07T14:18Z

---
## Report (done, 2026-11-07T14:59Z)

Closed by the driver: the report is on `s/real-health-14-00agent-7` (docs/reports/real-health-14-00agent-7.md); the worker didn't run finish.sh.
