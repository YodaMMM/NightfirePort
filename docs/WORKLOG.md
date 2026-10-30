# WORKLOG

## level-0d-jib-rim-gap (branch s/level-0d-jib-rim-gap)

- start (master 76081986). Task: level-0d-crane-and-to-the-end (merged) made the dock → crane
  deck crossing real input (ladder #435, the wire, the rotate) and mapped the deck → yard
  remainder, blocked only on the jib rim gap. Find how the game gets Bond from the deck to the
  yard [D the crane/jib placements, keys and collision; M the classes the route uses — wire,
  rotate, any script player], fix any port gap, and replace the remaining teleport in
  `walkreal_0700000d_live::walk_arrival_to_the_c5_door` with real input. Every test in the file
  green. Quote the totals.
- outcome (2026-10-31): the deck→yard route mapped end-to-end with committed probes
  (crane17/18/19). Settled: the swing is the cut-scene's beat (the walkway's 12.13 collision is
  a STATIC strip at the jib's rest spot — probe19's end-pose rays find nothing at r 6.5-8 after
  the swing while the deck's collision rotates [probe18]); the game's jump is VERTICAL-ONLY
  (Player_HandleJump 0x1916b0 Vec_Zero + a fixed WldGravity impulse — the caller 0x192310:69
  passes WldGravity @0x2d8890 [D symtab]; the port's arm is faithful — probe17's five stall-jumps
  all land in place). The mid-swing walk-off reaches r 7.4 but misses the 1.5 m strip by ~0.13 m
  (the tangential carry drifts the fall +0.25 [probe17 v3]). The teleport stands; the next
  window's list (the walk-off tuning candidates, the dock E #460, the beam boarding via the
  north steps → roof → catwalk → the westward pass) is in the report. Touched binaries:
  walkreal_0700000d 1/1, interactables_0700000d 11/11. No nf-game source changed.
