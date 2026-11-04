# WORKLOG

## music-unported-consumers
Started 2026-11-04 23:38 UTC. Branch s/music-unported-consumers (cut from origin/master f503c15c7).
Task: port what a player can hear from the audit tabled list (docs/reports/audit-music-events.md
on origin/s/audit-music-events): the per-track UpdateMFX_* readers of the event board, the cel
music-zone channel from Player_Update (cel+0xa1), and any section or fade logic they drive,
on-foot levels only. Cite each [M], pin each with a test on the real level's track data, list
what stays out. Merge origin/master first.
