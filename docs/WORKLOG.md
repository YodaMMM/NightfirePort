Branch cut from origin/master 8f27b95e (2026-11-02 01:36 UTC). WORKLOG started fresh (master tracks no docs/WORKLOG.md; recreated per prior-task convention).

## Task
level-1b-interactables-2: 0700001b every player-touchable kind behaves like the game. `interactables_0700001b_live.rs` has 7 tests. Using `interactables_0700000b_live.rs` as the model (one live test per KIND of player-touchable placement), run the census of 0700001b [D every placement kind], and for every kind a player can touch, add the live test that drives it through the real path (walk/E/shoot) and asserts the game's result [M the class's Update/Activate; D its params], with a sister that fails on the wrong result. Fix any port gap the tests expose, with evidence. Run interactables_0700001b_live, walkreal_0700001b_live and nf-game; quote the totals.

## Progress (level-1b-interactables-2)
- (in progress)
