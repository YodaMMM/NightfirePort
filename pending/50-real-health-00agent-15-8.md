## Your task: `real-health-00agent-15-8`, 07000015's 00 Agent spawn-to-exit real-damage walk goes green

Branch from origin/s/real-health-00agent-15-7 (its ledger is not on master yet) and read docs/reports/real-health-00agent-15-7.md there plus the refreshed ledger and ordered next levers in the ignore message on walk_spawn_to_the_omega_sector_exit_real_damage_00agent. The head-first desk stands and the door[32] lob both measured as working; take the next levers in the order the ignore message gives (starting with the fights' dry-mag resupply nav), judging each on its own real run. Helpers stay gated to 07000015 and difficulty 3, so the d1/d2 walks and replays stay byte-identical. Real input only, no top-ups, never lower damage or health. Remove the ignore when green; if still red, push the refreshed ledger and the next lever in the ignore message. Run the whole walkreal_07000015_live binary and interactables_07000015_live.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-15-8`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

