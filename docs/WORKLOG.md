# WORKLOG

## script-event-channel-writes (branch s/script-event-channel-writes)

Start: 2026-10-30 11:43 UTC, branch from origin/master dca2f368.

Task: the port parses Cmd::Event (op 0x12 [M 0x1d8548's stream walk]) but acts only on kind 0xB;
Script_EventHandler 0x1d94c0's 0xD/0x11 channel-write events are ignored. Port both events exactly
as the decompile does (which channel table, set vs clear, timing within the frame), with a unit
test per event on real disc scripts (07000012 script 06000530 stream 8 [60,60], 06000a25
[137,1]/[137,0], 07000005 06000952 ev 0xD -> ch 0x62). Then run every walkreal binary whose levels
carry these events (sweep the discs) and report each total.

Outcome 12:30 UTC: the 0xD/0x11 can't-clear rule landed — the game's events return on a zero
value BEFORE the store [M 0x1d94c0 case 0xd disasm 0x1d9758 beqz, case 0x11 disasm 0x1d9798
beqz; raw words 10600034 a0440000 / 10800024 a0620000 — the decompiled C drops both returns,
which the merged script-channel-events task had pinned as "the write happens, value 0"]. The
arms gated on the u32 value; the wrong lib test corrected with the address; the 0x11 p3==0
no-write pinned; the per-event unit tests on the real disc scripts landed (script_channel_
writes_live: the cited rows decoded from the ISO — 06000530 s8 [3c,3c] with its four event-3
holds, 06000a25 [89,1]/[89,0], 06000952 s8 [62,1], 06000a23 s1 the one 0x11 [98,98,1d,1] — each
driven through the real Script::update). Disc sweep re-run: 199 rows, 15 levels, 14 zero-value
rows. All 16 walkreal binaries over the event levels green (58 passed / 0 failed / 2
pre-existing ignores); the one walk the fix changed — 07000016's launch-chain [180,0] clear
pin — corrected with the address. nf-game full test suite green. Report:
docs/reports/script-event-channel-writes.md.
