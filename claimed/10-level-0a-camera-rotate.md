# 0700000a: security cameras sweep too fast

The camera near (11.32, 0.65, 6.38), about frame 11028, seems to rotate too fast. Read the Sensor update's sweep rate and how the placement params feed it, including any create-time FRAME_RATE scale (30 during level build, 60 in play), and compare degrees per second with the game.

Reproduce on the real level first through real input, not a teleport; a report that repeats means earlier tests passed without reaching the real path. Compare with the decompile, fix, and test red on the old code. Branch from origin/master, push to `s/level-0a-camera-rotate`, report in docs/reports/level-0a-camera-rotate.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox7 at 2026-11-07T23:00Z
