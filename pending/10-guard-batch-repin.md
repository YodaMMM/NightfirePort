# Guard radii and glass: last reds

Branch from `origin/s/guard-batch`, push to `s/guard-batch-repin`. It carries the per-goal arrival radii and guard rounds through glass, both reviewed against the decompile. Suite: 4399 passed, 8 failed.

Look at these two first, they are not walks: enemy_accuracy_live astronauts_hit_like_the_chain ("diff 2 d 5: 1 shots in 10 s, interval 180": check whether the astronaut's +0xc4 mode 0x1d change altered his firing) and kiko_button_real_walk_live ("the pit squad member #14 dies to Bond's fire"). If either is a game-code fault, fix it with a citation.

Then the walks, test-side: walkreal 16 real_damage, 01 00agent, 03 real_damage, 13 real_damage, 08 00agent (the shoot-out must kill every end pane), 06 real_damage. Do not loosen a pin tagged [L]. Run each binary whole and quote results.
