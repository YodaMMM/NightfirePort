
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

**level-0d-beam-boarding progress (15:05–16:30 UTC):** the stand-in narrowed to ONE documented
write — the boarding placement. The mover is a twin-girder gantry [D shape 0x0200_0a80: the deck
probe's column pairs — west girder top ~13.42-13.47 x 135.1..136.5 at rest, the slot x
136.5..138.6 open to the yard 4.0, east girder x 138.6..140.1; the riding 0x0200_0a81 is a 2-tri
0xEA trigger plane at y 14.28]; the origin sweeps 137.54 → ~177.22 (t≈1230) → ~137.59 (t≈2760)
[D the beam-watch trace]. The east-side boarding candidates (mezzanine 16.8 / ledge 13.76 / pad
14.7) are closed: the gantry's east face never passes x 181.4, 2.6 m short of the ledge, and the
jump is vertical-only [M 0x1916b0]; no stairs/ladder from the bay [D censuses]. The RIDE is real
input (the carry [M 0x194188 → 0x133e88] asserted: the offset held within 0.6 through the apex)
and the DROP is real input (the girder edges wall the walk — measured zero displacement; the
game's way off is the jump + the air drift into the slot, 9.5 m onto the yard — jump is a plain
jump, the icon context re-arms from proximity [M 0x1949a8:22]). walkreal_0700000d_live
**1 passed / 0 failed (51.8 s)**; interactables_0700000d_live **11 passed / 0 failed (42.2 s)**.
The suite runner's OOM-crashed binaries (mp_stress_explosives_live, worldobj_live) need a re-run
on a quieter box; the disc_golden trio is the expected slim-ISO set.
