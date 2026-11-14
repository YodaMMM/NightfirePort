# 50-real-health-00agent-15-9

07000015's 00 Agent real-damage walk (`walkreal_07000015_live::walk_spawn_to_the_omega_sector_exit_real_damage_00agent`)
now crosses the fence leg and dies at the exit corridor's door[51] threshold, t11681 at (0.08, 41.31, 11.27):
health 5.4 -> 0.0 to #12 (skin 0x500008c, 0x1a launcher at (5.4, 41.3, -1.9)), short by 0.04.

Read the ignore text on master (fdbdbe16e) for the full ledger. Levers in order:
1. #4 (the second 0x5000087 plank chaser) still pays 15.0 in 5 during the hall crossing: loop the chaser stand
   until the skin finder returns None, or widen the spot to the converging path (28-35, 33-38), tol 20.
2. #12: a pre-emptive aimed stand from the door[51] threshold before the last route legs.
Keep every change gated `difficulty >= 3 && !w.tops_up()`; the d1/d2 walks must stay byte-identical.
Tests: walkreal_07000015_live, interactables_07000015_live.

Measured 11-13 (master 42aff3fcb): lever 1 (loop the 0x5000087 chaser stand, also widened to (31.5,35.5) tol 20) changes nothing - the finder returns None after #17, #4 is not near yet (62.3 health after the stands, death t11681 unchanged). Lever 2 (visible-guard stand at the door[51] threshold, radius 20/30) kills #12 then #14 but dies sooner, t11654 - 5.4 health cannot stand anywhere. The gap is upstream: health into the hall.
