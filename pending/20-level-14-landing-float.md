# 14: Bond floats over the landing

`walkreal_07000014_live walk_the_infiltration_to_the_delta_exit_real_damage_00agent` wedges at tick 26340: Bond floats 0.41 over the floor at (33.84, 0.82, 30.95) on the 1.72 landing resupply leg. The earlier task on this closed with no report and no code.

Treat the float as a possible game-side bug first. Put Bond at that spot on master, settle, and print grounded, the floor ray and the capsule contacts. If he hovers there without the walk, fix the collision and cite the decompile. If only the walk causes it, say which leg and fix the route. Remove the ignore when green and quote the whole binary.
