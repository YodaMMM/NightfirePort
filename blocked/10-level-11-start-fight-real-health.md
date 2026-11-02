# level-11-start-fight-real-health — Bond dies early on 11's real-health walk

Branch from origin/master.

walkreal_07000011_live::walk_spawn_to_the_lower_floor_load_point_real_damage is ignored: Bond dies at tick 3193 at (10.1, 1.1, -13.8); the last hits are -3.00 x2 from drone #10 (skin 0x5000008, weapon 0x1c) at (19.5, 1.0, -11.4), 9.7 u.

Do:
1. Read the whole damage log for the run (who hit, how often, from where). Compare the start guards' fire rate, accuracy and damage with the game [M the drone weapon code for 0x1c; start_corridor_guard_07000011_live's [L] pins are the game's own timings].
2. Fix the port where it differs (cite it); otherwise give the walk a player's tactic (cover, kill order, the level's pickups). No health writes.
3. Un-ignore the walk; run walkreal_07000011_live, start_corridor_guard_07000011_live, every real-damage walk, then the full suite.

---
claimed-by: sandbox7 at 2026-11-02T19:35Z

---
## Report (blocked, 2026-11-02T20:32Z)

