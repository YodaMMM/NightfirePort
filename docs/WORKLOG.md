
## level-11-abseil-hang-gate

- 2026-10-25T04:26Z sandbox6 (continued): branch s/level-11-abseil-hang-gate created from origin/master (e6e3c899). Task: 07000011 — the rope guard p258 slides to y 22.34 with Bond parked at (22.34, 29.37, 3.52), 10 units past the 32.37 hang gate (3.0 above the opponent); the Slide → Hang rule is NDrone2_DSTATE_AbseilSlide 0x169730:57-58 (+0x184 vs +0x674, the 3.0 offset). Log the slide per tick, find why the compare misses at 32.37, fix the port's compare [M], un-ignore stairwell_guards_07000011_live::the_mode_1c_abseiler_hangs_within_the_game_s_3_0_of_the_opponent, keep abseil_death_live / abseil_damage_matrix / the stairwell walks green.
