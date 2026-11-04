# audit-sp-panes — the single-player HUD panes against the port

The multiplayer pane sweep (docs/reports/audit-mp-panes.md) found real gaps, so do the same for PaneList @0x2be5b8: every pane's create and update function, sprites, positions, colours, enable conditions and writers, read from the decompile and ee_start_05.bin. Check each pane's hide rules against the zoom, vehicle and remote-view panes, and every state of each update's switch. Fix what is missing or wrong where the draw path exists, pin each with a test, and list anything needing new drawing as a table with the template values. Driving panes are out of scope. Merge origin/master first.

---
claimed-by: sandbox4 at 2026-11-04T23:17Z
