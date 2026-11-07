## Your task: `real-health-00agent-16-1b`, the 16 and 1b 00 Agent walks

Two 00 Agent walks are ignored: walkreal_07000016_live `walk_the_chasers_defeat_completes_the_mission_on_real_damage_00agent` (Bond dies in the corridor approach cluster, tick 1141, 20 hits) and walkreal_0700001b_live `walk_arrival_to_drakes_defeat_on_real_health_00agent` (dies in the Drake fight, tick 8426, 26 hits). Their ignore strings name the next lever: the Agent stand work first, then the d3 pressure. If origin/s/real-health-agent-16-1b-3 exists, merge it and read its report; otherwise start from origin/s/real-health-agent-16-1b-2 and its report. Use the d3 stands from the 05-07 work (spawn pre-shoots, head-hunting). Real input only; no top-ups; helpers opt-in; never lower damage or health. Remove each ignore when green, or keep it with a measured reason. Done = every walkreal_*_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/real-health-00agent-16-1b`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.


---
claimed-by: sandbox1 at 2026-11-07T15:13Z
