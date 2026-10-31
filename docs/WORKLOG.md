
### level-0d-beam-boarding (started 15:05 UTC, 2026-10-31)

Task: settle the last stand-in in `walkreal_0700000d_live::walk_arrival_to_the_c5_door` —
the beam boarding + the west ride on SP #115's shuttle (137.54 <-> 177.22 over 3000 ticks [D])
and the drop into the yard. Settle the access from the data where possible [D the beam's
script-entity collision at each shuttle tick, the roof/catwalk edges it passes; M
Control_InheritVelocity 0x133e88 / Player_FeetOnPoint 0x194188 for the carry], walk the boarding
and the ride on real input, delete the stand-in; if only a live run can settle it, pin the exact
question with a sister assert. Branch from origin/master (21bfe192, batch-1031h).

Plan: WORKLOG+branch pushed first; start the fast runner; read the stand-in + the port's beam
handling; dig the disc script 0x060009EF entity writes for posed/moved collision; then the real
boarding walk and the ride.
