# WORKLOG

### level-0c-vent-pulley-class (start, 06:22 UTC, 2026-10-30)

Task: port the 0xE2 vent/pulley class from the decompile (its create/update fns [M]; the 0c
placements [D]), then replace the display-eye + plates-reach stand-ins in
`walkreal_0700000c_live::walk_the_whole_mission_from_the_start_to_the_0d_handoff` with the
real-input leg. The walk must stay green. Run the touched binaries and quote their totals.
Branch from origin/master 0c90eba8.

### level-0c-vent-pulley-class (the 0xE2 census + the staircase leg, 08:25 UTC, 2026-10-30)

Merged origin/master (0c90eba8 → 535b7d15). Findings:
- The 0xE2 SS class was ALREADY ported (ss.rs = SS_Create 0x1a9c58 / SS_Update 0x1a9948 /
  SS_Operate 0x1a9df8 / SS_Activate 0x1a9ea8); nothing to port.
- The 0c 0xE2 census [D dump]: #138/#179 (the ST-1 vent, already real), the pulley winches
  #351/#364/#377 (four SAME-position keys, ±0.026-rad sway, mask 0x2 — dead props, no travel
  lift anywhere), and the ANNEX VENT #180 "ventmesh" (53.80, 5.50, −163.59, ch 0x54, 6 keys
  rate 50) + VentLatch #182 (kind 0x2e) + blocker #184 (sinks y 5.43 → 0) — the game's own
  display-ground→warehouse connection, warehouse-side.
- The display eye stand-in is REPLACED by the real staircase descent: the 11.27 hall's own
  floor opening (x 9..12.4, z −115.5..−131.5 [D probe rays]) over obj_stor2_staircase01 #359
  (kind 0x19, bounds x 9.97..12.29, y 6.13..12.14, z −127.5..−117.1 [D]) — landing 11.29 at
  z −128, ramp down to 6.29 at z −120, the south ledge (6.27) north to the display pocket,
  the REAL camera photograph, objective 2 completes. Sister pins on the shaft/landing/ramp.
- The plates reach stays the master stand-in, now with closure evidence: the slab's south
  face z −155.77..−156.26 from y ~2.5 up leaves a 0.23..0.4 slot above the 2.37 deck (no
  capsule passes [D probe rays]; raw south walk stops at z −155.04 standing AND crouched);
  the 2.18 deep lane (x 8.5..12.2) ends at z ~−150 into the same closed pocket; the annex
  floor (~5.44..5.99, x 52.5..55.5) hangs over void from the display side (faces at
  x 14.9/31.7/45.2 at y 6.3 [D probe rays]).
- walkreal_0700000c_live: 2 passed 0 failed (15.9 s).
