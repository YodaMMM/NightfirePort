
## level-09-interactables (branch s/level-09-interactables)

- start (master 5ae5d60d). Task: 07000009 has only interact_07000009_live.rs (read what it covers). Using interactables_0700000b_live.rs as the model (one live test per KIND of player-touchable placement, driven the way a player drives it, each asserting the game's effect with a sister), census every player-touchable placement on 07000009 [D placements + docs/levels/07000009.md], add the missing kinds in crates/nf-view/tests/interactables_07000009_live.rs, fix every port gap the tests expose [M the class code; D the params]. Teleport only to reach a spot whose walk is a documented blocker. Every test green. Run the touched binaries and quote their totals.
