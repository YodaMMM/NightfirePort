
## level-0d-last-two-standins (2026-10-31, s/level-0d-last-two-standins)

Branch from origin/master (533ccf2b, batch-1031f). Task: the crane-to-yard chain fully on real
input — delete the two stand-ins left in walkreal_0700000d_live::walk_arrival_to_the_c5_door by
level-0d-jib-walk-off: (1) the catwalk -> bay exit, (2) the beam boarding + the west ride on
SP #115's shuttle (137.54 <-> 177.22 over 3000 ticks [D]) and the drop into the yard at
x 137.54. Find the game's way for each, walk both on real input, fix port gaps with evidence.

## level-0d-last-two-standins progress (2026-10-31)

STAND-IN 1 DELETED: the catwalk's exit is the west-end LADDER [D type 0x22, centre (155.60, 14.25,
−266.18), r 4.39 — the only ladder east of x 120] → the ROOF 18.58 → the east band z −268.5 →
the x 196.3 lane (a strict L around the shed's NE corner) → the push-off cycles over a 19.44
parapet → the annex 9.42-9.45 → the bay 9.53 (continuous at z −276..−282) → the E #460 walk
unchanged — all real step_bond input. The grab needs yaw −2.5 (the facing gate |diff| > π/2 [M
0x196118]; west = exactly π/2 = rejected, measured). probe0d_route_scan + probe0d_ladder_try
added. Stand-in 2 narrowed: the boarding surfaces (the mezzanine 16.8 x 169..183; the 17.2 yard
tower x 134..136 over the beam's parked span) are unreachable on the port's statics (no ladder,
no crates, the slot jump walled) — the live check. walkreal_0700000d 1/0, interactables_0700000d
11/0. Report: docs/reports/level-0d-last-two-standins.md.
