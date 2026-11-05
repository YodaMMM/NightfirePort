# WORKLOG

## music-unported-consumers
Started 2026-11-04 23:38 UTC. Branch s/music-unported-consumers (cut from origin/master f503c15c7).
Task: port what a player can hear from the audit tabled list (docs/reports/audit-music-events.md
on origin/s/audit-music-events): the per-track UpdateMFX_* readers of the event board, the cel
music-zone channel from Player_Update (cel+0xa1), and any section or fade logic they drive,
on-foot levels only. Cite each [M], pin each with a test on the real level's track data, list
what stays out. Merge origin/master first.
Progress 2026-11-05 00:55 UTC. Ported the audit's tabled consumers: the cel music-zone channel
(Player_Update 0x193600:117-122, prop 0x3c → cel+0xa1), TowerC 0x113a20 cases 0-6 (0b out of
the shared exit block — the game's 0b updater has no e[3]/e[8] head), CastleB 0x113060's
NIS-marker half, MayhewB 0x112270 / MayhewC 0x112730 whole, the machines' direct attack-gate
writes, and the restart's SfxNisMusicTrigger clear. The audit's "parser gap" was a
misdiagnosis — the MapMusic fields were already parsed; no parser change needed. 21/0 music
tests, nf-game lib 666/0, all changed-code live binaries green (list in the report). Report:
docs/reports/music-unported-consumers.md.
