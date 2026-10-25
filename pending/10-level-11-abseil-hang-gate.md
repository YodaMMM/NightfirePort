## Your task: `level-11-abseil-hang-gate`, 07000011: the rope guard stops at the game's hang height

`stairwell_guards_07000011_live::the_mode_1c_abseiler_hangs_within_the_game_s_3_0_of_the_opponent` is ignored: with Bond parked at (22.34, 29.37, 3.52), rope guard p258 slides to y 22.34 before hanging — 10 units past the 32.37 gate (3.0 above his opponent) — so he hangs mid-well beside the lower landings. The Slide -> Hang rule is `NDrone2_DSTATE_AbseilSlide` 0x169730: the opponent pointer +0x170 (Bond every tick in SP, `NDrone2_FindOpponent` 0x1424e0:100-120) and the height compare at :57-58 (+0x184 vs +0x674 and the 3.0 offset).

1. Log the slide per tick (guard y, the compared values, the state) and find why the compare does not fire at 32.37 [M — which heights the game compares (the guard's root vs the opponent's root/eye/+0x30), the per-tick slide step, any overshoot guard]. Fix the port to the game's compare.
2. Un-ignore the test; keep `abseil_death_live`, `abseil_damage_matrix` and the stairwell walks green.

**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-11-abseil-hang-gate`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.
