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

## Progress (2026-10-31, ~23:59 UTC) — the boarding closed from the data, the question pinned

- probe_crane21.rs (three print-only probes, committed): the CATWALK BAND scan (z −282..−293 ×
  x 133..194) killed the yard-catwalk hypothesis — only the yard 3.98, the parked gantry
  14.32/13.47 and two runway pillars (x 154 / x 169, tops 16.07, portals for the shuttle,
  unreachable); the ROOF MAP (x 130..190 × z −270..−305) found the 18.6 canopy covering the
  gantry's WHOLE swept footprint (edges clear of it: north z ≈ −267, west x ≈ 130; the 20.2
  rooftop structures over the east part with their own slot at x 175..179 down onto the
  canopy) — no roof edge or hole ever passes over the ride; the SWEEP (E #460 from the bay,
  every 8 ticks for the shuttle) verified segment_hit tracks the moving gantry live
  (13.47/14.32 readings travel with the origin).
- The script closure: 0700000d.bin's script 0x060009EF decoded — nine streams, NO player
  hand-off (no event 4 Player_Disable / 6 Player_Enable [M Script_EventHandler 0x1d94c0]); its
  event 3 (p0 = 0) is the negative-delay hold. Stream 2 spawns the riding 0x0200_0a81 as a
  0xEA trigger with params [103, 8] — the game's ABOARD DETECTOR (out event 103 = ch 0x67,
  mask 8 = player walk-in), a 2-tri flat plane at local +2.95 (y 14.28), x ±2.44, z ±16.80 —
  it can only fire for a body already at deck height. Placement #116 ShippingCrane_TouchPlate
  is a second copy of the same detector (hash 02000a81, out ch 0) [D the placements dump].
- walkreal_0700000d_live's boarding block now pins the closure with three new sister assert
  groups (the canopy columns; the no-hand-off script sister via a new boarding_script_ops()
  helper that parses the level bin from the disc; the detector's exact geometry) beside the
  pre-existing east-side closure asserts. The ONE pos write stands ([S] unverified); the ride
  and the drop after it are real input. No port source changed.
- Totals: walkreal_0700000d_live 1/0 (37.3 s), interactables_0700000d_live 11/0 (29.8 s);
  nf-game ran clean through its alphabetical binaries in the window. Chunked fast runner on
  this tree: 307/997 binaries recorded — 949 passed, 2 failed (net_soak_faults_live,
  levels_sweep_live — both TIMEOUT rc 124 at the runner's 100 s limit, the sandbox's known
  class), 79 ignored; the remaining 690 did not build here (disk-bound at 95%; the resumable
  state is /home/z/suite_0d_state.json).
- The boarding's exact question is the report's live check 1 (the candidates left: a glist the
  port skips but the game collides, or the trolley plane being solid in the game).
