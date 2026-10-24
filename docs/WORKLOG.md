# WORKLOG

## 987-mission-stats-carry

- Task: the pause SCORE tab counters must survive a mission's level change (07000005 → 07000006).
- Branch `s/987-mission-stats-carry` created from `origin/master` (26543436).
- Plan: (1) find in the decompile what PlrMissionStats holds and who zeroes it (mission start vs
  every level load: ResetMap_Load, Player_Init, the PlrStat_* inits) and what the SCORE tab's
  C_GCPAUSE_Handler 0x204408:447-498 reads; (2) make the port carry exactly that across the level
  change and reset exactly where the game resets; (3) real-path test: earn a Dispatched and a Bond
  Move on 07000005, cross into 07000006 through the real level exit, counters hold; sister:
  mission restart zeroes them if the game does.
- Note: the driver's task tail says "Branch: s/mission-stats-carry" but the header, task id and
  finish command all say s/987-mission-stats-carry; using s/987-mission-stats-carry.
