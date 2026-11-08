# 0700000a: the west hall glide height (second pass)

`the_west_hall_glide_height_matches_the_game` (crates/nf-view/tests/walkreal_0700000a_live.rs ~line 1390, ignored) still fails: the game settles at 1.1285 [L 2026-10-26]. An earlier task (level-0a-hall-glide-settle) did not make it pass. Start from that report, log Bond's root height every tick over the west hall on the real path, find which floor/step/slope rule gives the port a different settle, compare with the player ground ray and step code, fix, and un-ignore the test.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-0a-hall-glide-2`, report in docs/reports/level-0a-hall-glide-2.md. No draw changes: name what is wrong if it is a draw issue.
