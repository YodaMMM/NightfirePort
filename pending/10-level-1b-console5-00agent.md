# 0700001b: console 5 latches 93 ticks late on 00 Agent

On 00 Agent console 5 latches at frame 3667, after its 3574 deadline, so missile 5 launches (docs/reports/batch-1109a-walks.md). Find what makes the latch late (the walk's order, the console's hit test, a 00 Agent script timing [D the console UI scripts]) and fix the port or the route so the console phase finishes in time. Base on origin/s/batch-1109a-walks, push to s/level-1b-console5-00agent.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-1b-console5-00agent`, report in docs/reports/level-1b-console5-00agent.md. No draw changes: name what is wrong if it is a draw issue.
