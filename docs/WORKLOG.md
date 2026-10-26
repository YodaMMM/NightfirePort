# WORKLOG

## level-12-audit (2026-10-26, sandbox expiring ~09:16 UTC)
Branch `s/level-12-audit` off origin/master (b604242f). Task: the 07000012 behaviour audit —
every placement on the main map [D] (characters 0x0f, doors, triggers, scripts/SPs, switches,
monitors/sensors, pickups, destroyables, breakables, ladders, grapple points, lifts, special
objects), every script stream (ops, camera streams, hand-backs) and every mission record
(objectives, channels, fail conditions): the game's creator/update/handlers [M], the port's, and
a GAP column, written to docs/levels/07000012-audit.md. Small gaps fixed directly [M] with a test
each; large gaps become report sections (what, where, the [M] chain). The known symptoms: the
walk test passes but real play finds characters walking through Bond, guests that never talk,
doors that never lock, consoles whose action does nothing, cut-scenes with broken cameras.
Plan: reuse the sandbox (work/ + ISO + nf-queue + rust + alsa all present); branch pushed at
start; suite in the background early; the level doc docs/levels/07000012.md as the base; census
via a disc dump; the decomp per kind from work/decomp/ACTION.ELF.
(Note: the previous session's task fusebox-inverter-2 was closed first — its finish.sh + ntfy
had been lost with the sandbox; the queue shows it done with the report attached.)
