# WORKLOG

## level-03-audit (2026-10-25, sandbox expiring ~00:28 UTC)
Branch `s/level-03-audit` off origin/master (b312d959). Task: the 07000003 behaviour audit — every
placed object and character on the main map [D] (characters 0x0f, doors, triggers, scripts/SPs,
switches, monitors/sensors, pickups, destroyables, breakables, ladders, grapple points, lifts,
special objects) checked against the game: creator in parsemap_create_dynamic_objects 0x1d04f0 /
static path, update, use/damage/event handlers [M], and what the port does; plus every script
stream's ops, camera streams, hand-backs, and the mission records (objectives, channels, fail
conditions). Output: docs/levels/07000003-audit.md (one row per placement/script/record, game [M/D],
port, GAP). Small gaps fixed directly with tests; large gaps documented as future tasks.
Plan: branch pushed at start; work-slim + ISO downloads; census placements first, then scripts/mission
record; fix small gaps; suite via tools/test/suite.py.
