# WORKLOG

## level-0a-corner-push-model (branch s/level-0a-corner-push-model)

- start (master bdb8c6e0). Task: with the push model kept (the [L 2026-10-30] rest z 15.5904 = master's push model), find what differs at the 0a hall corner where `walkreal_0700000a_live::the_corner_crosses_like_the_game` needs 870 ticks (NF_WALK_KILL) vs the game's ~60 frames [L 2026-10-26] — [D the corner's faces; M Intersect_CylGeom 0x109da0, Collide_Intersect 0x1ebaa8, the player push/slide] — fix it, un-ignore the test without the kill switch, every test in the file green, run every walkreal_* binary, quote totals.
