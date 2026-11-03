## Your task: `level-0d-real-health-crossfire-2`, finish 0700000d's walk to the C5 door on real damage

Continue origin/s/level-0d-real-health-crossfire: branch from it, merge origin/master in, and read its report docs/reports/level-0d-real-health-crossfire.md. That session fixed the crossfire's port-side over-hit [M] and moved survival from tick 4177 to 9412. Two things still drain the health pool: the last roof guard's fire when Bond arrives on the deck, and #10's shots during the climb. For each, check the wake, the sight and the hit rate against the game [M the sight, accuracy and damage paths; D coll-scan for the rays]. Fix the port only where it differs. Otherwise change the walk's tactics: kill the roof guard before the deck arrival, climb between #10's shots, or take cover. Un-ignore walkreal_0700000d_live `walk_arrival_to_the_c5_door_real_damage` on real input only. Never lower the damage or health to make it pass. Run walkreal_0700000d_live, enemy_accuracy_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0d-real-health-crossfire-2`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

