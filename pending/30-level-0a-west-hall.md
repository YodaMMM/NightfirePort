# 0a: the west hall doorway

`walkreal_0700000a_live` ignores two tests where the port differs from measured game behaviour: `the_west_hall_glide_height_matches_the_game` (settled y 1.0905 on master, game 1.1285) and `the_corner_crosses_like_the_game`. Read both ignore reasons first: the move-clip idea is dead, and the old "crosses at +76 ticks" number came from a dead flag path.

Start from `Player_Collision` 0x194df0 and `Player_CollisionHandler` 0x1949a8 and find what lifts Bond in the doorway. Every doorway contact pushes horizontally in the port, so look for what the game does that the port lacks. Fix it and remove the ignores, or report the measured next lead. Quote walkreal 0a and two other levels' walks.
