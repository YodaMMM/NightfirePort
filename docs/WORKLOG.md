## records-medal (started 01:22 UTC 2026-10-30)

Task: the Dossier Records page shows gold after a platinum Double Cross Part 4 finish.
Find how the game picks the medal shown on Records [M C_RBDSRECORDS_Handler 0x20bf00,
PlrStats_GetLevelTotals, the medal thresholds per level / difficulty, the reward counter
PlrStarts_ProcessRewardCounter] and what the results page uses for platinum; fix the port
so Records shows the same medal the results awarded. Pin with a real 07000004 finish at
platinum. FULL suite, quote its total line.

Plan: branch pushed from origin/master (21fbeed3) now; then read the Records handler and
PlrStats decomp, compare with the port's records screen, fix, pin with a test through the
real input path.
