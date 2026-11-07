# 07000007: an actor stands stuck in a doorway

The waiter at (76.55, 44.12, -51.91), about frame 1856, is stuck in the open library door and does not get through. Read his route, the door's state and the blocking rules. This has come up before, so find out why earlier fixes did not hold.

Reproduce on the real level first through real input, not a teleport; a report that repeats means earlier tests passed without reaching the real path. Compare with the decompile, fix, and test red on the old code. Branch from origin/master, push to `s/level-07-actor-stuck-doorway`, report in docs/reports/level-07-actor-stuck-doorway.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox4 at 2026-11-07T21:31Z
