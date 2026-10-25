# WORKLOG

One section per task, appended at the end. Never rewrite or drop another task's entry.

### level-04-audit (started 22:50 UTC, 2026-10-25)

Branch from origin/master (b312d959). Task: the 07000004 behaviour audit — every placement on
the main map [D] (characters 0x0f, doors, triggers, scripts/SPs, switches, monitors/sensors,
pickups, destroyables, breakables, ladders, grapple points, lifts, special objects), every
script stream's ops/camera streams/hand-backs, and the mission records, each row: the game's
creator/update/handlers [M], what the port does, a GAP column → docs/levels/07000004-audit.md.
Small gaps fixed directly [M] with a test each; large gaps become report sections for their own
tasks.

Plan: census the level's placements/scripts/records from the disc data; read the port's
07000004 paths (load/scene/mission + the walk test); build the audit table row by row; fix the
small gaps; suite; report.
