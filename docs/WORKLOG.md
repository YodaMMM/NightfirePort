# WORKLOG

## script-event-channel-writes (branch s/script-event-channel-writes)

Start: 2026-10-30 11:43 UTC, branch from origin/master dca2f368.

Task: the port parses Cmd::Event (op 0x12 [M 0x1d8548's stream walk]) but acts only on kind 0xB;
Script_EventHandler 0x1d94c0's 0xD/0x11 channel-write events are ignored. Port both events exactly
as the decompile does (which channel table, set vs clear, timing within the frame), with a unit
test per event on real disc scripts (07000012 script 06000530 stream 8 [60,60], 06000a25
[137,1]/[137,0], 07000005 06000952 ev 0xD -> ch 0x62). Then run every walkreal binary whose levels
carry these events (sweep the discs) and report each total.
