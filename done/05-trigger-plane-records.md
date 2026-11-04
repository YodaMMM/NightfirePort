# trigger-plane-records — land the game's trigger contact: parse the glist plane records

Continue origin/s/trigger-hittest-segment (merge origin/master first) and read docs/reports/trigger-hittest-segment.md. The law is ported as `seg_touches_tri_raw` in crates/nf-game/src/triggers.rs; wired (commit e7f80cd6) it made walkreal_0700000a_live::low_beam_621_fires_standing GREEN but broke fix_truck_live (05 spawn plate ch 0x45) and the office-ring walks, because the one-sided entry's sign came from the tri winding. The game reads the glist's OWN plane records: each tri's 4th u16 indexes a 4-i16 (n x 1e-4, d) quad [M asm 0x10a283-0x10a1e8].

Do: parse the plane records in the level parse, feed `seg_touches_tri_raw` the record's plane, re-wire `touched_by` (r > 0) and the standing `player.body` (the diff is e7f80cd6), retune the 0a ring's #614 jump press to ~1.8 (the route-side legs are in e7f80cd6).

Done = low_beam_621_fires_standing un-ignored and green; walkreal_0700000a_live, fix_truck_live, interactables_07000011_live and every trigger/beam/laser test green; every walkreal_*_live green; the full suite total line. List each binary with its count.

---
claimed-by: sandbox7 at 2026-11-04T03:10Z

---
## Report (done, 2026-11-04T06:34Z)

Closed by the driver: the report is on `s/trigger-plane-records` (docs/reports/trigger-plane-records.md); the worker didn't run finish.sh.

---
## Report from the worker (blocked, 2026-11-04T06:35Z)

