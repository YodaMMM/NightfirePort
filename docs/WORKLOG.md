## level-12-start-damage-gap (2026-11-22, s/level-12-start-damage-gap)

Task: the_idle_start_costs_bond_what_the_games_does (ignored on stream batch 1121d): the port
reads 80.75 health at tick 480 and 56.25 at 840; the game reads 53-72 and 0-37 [L].
start_rounds_07000012_live traces every round: no rolled hit passes the 120-tick NewSighting
gate before t296, and each stand/crouch swap holds fire about 90 ticks. Find which gate or
timer differs from the decompile (NewSighting, ChooseCombatMove 0x150c18, the stand/crouch
swap hold) and fix it with citations; un-ignore the test.

### level-12-start-damage-gap progress
