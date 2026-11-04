# audit-mp-panes — the multiplayer HUD panes against the port

Multiplayer's lines, status text and pickups all had gaps. Sweep MPPaneList and every HUD_MP* update and create function: each pane's sprites, positions, colours, enable conditions, and what writes them, read from the decompile and ee_start_05.bin. Include the per-scenario objective icons, MP_setPlayerStatus's status bits and what reads them. Fix what is missing or wrong where the draw path exists, pin each with a test, and list anything needing new drawing as a table with the template values. Merge origin/master first.

---
claimed-by: sandbox3 at 2026-11-04T21:26Z

---
## Report (done, 2026-11-04T21:53Z)

Closed by the driver: the report is on `s/audit-mp-panes` (docs/reports/audit-mp-panes.md); the worker didn't run finish.sh.
