# 07000008: Bond's guided rocket turns into the cabin wall in the Hind fight

In the Hind fight the 00 Agent walk dies to its own guided rocket: fired with the Hind behind the cabin wall, the rocket steers into the wall 0.8 u from Bond. Read the guided rocket's steering (turn rate, target lead, line-of-sight to the target) and its arming distance in the decompile, and whether the game's rocket would hit the wall there. Fix the port if the steering or arming differs; otherwise re-route the walk. Un-ignore the 00 Agent walk if it passes.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/level-08-rocket-self-blast`, report in docs/reports/level-08-rocket-self-blast.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox3 at 2026-11-08T19:57Z

---
## Report (done, 2026-11-08T20:43Z)

