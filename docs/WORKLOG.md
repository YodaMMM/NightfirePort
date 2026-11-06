
## real-health-00agent-02-03-4 (2026-11-06)

Task: continue from master; read the -3 report (the 03 death diagnosed, the pre-clear kills the
decisive cluster; one approach problem and one stall remain; the 02 sister not started). Real
input only; no top-ups; helpers opt-in. Remove each ignore when green; every walkreal_*_live;
the full suite total line.

Progress:
- [05:15 UTC] Branch cut from origin/master (c2c4a6efd). Next: the -3 diff port, the 03 repro.

Progress (continued):
- [05:20-06:40 UTC] The 03 00agent repro on master: death t222034 — the mid-courtyard #9 fight
  32.4 (d1 costs 0.0, d2 2.1 — the accuracy scaling lands everything at d3), the west lawn 31.2,
  and the death burst: the point-blank shotgun post #5 (d 2.3) ate 31.6 in ~90 ticks with no
  fight ever answering. KEY FINDING: from the same 0x50-plate stand the d1 nav takes the east
  line and the d3 nav the north wing — the web's cross-path hops move_test the world with doors
  included [M nav_route_global; door.rs CLOSED=2] and the dump shows doors 4/5/6/9/10 ALL
  CLOSED at the d3 leg. Door-opening runs exposed the corridor cluster (#2 the captor at
  (29.1,-13.1) d 3.4, #19 d 1.8): the fights kill #2 but the exchange costs 34.0 (36.4 -> 2.4),
  death t217995. Seven runs total; the sidestep ladder/anchor never fire (the death is inside
  the first walk_nav_to's re-derivation).
- [06:35 UTC] The 02 00agent repro: death t6833 — geisha-1's shotgun captor #7 at d 3.5 ate
  88.8 in 32 hits with a DRY gun ("0x1c not carried"; the cuffs at clip 0 spare 0). The scoop
  economy needs an earlier drop.
- Report at docs/reports/real-health-00agent-02-03-4.md; the full-suite sweep did not fit (no
  crates/*/src or shared-helper change — the other binaries are unaffected by construction).
