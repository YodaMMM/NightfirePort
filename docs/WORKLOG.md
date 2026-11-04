# WORKLOG

## audit-mp-panes
Started 2026-11-04 21:28 UTC. Branch s/audit-mp-panes (cut from origin/master 8c9f02b03).
Task: sweep MPPaneList and every HUD_MP* update/create function against the decompile and
ee_start_05.bin: each pane's sprites, positions, colours, enable conditions, and what writes
them. Include the per-scenario objective icons, MP_setPlayerStatus's status bits and what
reads them. Fix what is missing or wrong where the draw path exists, pin each with a test,
list anything needing new drawing as a table with the template values.

## audit-music-events
Started 2026-11-04 21:36 UTC. Branch s/audit-music-events (cut from origin/master 8c9f02b03).
Task: sweep every Music_Event and music-control call in ACTION.ELF for on-foot play — event
number, argument, condition, port site; drones/alarm/copters/turrets/objectives/level scripts
in scope, driving out. Fix missing or wrong calls with clear port sites, pin with tests,
report the rest as a table. Merge origin/master first.
