# Guards in their fire animation: the standing body Bond collides with

The 0d wire-grab work found that a guard playing a fire animation has no standing body for Bond to collide with in the port, while the game keeps one (see docs/reports/level-0d-wire-jump-grab.md on s/level-0d-wire-jump-grab for the measured case and the live checks it lists). Read the game's drone collision shape per state (Drone_CollisionHandler 0x140968, the body capsule/sphere setup) and give the port the same body in every state the game has it. Pin a bump into a firing guard on a real level.

Reproduce on the real level first through real input, not a teleport. Compare with the decompile, fix, and test red on the old code. Run the walkreal_* binary of every level the change touches, one level per tool call. Branch from origin/master, push to `s/guard-standing-capsule`, report in docs/reports/guard-standing-capsule.md. No draw changes: name what is wrong if it is a draw issue.

---
claimed-by: sandbox5 at 2026-11-08T22:12Z
