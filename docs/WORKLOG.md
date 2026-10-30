# WORKLOG

## level-0c-interactables (branch s/level-0c-interactables)

- start (master b3c4e4a1). Task: one live test per KIND of player-touchable placement on
  0700000c, driven the way a player drives it, each asserting the game's effect with a sister
  (model `interactables_0700000b_live.rs`); read what the existing interact_/walk tests for
  0700000c already cover; census every player-touchable placement [D placements +
  docs/levels/0700000c.md]; add the missing kinds as `crates/nf-view/tests/interactables_0700000c_live.rs`;
  fix every port gap the tests expose [M class code; D params]. Teleport only to reach a spot
  whose walk is a documented blocker; the operation itself always real input.
