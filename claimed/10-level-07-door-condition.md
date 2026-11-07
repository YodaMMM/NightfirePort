# 07000007: a door opens before its condition is met

Near (94.55, 44.13, -50.47), facing the library door with two suited guards, the door can be opened at once. Read the door's lock channel and what the level's script expects before it may open, and find whether the port misses a lock or its writer.

Reproduce on the real level first through real input, not a teleport; a report that repeats means earlier tests passed without reaching the real path. Compare with the decompile, fix, and test red on the old code. Branch from origin/master, push to `s/level-07-door-condition`, report in docs/reports/level-07-door-condition.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox4 at 2026-11-07T22:38Z
