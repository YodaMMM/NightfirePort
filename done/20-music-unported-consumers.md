# music-unported-consumers — the music logic the audit tabled

Read docs/reports/audit-music-events.md on origin/s/audit-music-events ("Unported consumers" and "What's left"). Port what a player can hear: the per-track UpdateMFX_* readers of the event board, the cel music-zone channel from Player_Update (cel+0xa1), and any section or fade logic they drive, on-foot levels only. Cite each [M], pin each with a test on the real level's track data, and list what stays out. Merge origin/master first.

---
claimed-by: sandbox5 at 2026-11-04T23:34Z

---
## Report (done, 2026-11-05T00:37Z)

