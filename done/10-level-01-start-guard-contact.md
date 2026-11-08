# 07000001: the start guard's first rounds and the contact

At the level start the contact stands near the line between start guard #0 and Bond. On the current tree the guard's rounds can kill the contact from 0.04 u off the line. Read the guard's first-attack rules (when an idle Bond at the start spot is engaged, the spread, and whether rounds stop on an ally) and compare the hit with the game's collision for the contact's body. Fix the port if the contact is hit where the game's round would pass.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-01-start-guard-contact`, report in docs/reports/level-01-start-guard-contact.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox3 at 2026-11-08T21:19Z

---
## Report (done, 2026-11-08T21:54Z)

