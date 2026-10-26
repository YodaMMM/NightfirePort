# WORKLOG

## level-11-audit — the 07000011 behaviour audit (2026-10-26 ~07:20 UTC, sandbox expiring ~09:17 UTC)
Branch start: s/level-11-audit = origin/master (b604242f). Task: walk test for 07000011 passes but
real play finds missing behaviour (characters walk through Bond, guests never talk, doors never
lock, consoles whose action does nothing, cut-scenes with broken cameras). Audit EVERY placement on
07000011's main map [D] (characters 0x0f, doors, triggers, scripts/SPs, switches, monitors/sensors,
pickups, destroyables, breakables, ladders, grapple points, lifts, special objects) against the
game: creator in parsemap_create_dynamic_objects 0x1d04f0 / static path, update + use/damage/event
handlers [M], and the port's handling; same for the level's scripts (every stream's ops, camera
streams, hand-backs) and mission records (objectives, channels, fail conditions). Table goes to
docs/levels/07000011-audit.md with a GAP column; small gaps fixed [M] with tests, large gaps
documented as their own future tasks. Tests pin only proven behaviour ([M]/[D]/[L] evidence).
Plan: census placements/scripts/records from the disc first, then decompile-side handler chains per
kind present, then port-side coverage, then fixes. Suite runner starts once the ISO + toolchain land.
