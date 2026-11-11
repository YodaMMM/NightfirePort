## Your task: `real-health-14-00agent-10`, 07000014 can be walked to the delta exit on 00 Agent with real health

Branch from origin/s/real-health-14-00agent-9, merge current origin/master, and read its docs/reports/real-health-14-00agent-9.md plus the ignore message on walk_the_infiltration_to_the_delta_exit_real_damage_00agent. Pin the descent's tick count first (lever 4: the drop pocket re-rolls #6's charge state with every upstream timing change), so you tune against one state. Then take its ordered levers: arm through switch_to_armed at the landing and take the head round at the closer from the pad west; then the pad standoff on the walker's own leg machinery; then the unseen-cover opt-in scoped to the descent's legs. Judge each over several real runs. Helpers stay gated to 07000014 and difficulty 3, so other walks stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if it is still red, push the ledger and next lever in its ignore message. Run the whole walkreal_07000014_live binary and interactables_07000014_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-14-00agent-10`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

