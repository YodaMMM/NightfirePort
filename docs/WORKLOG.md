# Worklog

## level-04-every-objective (branch s/level-04-every-objective, from origin/master c0d3e92e)

Sandbox opened 23:49 UTC, expires ~01:48 UTC (stop new work 01:28). Scope: walk the WHOLE
07000004 mission from a fresh load on real input in walkreal_07000004_live.rs — every mission
objective [D records / Mission_MonitorObjectives 0x1827d8 M] in player order, each asserted
complete, ending on the level exit; blockers are port gaps to fix with [M/D] evidence.

Plan: (1) read the existing test + list covered objectives; (2) mine prior level-04 branches
(audit / mission-complete-early / mission-hold-2) and done/ queue reports for proven intel;
(3) write legs for the uncovered objectives, iterate on real input; (4) full suite, report,
finish.

Progress (01:10 UTC): the coverage list first — the level's mission data lists EXACTLY ONE
objective record: obj0 0x0400000d "Defeat the assassin" (fail none, hint 0x04000015, condition
ch 0x54, enable ch 0x53, flags 0), decoded straight from the EE snapshot [L ee_snapshot.bin
0x2a4350 + row 7*40 → objective table 0x2a3f60, count 1; matches MissionData[7] [M ACTION.ELF
0x2a4350] and crates/nf-game/src/mission/c07000004.rs]. So every-objective for 07000004 = {obj0},
and the existing tests already cover it END TO END on real input: walk_the_full_mission_start_to_
finish walks spawn → intro/story settle (ch 0x53) → obj0 announced (state 3) → StormUzi + BodyArmour
Heavy pickups → the kill → ch 0x54 [M DroneFunc_SetDeathChannel 0x13e660] → OUT mux #46 → ch 100 →
obj0 complete (state 4) [M 0x1827d8:78-85] → "Mission Complete" banner → mission_end Some(0x0730004)
— the level's own exit (no 0xE8 trigger exists [D]). There are no further objectives to leg. Added:
the record-count pin (assert LEVELS[row].objectives.len() == 1 with the [L] cite) + the header
coverage note. walkreal_07000004_live: 5 passed / 0 failed (4.0 s). Sandbox notes: the background
suite dies between tool calls here (same as last session) → chunked foreground runs; the nf-view
link needs LIBRARY_PATH=/home/z/alsa-prefix/usr/lib/x86_64-linux-gnu (the alsa-sys build emits only
the system -L where the dev symlink libasound.so does not exist); disk was rescued by deleting 4.2 GB
of stale test executables in target/release/deps (1075 files from previous sessions). Suite (fast
runner, chunked): at 01:08 UTC, 199/888 binaries recorded — 1564 passed / 4 failed / 45 ignored;
the 4 failures are disc_golden's 3 allowed slim-ISO failures + identity_disc (pins the FULL retail
disc 1,191,072 sectors; the sandbox's on-foot image has 518,435 — environment, not a port gap;
left as-is per "never weaken a disc-backed test").
