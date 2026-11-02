## level-11-start-fight-real-health (2026-11-02)

Branch s/level-11-start-fight-real-health from origin/master (e79e7bc9).
Goal: un-ignore walkreal_07000011_live::walk_spawn_to_the_lower_floor_load_point_real_damage —
Bond dies at tick 3193 at (10.1, 1.1, -13.8), last hits -3.00 x2 from drone #10 (skin 0x5000008,
weapon 0x1c) at (19.5, 1.0, -11.4), 9.7 u. Plan: read the full damage log; compare start guards'
fire rate/accuracy/damage vs game [M drone weapon code 0x1c; start_corridor_guard_07000011_live
[L] pins]; fix port where it differs, else route the walk with real tactics (no health writes);
run walkreal_07000011_live, start_corridor_guard_07000011_live, all real-damage walks, full suite.
