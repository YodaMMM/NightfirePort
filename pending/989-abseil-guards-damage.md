## Your task: `abseil-guards-damage`, guards on the rope must take damage and die (REPEAT report: 700u4 did not fix it)

User playtest 2026-10-22 (check abseil11m, 07000011, playtest build master 6bbf81b2): "the guards [are] invincible on the rappel". F12: dist/nf-shots/013-07000011.png at `--bond 22.40,24.01,-1.39`, frame 3688: a guard on the rope, bullet holes in the wall behind him.

1. **First a real-path test that FAILS on master**: load 07000011, play to the rappel (or trigger it the way the level does) and fire real rounds at a roping guard through `step_bond` (no damage pokes). Report what happens: do the rounds miss his hit volume (the roped pose vs the hit spheres), or do they hit and the damage is dropped while he is in the abseil state?
2. Fix it against the decompile: the abseil state's damage handling and death (700u4's doc, drone-abseil-elevator task 720) [M].
3. Sister: a guard on the ground still dies the same way.
