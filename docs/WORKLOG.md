## level-11-start-fight-real-health-3 (started 2026-11-03 11:30 UTC)

Branch cut from origin/master 01917076. Task: walkreal_07000011_live
`walk_spawn_to_the_lower_floor_load_point_real_damage` (~line 295) still #[ignore]d — Bond dies at
tick 3193 at (10.1, 1.1, -13.8) to drone #10 (weapon 0x1c at 9.7 u). Fight #17, Bond blocked with
clip 18 + spare 8, one tactic short. Plan: (1) read blocked/10-level-11-start-fight-real-health{,-2}.md
and the two prior reports, (2) merge the newest prior branch not on master + origin/s/player-upward-shot,
(3) check whether the stands fire from the hip, (4) play the fights aimed (kill order + cover a player
would use) via the opt-in `DamageLog::aimed_fights`, (5) fix the port only where it differs from the
game, with the address, (6) un-ignore the test on real input only, run walkreal_07000011_live,
start_corridor_guard_07000011_live, nf-game, and the full suite via tools/test/suite.py.
