# WORKLOG

## playscript-deaf-0x168
Started 2026-11-05 05:45 UTC. Branch s/playscript-deaf-0x168 (cut from origin/master a89818235).
Task: scripted guards on 0x6000168 ignore gunfire while their script plays. NDrone2_DSTATE_PlayScript
0x15ba68's ENTER sets +0x4F8 |= 0x400000 on three scripts — the shared tail of 0x6000168 and 0x6000169
(decomp line 73) and the 0x60007ac arm (line 84) — but the port's (PLAY_SCRIPT, msg::SOUND) arm in
drone.rs (~line 7393) tests only 0x6000169 and 0x60007ac, so a 0x6000168 guard breaks out of the script
on gunfire where the game's stays deaf [M 0x177800:19-21]. Confirm against the disassembly at
0x15bb50-0x15bba0; read the three clears (decomp lines 128, 135, 153) so the deaf window ends on the
same messages for all three scripts; fix the comment citing :36-44. List which levels place a guard on
0x6000168 [D]; pin it on one of those guards with real gunfire and pin the guard hears once the script
ends (unit pin if no disc placement). Run cover_nodes_live, fix_06_alarm_live, the ai binaries and
nf-game; suite via tools/test/suite.py in the background + once more on the final tree merged with
origin/master. Sandbox expires ~07:45 UTC.
