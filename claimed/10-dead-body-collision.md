# Do dead guards block the player?

The game gives a dead guard a collision body (the capsule-join work found the code path) while the port gives corpses none. Read the dead/ragdoll states' collision setup and whether the player's capsule push meets a corpse in the game (NDrone2_Collision 0x140e90, the dead state's flags), pin it through real input (walk Bond into a fresh corpse), and port it if the game blocks.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/dead-body-collision`, report in docs/reports/dead-body-collision.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox3 at 2026-11-09T13:24Z
