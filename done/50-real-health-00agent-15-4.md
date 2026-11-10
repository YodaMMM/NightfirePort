## Your task: `real-health-00agent-15-4`, 07000015 can be walked to the exit on 00 Agent with real health

Branch from origin/s/real-health-00agent-15-3, merge current origin/master, and read that branch's docs/reports/real-health-00agent-15-3.md plus the ignore message on `walk_spawn_to_the_omega_sector_exit_real_damage_00agent`. Its three levers measured negative or unreachable in four runs, and the walk is still red at the 8543 shape. Take its next lever, the meet's pool chain; the hall tail is downstream of it. Do not repeat its measured dead ends. Blocks stay gated on difficulty 3. Real input only, no top-ups, never lower damage or health. Remove the ignore when green. If the walk is still red, push the ledger and next lever in the ignore message. Run the whole walkreal_07000015_live binary and interactables_07000015_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-15-4`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox7 at 2026-11-10T03:11Z

---
## Report (done, 2026-11-10T04:56Z)

