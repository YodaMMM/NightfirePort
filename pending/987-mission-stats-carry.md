## Your task: `mission-stats-carry`, the pause SCORE counters must survive a mission's level change

User playtest 2026-10-22 (check score05r): "Score tab works until you transition to another area and then it resets." The SCORE tab (bonus-007-2, now on master) reads the progress layer's live counters (Bond Moves, Dispatched, Subdued, Surrendered, 007 Bonus, plus shots/accuracy/time). On The Exchange, going 07000005 → 07000006 zeroes them.

1. Find in the decompile what the game keeps across a mission's level change: PlrMissionStats (docs/fields/player-stats.md). Who zeroes it (mission start vs every level load: `ResetMap_Load`, `Player_Init`, the `PlrStat_*` init) [M], and what does the SCORE tab's `C_GCPAUSE_Handler` 0x204408:447-498 read?
2. Make the port carry exactly what the game carries across the level change, and reset exactly where the game resets (a new mission, a restart).
3. Real-path test: on 07000005, earn a Dispatched and a Bond Move, cross into 07000006 through the real level exit, and the counters still hold. Sister: restarting the mission zeroes them, if the game does [M].
