# 07000003: something pushes Bond out of the Load Point plate

On the joined tree, Bond standing on the Load Point plate is pushed 0.42 u off it (master left him inside). Find what pushes him (a door leaf, a pane, a blocker volume, an actor) by logging contacts per tick on the real path, compare with the game's collision for that object, and fix the port if it differs. The level exit must still trigger from the plate.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-03-loadpoint-push`, report in docs/reports/level-03-loadpoint-push.md. No draw changes: name what is wrong if it is a draw issue.
