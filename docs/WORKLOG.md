# WORKLOG

## level-04-interactables-2 (branch s/level-04-interactables-2)

- start (master ec9dfc3f, batch-1031j). Task: `interactables_07000004_live.rs` covers only the
  lamps (2 tests); using `interactables_0700000b_live.rs` as the model (one live test per KIND
  of player-touchable placement), run the census of 07000004 [D every placement kind], and for
  every kind a player can touch add the live test that drives it through the real path
  (walk/E/shoot) and asserts the game's result [M the class's Update/Activate; D its params],
  with a sister that fails on the wrong result; fix any port gap the tests expose, with
  evidence. Run interactables_07000004_live, walkreal_07000004_live and nf-game; quote totals.
