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

Progress (02:58 UTC): found BOTH Records bugs in the 0x20bf00 read. (1) The medal arms were
swapped: the game's all-four-flags arm loads 0x306/0x300013f and the k1..k3 arm 0x307/0x300013e
[M disasm 0x20c0e4-0x20c154; the results page pairs platinum→0x300013f / gold→0x300013e,
M 0x208e78:109-139] — the port showed GOLD after platinum (and platinum after gold). FIXED in
record_medal. (2) The "Upgrades" census: the game counts only the kind-4 (upgrade-group)
counters with the bit standing [M 0x20bf00:34-44; every RewardsTable row keeps kind 4 at
counter 3, L 0x2bc920] — the port counted every earned medal (4 after platinum, game 1).
FIXED in record_row. Pinned END TO END: new the_platinum_finish_shows_platinum_on_records
walks 07000004 at 007 through the real input path (the autoaim assist drops at 007 [M
0x1a73f8:127-149], so the walker resupplies at the respawn-2 StormUzi placements [D]) →
committed 1,113,030 ≥ 1,000,000 → results medal 4/0x300013f → Records 0x306/0x300013f and
census 1. Two test sites fixed per the discipline (walkreal_07000004's `want` carried the
same swap; fix_save_profile's census counted every medal). walkreal_07000004 7/7,
fix_save_profile 7, save_progress_roundtrip 4, dossier_pages 2 — all green. FULL suite
PARTIAL: 186/908 binaries, 660 passed, 0 failed, 47 ignored (chunk_suite; the 2 mid-run
failures were the previous task's untracked scratch_*.rs files, moved out of the worktree
and dropped from the tally); ~722 binaries remain — resume chunk_suite with the same state.
Report: docs/reports/records-medal.md.
