# agent-rev-panes

Comment-only repair on top of s/pane-ride-on-2. No behaviour change: PANE_RIDE stays off.

The pane branches described the ray skip as the game's rule, citing per-cel ray walks that a
cross-room pane never joins. `Collide_RayIntersect` 0x1ec690 runs `Collide_Pick` 0x1eb320, the
same `Collide_StraddleCels` candidate-room walk the movement queries use, so that citation does
not hold. The comments now mark the skip [S], a port stand-in, and say the ride is off where they
read as if it were on.

Tests: pane_pairing_census 2/0, dead_pane_live 0/0 2 ignored on this tree. The code tree under
the comments ran walkreal_07000001_live 8/0, walkreal_07000008_live 6/0,
glass_pass_every_level_live 3/0 3 ignored.
