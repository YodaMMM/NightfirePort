# WORKLOG

## audit-sp-panes
Started 2026-11-04 23:24 UTC. Branch s/audit-sp-panes (cut from origin/master f503c15c7).
Task: the single-player HUD panes against the port. PaneList @0x2be5b8: every pane's create
and update function, sprites, positions, colours, enable conditions and writers, read from the
decompile and ee_start_05.bin. Check each pane's hide rules against the zoom, vehicle and
remote-view panes, and every state of each update's switch. Fix what is missing or wrong where
the draw path exists, pin each with a test, and list anything needing new drawing as a table
with the template values. Driving panes are out of scope. Merge origin/master first.
Outcome (2026-11-05 00:45 UTC): the sweep re-walked all 22 PaneList slots and every SP update
switch against the port; the port's panes matched the decompile everywhere except two deltas,
both fixed and pinned — the lens-flare pane (pane 8) was never drawn (the alpha ramp, the
zoom-rescaled sun gate, the 7 template quads; HudTables.flare + HudState::flare_alpha +
flare_quads; 2 new test files, 5 tests) and the health pip-0 blink skipped the pane fade.
SpaceMissile_Update 0x1aa118 is now decompiled: its jump table gives the MissileDeploy console
states as [M] (the port keeps deploy=0, reported). Four new-drawing/modelling rows and four
live checks in docs/reports/audit-sp-panes.md. The changed-code binaries all pass (33/0); the
full suite did not fit this sandbox's disk (ld SIGBUS relinking the nf-view binaries).
