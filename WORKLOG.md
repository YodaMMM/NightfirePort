---
## level-4a-lift-4 (2026-10-24T02:57Z)
- Claimed; branch from origin/master (8df0d8a46). Task: the lift ride's timed satchels (0700004a) — who drops them, the object class + its update (the timer, the ch-0x62 write on expiry), the laser defuse (the 0x4E burn); port them, remove the [S] kill zone, ride-the-lift + burn-each-satchel tests with real input, the undefused sister. Plan: census the level's scripts/drones for the satchel spawners, trace the object class in the decompile, port, then the walkreal ride tests.
