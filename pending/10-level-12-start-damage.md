## Your task: `level-12-start-damage`, the 07000012 start spot hurts Bond as hard as the game does

Bond stands still at the 07000012 start (22.49, 1.09, 19.16) from the level load, no input, difficulty 1 (0x2a3790 = 1). Health (BLData+0x894) every ~120 frames from the load [L PCSX2, three runs] vs the port (load_with_carry_difficulty(.., 1), step_bond idle):

| frame | game run 1 | game run 2 | game run 3 | port |
|---|---|---|---|---|
| 0-240 | 100 | 100 | 100/98 | 100 |
| 360 | 100 | 86 | 93 | 90 |
| 480 | 72 | 53 | 68 | 79 |
| 600 | 51 | 35 | 48 | 76 |
| 720 | 44 | 18 | 32 | 74 |
| 840 | 37 | 0 | 21 | 58 |

The game's guards land the opening hits about twice as hard or as often (frames 360-840: game -49..-86, port -32). Find which drones hurt Bond here in the port and in the game (shooter, weapon, burst length, hit-chance roll / damage per hit at difficulty 1 - e.g. breakpoint the damage path 0x1902c8 and the hit-chance 0x179740 in the game and log each event), name the arm that differs, fix it against the decompile and pin the first-14-s damage at difficulty 1 against these [L] numbers (state the tolerance; the game's spread across runs is real). Sister task: 10-level-04-start-chip-damage (there the port hurts MORE) - check whether one cause explains both.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/10-level-12-start-damage`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
