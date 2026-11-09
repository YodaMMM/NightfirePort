# 07000006: dropping into the well

With the current hurt-volume rule (touching a hurt volume always deals its damage; the who mask only gates its start [M 0x194458:94-101, 0x17e5d0:51-102]), dropping into the 06 well kills Bond through hurt volume #373 (1000 damage [D]). Check that this is the game: the volume's placement, its start state, where a falling Bond meets it, and whether the level expects the well to be a death drop or a route (objectives, scripts, any ladder or exit below). Fix the port if a player in the game survives or uses the well.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-06-well-drop`, report in docs/reports/level-06-well-drop.md. No draw changes: name what is wrong if it is a draw issue.
