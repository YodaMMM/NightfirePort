## level-0d-boarding (2026-10-31, s/level-0d-boarding)

Branch from origin/master (81da6c47, batch-1031l). Task: 0d — the last placement in the crane
chain: the boarding of SP #115's gantry in walkreal_0700000d_live::walk_arrival_to_the_c5_door
(the ride west and the drop into the yard are real input now). Read the merged
level-0d-beam-boarding history in the test comments and probe_crane20; settle the boarding
from the data [D the gantry's script-entity collision per tick of the 3000-tick shuttle; the
roof/catwalk edges it passes; M Control_InheritVelocity 0x133e88 / Player_FeetOnPoint
0x194188], walk it on real input and delete the write; if only a live run can settle it, pin
the exact question with a sister. Run walkreal_0700000d_live and interactables_0700000d_live;
quote every total. Full suite with the fast runner on the final tree.

## Progress (2026-10-31, ~22:35 UTC)

- Branch created from origin/master (81da6c47). Existing 0d coverage on master noted:
  walkreal_0700000d_live (1 test, the boarding stand-in write), interactables_0700000d_live,
  walk_0700000d_live, census_0700000d.rs, audit_0700000d.rs, docs/levels/0700000d.md +
  -audit.md. The boarding research starts next: probe_crane20 + the level-0d-beam-boarding
  history, then a per-tick collision sweep of the gantry.
