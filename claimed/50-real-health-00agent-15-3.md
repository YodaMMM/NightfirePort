## Your task: `real-health-00agent-15-3`, 07000015 can be walked to the exit on 00 Agent with real health

Branch from origin/s/real-health-00agent-15-2, merge current origin/master, and read that branch's docs/reports/real-health-00agent-15-2.md plus the ignore message on `walk_spawn_to_the_omega_sector_exit_real_damage_00agent`. The run now dies at tick 8543 in the great hall. Take its levers in order: first the #15 meet, the largest toll at 42.0, with a pre-aimed P2K shot at his advance spot or a meet timed like round 2's; then the west-room lobs (24.5), with a bounded first exchange at 8 u or less; then the 733 flank, from the bridge's east side or a frag onto (6.25, 35.69) from the door[50] threshold. Blocks stay gated on difficulty 3, so the d1 and d2 paths stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green. If the walk is still red, push the ledger and next lever in the ignore message. Run the whole walkreal_07000015_live binary and interactables_07000015_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-15-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-08T02:53Z
