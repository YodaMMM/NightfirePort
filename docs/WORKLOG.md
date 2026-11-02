## level-0d-yard-sniper-los (2026-11-02)

Branch s/level-0d-yard-sniper-los from origin/master (e79e7bc9).
Goal: un-ignore walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage —
drone #31 (placed sniper, weapon 0x1e) fires the game's law exactly, but the port gives it an
unbroken line of sight to every cell of the yard (x 180-216, z -224..-268; probe_0d_los_grid on
master), so Bond is hit the whole way and dies at tick 6549. Plan: check the yard's occluders
[D placements/collision] and whether the port's drone LOS ray tests them [M the drone
visibility/LOS call and its collide mask]; fix the missing occluder class or wrong mask, else
route the walk through real cover; then run walkreal_0700000d_live, the drone LOS/sight tests,
every real-damage walk, and the full suite.
