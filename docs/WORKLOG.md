# WORKLOG

### level-14-mezzanine-descent-2 (start, 03:45 UTC, 2026-10-30)

Task: Countdown Part 1 (07000014) — walk the mezzanine descent into the facility on real
input, fix any port collision gap [D/M], un-ignore
`walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit` with the rest of the walk
green, run the FULL suite (fast runner) and quote its total line. Continuing
level-14-mezzanine-descent: the hand-back pad (floor 2.15) + the 1.72 landing are mapped;
leads = the east lane's holes over the machine room (the #330/#331 climb-back spot) and the
machine-room → west-wing connection (under-pad floor, lane south end, the A5_1 east wall).

### level-14-mezzanine-descent-2 (04:50 UTC, 2026-10-30)

The descent is WALKED and `walkreal_07000014_live::walk_the_infiltration_to_the_delta_exit`
is UN-IGNORED (the binary 6 passed / 0 failed). The route (all real input through the Walker):

- The pad west → (34.63, 2.80, 37.94) → south (34.57, 2.80, 37.18) → the 0.43 step down onto
  the 1.72 landing (33.09, 2.37, 35.38) — the level-14-mezzanine-descent window's walked legs,
  reused as-is.
- West off the landing's edge onto the low machine floor (measured 0.80 under every walked
  point x 31.95..32.94, z 34.28..35.36 [D rays]; the grid band 0.2..1.4 spans x 31.5..33.5,
  z 33..35.5), then down onto the facility corridor floor (0.0 [D rays]; the corridor's own nav
  lane runs nodes 57/58/257-265 at x 27.3..28.3, disc y 0.0 [D nav]) and south to the leg-2
  start — the descent landed (29.81, 0.64, 37.65), inside the leg's assert box.
- Two real-input robustness pieces the walk needed: (1) the low floor's east margin sits under
  the landing slab's overhang — a grounded wedge at (33.68, 1.15, 35.20) with a CLEAR segment to
  the target, escaped with the player's own push (steer south-west around the slab corner, hop
  when grounded; the overhang mechanism is [S] inferred, the wedge is measured); (2) the door
  wall's west jamb bars the direct south steer at x ≥ 30.3 (the segment hit 0.17 along from
  (30.33, 0.55, 35.44) [D rays]) — the corridor crossing walks at x ≈ 29.7.
- Door #834 (plc 834, sim kind 2, ch_lock 0 [D params]) opens with E (state 0x4, pos01 1.00,
  probed) but the walked route passes WEST of its wall run (leaf x 32.00..32.65 + the fusebox
  column #833 x 31.93..32.65 [D]) — the leaf plane is never crossed. Its role is the report's
  live check.
- The EAST lane / machine room is NOT the descent: the machine-room patrol web (path 1 'Patrol01
  - Machine Room', path 0's loop n44-47-48-49-50 + n140→n52→n45→n42→n43→n41→n40) links only
  east (n2 (49.1, 0.4, 42.8)) and south (n7 (37.4, 0.4, 53.0)) [D nav] — no west exit, matching
  the prior window's west bound at x 37.1.

No crates/ code changed — the route existed in the port's collision; the prior window's
"wall at x 31.5 / no walkable strip" reading was the pad's south-west steering trap, not the
route (the descent exits north-west from the LANDING). Suite running (fast runner) on the final
tree (master 774b1cbf + this branch); total line to be quoted in the report.
