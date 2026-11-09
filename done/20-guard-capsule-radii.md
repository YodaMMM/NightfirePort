# Guard body capsule sizes per state

The game's guard collision body uses different capsule radii by state (0.55 and 0.3 variants) and a per-guard switch for whether a guard collides at all. Read the drone collision setup (NDrone2_Collision 0x140e90, 0x149ec8, the capsule radius choice) and port the per-state radius and the switch so door contacts and bumps use the game's size.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/guard-capsule-radii`, report in docs/reports/guard-capsule-radii.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox3 at 2026-11-09T17:44Z

---
## Report (done, 2026-11-09T20:53Z)

