## level-13-interactables-2 (2026-11-01, s/level-13-interactables-2)

Branch from origin/master (e79794da). Task: level-13 — every player-touchable KIND on 07000013
behaves like the game. interactables_07000013_live.rs has 6 tests; using interactables_0700000b_live.rs
as the model, run the census of 07000013 [D every placement kind], and for every kind a player can
touch add the live test that drives it through the real path (walk/E/shoot) and asserts the game's
result [M the class's Update/Activate; D its params], with a sister that fails on the wrong result.
Fix any port gap the tests expose, with evidence. Run interactables_07000013_live,
walkreal_07000013_live and nf-game; quote the totals. Full suite with the fast runner on the
final tree (merged with origin/master).
