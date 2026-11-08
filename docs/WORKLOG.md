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

Done (23:35 UTC): the game's body read (0x140e90/0x149ec8/0x14b190/0x1ec180/0x140968) — per-bot,
not per-state; the port's door-guard-only grounded push was the gap. Fix in scene.rs (every near
bot: the standing capsule with the animated hip; ground_bots = near.len()) + the player.rs gate
comment. Pin guard_capsule_live: red 0.189 (through a firing 0x74/0x68 captor), green 0.942 (the
0.55+0.4 spine gap). Ran 07 (5/0), 01 (6/1 — the contact ate guard #15's burst under the shifted
lines; the sister task owns the contact line; the sweep's flank now holds the pocket rule), 0a
(12/1 — west_doorway_repro's ring tripped a mission fail at node 34; next session). Report written.
