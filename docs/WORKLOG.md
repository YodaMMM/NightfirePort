## level-05-interactables-2 (2026-10-31, s/level-05-interactables-2)

Branch from origin/master (e4550f2e, batch-1031k). Task: every player-touchable kind behaves
like the game — `interactables_07000005_live.rs` has 3 tests; using
`interactables_0700000b_live.rs` as the model (one live test per KIND of player-touchable
placement — doors, switches/consoles, fuseboxes, lifts, pickups, breakables, plates, monitors,
rigid bodies, wires/zip lines, grapple points, SimpleScripts the player triggers), run the
census of 07000005 [D every placement kind], and for every kind a player can touch that is not
yet covered, add the live test that drives it through the real path and asserts the game's
result [M the class; D its params], with a sister that fails on the wrong result. Fix any port
gap the tests expose, with evidence. Run interactables_07000005_live, walkreal_07000005_live
and nf-game; quote the totals.
