# WORKLOG

## level-13-spawn-to-end-walk (branch s/level-13-spawn-to-end-walk)

Start: 2026-10-30 06:40 UTC, branch from origin/master 0c90eba8.

Task: one walk that loads 07000013 the way the game does after 4A (RamSave carry [M Player_RamLoad 0x1957a8]) and reaches the level's end on real input, every objective set by the game's own writers (no channel pokes), pinning 0x04000035 "Escape from the tower" complete (writer: whole-mission AND ch 100 [M 0x1827d8:119-123], first set on 07000013).

Progress 07:45 UTC: the walk already reached the mission end (level-13-every-objective's exit
leg); this task's gap is part 0's records. Extended `walk_the_warm_chain_to_the_front_entrance_and_out`
to pin them: 0x04000036 (states[15][1]) folded on the warm chain (ch 0x48 written by 12's spawn
touch #679 [D]); 0x04000035 "Escape from the tower" (states[15][0]) still announced (3) at 13's
load — its writer is the whole-mission AND ch 100 [M 0x1827d8:119-123] that first fires here —
then COMPLETE (4) after the success touches write ch 0x64 = 100, plus ch 100 raised by the
monitor's all-AND and all six mission records complete at the latch. No port change yet — the
monitor already ANDs every record and writes ch 100 [M 0x1827d8:119-123]; the test run decides.

Outcome 09:15 UTC: the extended walk is GREEN —
`walkreal_07000013_live::walk_the_warm_chain_to_the_front_entrance_and_out` 1 passed / 0 failed
(8.05 s, re-verified twice). The pin held with NO port change: the port's
`Mission_MonitorObjectives` already ANDs every record and writes ch 100 [M 0x1827d8:119-123],
so once 07000013's success touches #73/#164/#165 write ch 0x64 = 100 the whole-mission AND folds
0x04000035 3 → 4 the same tick. Full-suite run incomplete at the sandbox deadline (see the
report): 664/915 binaries recorded, 1510 passed, 3 failed (2 documented no-GPU pylon env
failures + net_internet_live), 173 ignored, ~160 OOM kills from 4-way parallelism (my runner's
fault, not the tree's), ~250 unrecorded. State: /home/z/suite13_state.json (persists for the
next session; rerun the rc=-9/124 entries solo).
