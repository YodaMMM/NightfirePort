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

## Progress (2026-10-31, ~19:20 UTC)

- Census re-run over `nf-extract triggers 07000005.bin` (47 trigger rows) + the §7 kind table: the
  player-touchable kinds not covered by a live drive on 05 were (a) the plunge walk-off death chain
  (the shells fired, but no test pinned ch 190 → SP #284 → ch 0x62 end-to-end), (b) the wall-hug
  window ledge run (the lock-gated TouchOnces #311-#314 + AND #80 + the 0x1B1900 alarm→0x6F engine
  latch — the audit's open item), and (c) the blocked ch-0x7D Touch #70 (modeled None; nothing
  pinned the nothing).
- 3 new live tests in `interactables_07000005_live.rs` (now 6): the valley walk-off runs the death
  NIS chain; the quiet window-ledge run fires all four plates, completes AND #80 → ch 105 → sound
  332, and the post-alarm #314 is refused (the one-shot); the blocked cut-scene touch writes
  nothing while the toucher pass demonstrably runs. All cite [M]/[D]/[L] per line.
- The census's "ledge floor sits low" open item re-measured: the floors are the disc's own 36.5-37.3
  (fixed by the west-lane merges); the death-NIS catch is the WEST slide off the icon ledge's edge
  (y 35.4 < shell #291's top 35.47) — documented in the level doc, avoided by the east-side crossings.
- No port code changed: the three chains already behaved like the game; the tests pin them.
- Run: interactables_07000005_live 6/6, walkreal_07000005_live 11/11, nf-game 953/0. Full suite
  (`tools/test/suite.py`) on the final tree: running at report time — the total goes in the report.
