# 0700000c: the vent access cannot be crawled through

Near (5.33, 12.49, -110.94), behind the crates, Bond cannot crawl through the vent. This has come up before. Find the crouch and duck rules, the grille, and the collision at the opening, and walk it end to end with real input.

Reproduce on the real level first through real input, not a teleport; a report that repeats means earlier tests passed without reaching the real path. Compare with the decompile, fix, and test red on the old code. Branch from origin/master, push to `s/level-0c-vent-crawl`, report in docs/reports/level-0c-vent-crawl.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox1 at 2026-11-07T21:41Z

---
## Report (blocked, 2026-11-07T23:18Z)

