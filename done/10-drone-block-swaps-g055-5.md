# drone-block-swaps-g055-5 — the last three walks before G-055 can land

Continue from origin/s/drone-block-swaps-g055-4; merge origin/master first. The FirstAttack and HandleImpact code reads correct against the decompile; do not change it to make a walk pass unless a line is proved wrong. Merged with master these three are red, and they are green on master:
- walkreal_07000003_live::walk_the_full_route_and_complete_every_objective_real_damage
- walkreal_07000011_live::walk_spawn_to_the_lower_floor_load_point_real_damage (panic at common/walker_b2.rs:170)
- walkreal_07000011_live::walk_spawn_to_the_lower_floor_load_point_real_damage_agent (panic at common/walker_b2.rs:641)
For each, say what changed for the guards on that level, then re-route or re-pin the walk with the opt-in levers the 14 and 16 walks used. Run every walkreal binary and the full suite, and report the total line.

---
claimed-by: sandbox1 at 2026-11-04T20:17Z

---
## Report (done, 2026-11-04T21:32Z)

Closed by the driver: the report is on `s/drone-block-swaps-g055-5` (docs/reports/drone-block-swaps-g055-5.md); the worker didn't run finish.sh.
