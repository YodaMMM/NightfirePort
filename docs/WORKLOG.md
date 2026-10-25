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

## level-03-audit progress (2026-10-25 22:57 UTC)
Census done: 511 main-map placements / 31 kinds [D placements-all] vs the dispatcher [M 0x1d04f0] —
written to docs/levels/07000003-audit.md (one row per kind, creator/update/handlers/port/GAP).
Scripts: the level's NIS entries live inside 07000003.bin (9 of them; no 078/079 companion packs);
SP #475 → 0x06000080 (the keyed-camera cut-scene, fired by the ch-0x36 plates), SP #109/#110 →
0x06000563 (the Dragon Safe reveal, mode 64 preload-then-prime, mask 4 = the E-use latch).
GAP-2 fixed: 07000003.md's "no disc writer for 0x4E/0x51/0x52" was wrong for 0x4E (the script-spawned
safe key's +0x28 [D 06000563 s1; M 0x18e650:330-333]) and 0x52 (AND #435 = 0x4E AND 0x51 [D; M
0x1b2c68]). GAP-3 fixed: crates/nf-view/tests/wiring_07000003_live.rs — 4 tests, green (the AND/SEQ
chains, the locked door #4 + SS #502's ch-0x31 semantics, the mission record, the spawned key's params).
GAP-1 documented (3 rigid-body lamps — the shared 01/02 item). Suite running.
