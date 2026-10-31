
## level-0c-secret-door-reach-3 — started (in progress)

- 13:00 UTC branch s/level-0c-secret-door-reach-3 created from origin/master 533ccf2b and
  origin/s/level-0c-secret-door-reach-2 merged (sandbox warm; disk pruned to 584M free).
  NEXT: read blocked/10-level-0c-secret-door-reach-2.md + the three [L 2026-10-31] live checks
  (the apron slab stops x 56.3-56.75 at z -152..-166, opens at z -146/-141; the barrel row stops
  the corridor's north end at z -150.23 in EVERY lane, running jumps fail; the ST-1 walkway
  railing stops z -153.65), find the barrels' class (destroyable/rigid-body/0x3B?) and the real
  route to plates #558/#559 (a mezzanine drop at 5.44? the annex vent #180/#182/#184 ch 0x54? a
  door?), make the port's barrels block like the game's (a pin at z -150.2 from the north), walk
  the real route on real input, open the door through the plates, delete the [plate2 + 0.6]
  placement in walkreal_0700000c_live.

- 13:10 the corridor placement dump (dump_0700000c_corridor_rows.rs): NO barrel placements exist
  near (53-56, -150) — the 0700000c barrels are 0xE1 Destroy rows at (61.9-63.2, 0.38, -183) by
  the secret door and in the warehouse; the [L] "barrel row" at z -150.23 is the level's STATIC
  collision. The plates #558/#559 (out ch 3/ch 2), the mux #560 (out 8, inputs 2, 3), the door
  #544 (ch 8) and the gate #556 Load_Level (target 0x0700000d, key ch 0x1c) dumped [D].
- 13:25 the lane probe: the PORT already stops the south walk at z -150.24/-150.23/-150.74 in the
  [L] lanes x 53.0/54.3/55.4 — the static collision matches the game; a strict-south walk at x
  54.3 slides EAST past the row's east end to (61.35, -170) [S the game's east end unmeasured].
- 13:30 the lip probe: the walkway's south seal reproduced (the slow walk hangs, the run-through
  hangs at -153.65 [L-matching], the pre-edge jumps land ON the railing top at 16.70 and stick) —
  the deck chain is dead in the port too.
- 13:40 THE ROUTE: the annex vent. The vent SS #180 + the blocking poly #184 hang in the
  corridor's ceiling at (53.8, 5.5, -163.59) with the Q-Laser latch #182 at (53.80, 5.90, -165.36)
  [D p5 = 0x54], above the mezzanine deck (5.44..6.03, x 52..56, z -176..-152 [D reach-1]). The
  ventroute probe: the deck stand -> the 0.56 step-down (a grounded jump crosses; the walk holds,
  the game's own lip-hold) -> the held-Q-Laser burn of #182 (+144 ticks) -> ch 0x54 -> the SS
  sweep -> the drop into the corridor at (54.23, 2.02, -163.54) PAST the barrel row -> the
  corridor walk -> the plates (ch 2) -> the E-arm -> ch 8 -> the door sweeps. ALL REAL except the
  mezzanine REACH (the port's statics flood-isolate the deck [D reach-1 comp 3]; the grapple
  points gp0/gp1 at y 12.3 hang over the warehouse — the live check).
- 13:50 walkreal_0700000c_live leg 4 rewritten: the [plate2 + 0.6] placement DELETED, replaced by
  the vent route (one documented stand-in: the mezzanine reach). The final gate approach
  tightened to the #556 centre (96.92, -209.71 [D]) — the old r(1.0) stopped 1.9 short and the E
  missed the quad. walkreal 2/0, interactables 5/0, the barrel-row pin 1/0.
