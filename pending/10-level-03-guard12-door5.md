# 07000003: guard #12 and door #5

With the goal-node search limited to the goal's room cel (AINetwork_NodesForPosition 0x1586a8), guard #12 on 07000003 no longer walks through door #5 from the corridor side. Trace his patrol/route on the real path and compare with the game's route data and states: does he ever use that door in the game? Fix the port if his route differs from the game's.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-03-guard12-door5`, report in docs/reports/level-03-guard12-door5.md. No draw changes: name what is wrong if it is a draw issue.
