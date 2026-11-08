# WORKLOG

## guard-standing-capsule (2026-11-08, UTC)

Branch `s/guard-standing-capsule` cut from `origin/master` (b059df948). Task: a guard playing a
fire animation has no standing body for Bond to collide with in the port, while the game keeps one
(the 0d wire-grab finding — docs/reports/level-0d-wire-jump-grab.md on s/level-0d-wire-jump-grab
has the measured case and its live checks). Read the game's drone collision shape per state
(Drone_CollisionHandler 0x140968, the body capsule/sphere setup) and give the port the same body in
every state the game has it. Pin a bump into a firing guard on a real level, real input, no
teleport. No draw changes.

Entry started — reading the 0d report and the decompile.
