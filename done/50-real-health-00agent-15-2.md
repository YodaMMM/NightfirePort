## Your task: `real-health-00agent-15-2`, 07000015 can be walked to the exit on 00 Agent with real health

Branch from origin/s/real-health-00agent-15, merge current origin/master, and read that branch's docs/reports/real-health-00agent-15.md plus the ignore message on `walk_spawn_to_the_omega_sector_exit_real_damage_00agent`. Take the report's "What's left" in order: the finisher's leg-stop nudge, the plank stand at door[39]'s north side (34.6, 41.4, 28.0) run the moment door[39] opens, the #462 shot at (int)d >= 14, then the #526 lob's green run. By that ledger the budget closes at about 85-90 if the crossing fight lands at 21 or less with no grenade. Do not repeat the runs it lists as dead ends. Blocks stay gated on difficulty 3, so the other paths stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green. If the walk is still red, push the ledger and next lever in the ignore message. Run the whole walkreal_07000015_live binary and interactables_07000015_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-15-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-07T14:17Z

---
## Report (done, 2026-11-07T15:12Z)

Closed by the driver: the report is on `s/real-health-00agent-15-2` (docs/reports/real-health-00agent-15-2.md); the worker didn't run finish.sh.
