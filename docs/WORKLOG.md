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
level-11-audit completion (2026-10-26 ~08:55 UTC): census 319 placements/0 unresolved + 7 scripts +
mission record #15 [L]; audit table docs/levels/07000011-audit.md (one row per placement, GAP
column); port matches the game on every sim-side law this level exercises (symptom list resolved in
docs/reports/level-11-audit.md); 3 pin-tests added (door lock-edge full law, 0xf9 difficulty gate,
rotor tick) — 3/3 pass; one draw-only gap (rotor pose, mesh/matrix + render spots named); 6 doc
errata; suite chunked in the sandbox's foreground (background jobs die here): 820 binaries cached,
60-72 run per chunk, 0 failures except the known no-GPU probe_pylon_render, no total line.
