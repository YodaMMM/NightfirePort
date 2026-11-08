# 07000012: the wing guard's rounds and office civilian #24

On the east wing, guard placement 9 fires at Bond's back and a round kills civilian #24, failing the mission with Killed Civilians. Check the game's rules for this: does the guard fire with a civilian in his line (the fire gate's line-of-fire test), and does the round stop on the civilian. Fix the port if it fires or hits where the game would not.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-12-wing-guard-civilian`, report in docs/reports/level-12-wing-guard-civilian.md. No draw changes: name what is wrong if it is a draw issue.
