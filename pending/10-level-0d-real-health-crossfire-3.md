## Your task: `level-0d-real-health-crossfire-3`, finish 0700000d's walk to the C5 door on real damage

walkreal_0700000d_live `walk_arrival_to_the_c5_door_real_damage` is still #[ignore]d. Read docs/reports/level-0d-real-health-crossfire.md and docs/reports/level-0d-real-health-crossfire-2.md on master. -2 measured the two remaining drains to the hit: the last roof guard's fire when Bond arrives on the deck, and #10's climb shots. It found the hit rate is the game's own law, and its ray-gated sweep still left a 3-shooter fight. So the fix is the route. Use the aimed act (`DamageLog::aimed_fights`) at range. Kill the roof guard from a stand whose static eye-to-chest ray is clear before Bond goes up. Take #10 before or between the climb shots, and use the canopy corner as cover. If the game gives Bond armour or health pickups on this route [D the placements], plan the route through them. Un-ignore the test. Real input only: no health, ammo or channel writes, and no teleports. Never lower damage or health to make it pass. Fix the port only where it differs from the game, with the address. Shared test helpers (crates/nf-view/tests/common/) change only behind a per-level opt-in. Run walkreal_0700000d_live, enemy_accuracy_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-0d-real-health-crossfire-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

