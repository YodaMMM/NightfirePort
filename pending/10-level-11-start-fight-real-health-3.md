## Your task: `level-11-start-fight-real-health-3`, finish 07000011 to the lower-floor load point on real damage

walkreal_07000011_live `walk_spawn_to_the_lower_floor_load_point_real_damage` (~line 295) is still #[ignore]d: Bond dies at tick 3193 at (10.1, 1.1, -13.8) to drone #10 (weapon 0x1c at 9.7 u). Read blocked/10-level-11-start-fight-real-health.md and blocked/10-level-11-start-fight-real-health-2.md on the queue branch, and the reports on origin/s/level-11-start-fight-real-health and its -2. They name fight #17, where Bond is blocked with clip 18 and spare 8, one tactic short. Branch from origin/master. Merge in the newest of those two branches that is not on master, and origin/s/player-upward-shot (the opt-in aimed act, `DamageLog::aimed_fights`: hip fire scatters by the weapon's spread base [M Bullet_init 0x12c5a0:139-170]). First check whether the stands fire from the hip. Then play the fights aimed, with the kill order and cover a player would use. Fix the port only where it differs from the game, with the address. Un-ignore the test on real input only. Run walkreal_07000011_live, start_corridor_guard_07000011_live and nf-game.


**Tests pin only proven behaviour.** Every assertion about what the game does cites its evidence: [M] a decompile
address that actually says it, [D] disc bytes, or [L] a live PCSX2 measurement from a FRESH boot (a save-state resume
skips the boot path). A guess, a port choice or an [S] item is never a hard assertion: mark it `// [S] unverified` and
keep it out of pass/fail. If an existing test contradicts the decompile, the TEST is wrong: fix it, with the address, and
say so in the report. Never weaken a decompile-backed test to make code pass.

**Run the crates your tests touch before pushing**, and report what you ran.

**Branch:** `s/level-11-start-fight-real-health-3`.

- **Run the full suite with the fast runner, started in the background in your first minutes** (it fits the 2-hour window):
  `NF_ISO=<abs path> nohup python3 tools/test/suite.py > suite.out 2>&1 &` (docs/tools/test-suite.md; once `tools/test/suite.py`
  is on your branch: merge `origin/master` first). Read `tail -1 suite.out` and its FAILED/TIMEOUT lines before you push, and
  run it once more on your final tree (merged with `origin/master`). Report the total line.

