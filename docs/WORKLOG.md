## level-0c-interactables-2 (2026-10-31, s/level-0c-interactables-2)

Branch from origin/master (81da6c47, batch-1031l). Task: 0c — every player-touchable kind
behaves like the game. `interactables_0700000c_live.rs` has 5 tests; using
`interactables_0700000b_live.rs` as the model (one live test per KIND of player-touchable
placement), run the census of 0700000c [D every placement kind], and for every kind a player
can touch, add the live test that drives it through the real path (walk/E/shoot) and asserts
the game's result [M the class's Update/Activate; D its params], with a sister that fails on
the wrong result. Fix any port gap the tests expose, with evidence. Run
interactables_0700000c_live, walkreal_0700000c_live and nf-game; quote the totals.

## Progress (2026-10-31, ~21:10 UTC)

- Branch created from origin/master (81da6c47). Existing 0c coverage on master noted:
  interactables_0700000c_live.rs (5 tests), interact_0700000c_live.rs, walk_0700000c_live.rs,
  walkreal_0700000c_live.rs (+ barrelrow/eastroute), census_0700000c.rs,
  docs/levels/0700000c.md + -audit.md. Census starts next.
