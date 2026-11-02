# level-11-start-fight-real-health-2 — finish 11 on real health

Continue origin/s/level-11-start-fight-real-health (branch from it, merge origin/master in). That session landed the flight-B blocker stand and a full damage-chain audit of the two deaths; the report on the branch names the exact remaining blocker (fight#17: Bond blocked with clip 18 spare 8 — one tactic short).

Do:
1. Apply the tactic the report names for the remaining fight (real input, no health/ammo writes); fix the port only where it differs from the game, with citations.
2. Un-ignore walkreal_07000011_live::walk_spawn_to_the_lower_floor_load_point_real_damage; it must reach the load point. Run walkreal_07000011_live, start_corridor_guard_07000011_live, every walkreal_*_live, then the full suite.

Rules: shared test helper changes (crates/nf-view/tests/common/) must be opt-in per level (and the code must check the flag); re-run every walkreal_*_live before finishing.
