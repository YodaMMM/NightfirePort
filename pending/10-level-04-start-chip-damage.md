## Your task: `level-04-start-chip-damage`, the 07000004 start of play hurts Bond as fast as the game does

Bond stands still where play starts on 07000004 (after the intro: (-3.87, 5.46, 30.30) yaw 1.658 in both). At difficulty 1 (GameState+0x28 = 0x2a3790 = 1) the two lose health very differently [L PCSX2, health = BLData+0x894 sampled every 30 frames from the first play frame, no input]:

| s into play | game | port (load_with_carry_difficulty(.., 1), step_bond idle) |
|---|---|---|
| 0.0-0.5 | 100 | 100 |
| 1.0 | 97.5 | 98.5 |
| 2.0 | 97.5 | 86.5 |
| 3.0 | 97.5 | 76.0 |
| 4.5 | 95.0 | 65.5 |
| 8.0 | 70.0 (one 25 hit; Bond turned to yaw 2.558) | 64.0 |
| 12.8 | 68.75 | 35.25 (one 25 hit, Bond lifted to y 5.91) |

The port's chip damage in the first 5 s is about 7x the game's. Find which drones hurt Bond in that window in the port (shooter index, weapon, hit chance roll, range) and in the game (breakpoint the damage path, e.g. the hit-chance 0x179740 / damage 0x1902c8 callers, with Bond's health held only AFTER logging each event), name the arm that differs, fix it against the decompile, and pin it with a test at difficulty 1 that checks the first-5-s damage against these [L] numbers (tolerances stated). The 25-point hit at ~8 s in the game (Bond also turns) vs ~12.8 s in the port is the second item: what lands it, and why the port's comes later and lifts Bond.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-04-start-chip-damage`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
