
## level-12-interactables-2 — 2026-10-31

Task: every player-touchable kind of 07000012 behaves like the game. Starting from the 9-test
`interactables_07000012_live.rs`; the census [D every placement kind] decides what is missing,
`interactables_0700000b_live.rs` is the model (one live test per kind, walk/E/shoot through the
real path, sister per test).

## level-12-interactables-2 — 2026-10-31 (the two added kinds + the totals)

Added to `interactables_07000012_live`: the TouchOnce plate test (walk into #30 → ch 0x5
latches, holds after the toucher leaves, fresh-load sister) and the laser TOUCH beams test
(all 20 rows out = param 0 = 0 [D] → crossing pins the whole channel board unchanged; #683 the
kind's real-out sister). The TouchOnce OUT slot is the placement's param 0 [M 0x1b2b88; the
port's from_params row out = q(0) — the earlier "out = p1" reading in this entry's start note
was wrong and is corrected here]. The grapple point is not player-reachable on 12 (no claw in
the loadout/pickups [D]); the sound triggers are channel emitters, not walk-ins [M 0x1b2790].

Totals: interactables_07000012_live 10 passed / 1 ignored (the documented doors blocker) /
0 failed; walkreal_07000012_live 1/0/0; nf-game 953 passed / 0 failed / 98 binaries. Full suite
chunks: formats+audio 136 passed / 3 failed (the allowed slim-ISO disc_golden trio), nf-game
953/0, nf-view 736 passed / 0 failed / 172 binaries before the window closed inside
front_end_chain_01_to_05 (a master test untouched by this diff) — the tail is next window's.
