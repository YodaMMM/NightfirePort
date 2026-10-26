
## level-07-audit (2026-10-26, sandbox expiring ~02:22 UTC)
Branch `s/level-07-audit` = origin/master (b312d959). Task: audit EVERY placement on 07000007's
main map [D] (characters, doors, triggers, scripts/SPs, switches, monitors/sensors, pickups,
destroyables, breakables, ladders, grapple points, lifts, special objects) against
parsemap_create_dynamic_objects 0x1d04f0 / the static path [M]: creator, update, use/damage/event
handlers, port coverage (created/updated/drawn/usable/reacts/sounds). Same for the level's scripts
(every stream's ops, camera streams, hand-backs) and the mission records (objectives, channels,
fail conditions). Table to docs/levels/07000007-audit.md with a GAP column; fix small gaps [M]
with tests; large gaps become report sections for their own tasks.
