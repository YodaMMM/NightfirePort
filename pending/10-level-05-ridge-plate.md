# 07000005: can the ledge plate be reached with the ridge blocker solid

On 07000005 an invisible blocker on the roof ridge is now solid and keeps Bond off the old spot by plate #253 (interact_07000005 test moved). Check on the real path whether a player can still reach and trigger #253 (its sound trigger) from any approach the game allows, using the blocker's real collision and the game's rules for that object kind. If it is unreachable in the port but reachable in the game, fix the port.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-05-ridge-plate`, report in docs/reports/level-05-ridge-plate.md. No draw changes: name what is wrong if it is a draw issue.
