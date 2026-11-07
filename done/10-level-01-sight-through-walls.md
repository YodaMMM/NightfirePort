# Guards see Bond through walls

Seen on 07000001 near (11.43, 6.47, -3.51) and reported as general. Read the sight test (the line of sight call, its flags, which cels and objects block it) and compare it with the port's guard sight on real geometry: a guard with a solid wall between it and Bond must not see him.

Reproduce on the real level first through real input, not a teleport; a report that repeats means earlier tests passed without reaching the real path. Compare with the decompile, fix, and test red on the old code. Branch from origin/master, push to `s/level-01-sight-through-walls`, report in docs/reports/level-01-sight-through-walls.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox5 at 2026-11-07T21:55Z

---
## Report (done, 2026-11-07T22:50Z)

