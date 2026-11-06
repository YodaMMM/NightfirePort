
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
- [22:22 UTC, Nov 6] real-health-00agent-03-5 claimed; branch from origin/master e4fb1c512
  (WORKLOG restored from s/real-health-00agent-02-03-4). Next: merge s/door-walk-03b, re-run
  the 00 Agent walk on that tree, read the death ledger, then the levers (door-9 high-health
  corridor start, (35.6, 8.7) door stall, exit-lawn #16 volley, stand_shooters_down if the
  ledger shows far-shooter bleed).
- [23:25 UTC, Nov 6] real-health-00agent-03-5: the 00 Agent walk re-run on the merged 03b tree
  (master e4fb1c512 + s/door-walk-03b, the 03b doc rows taken over the stale sweep rows) and
  the ledger read: death t404475 at (22.24,5.46,-13.99), 28 hits for 100 — #17's moat-west
  point-blank P99 burst 60.0 in 4 volleys (the NF_DMG_TRACE run: ticks 14353-14576 at d
  2.3-4.6, the fight machinery's failed-fight cooldown covered the whole burst, killed true
  only at 14690), the mid-courtyard wedge #9 28.8 (ticks 7734-7845, killed true 7890, the
  same cooldown cover), the corridor cluster finishing 11.2 at 11.2 health. Eight runs of
  levers: the 06 aimed-stand sweep (test-local, recency 600->2400) never engaged (the bursts
  land between sweeps), the leg-skip experiment rushed the lane cluster into the door gap
  (80.8, run 2), the trip-1 clear_near woke the dodger #8 (47.2, run 6), the 8-u aimed clear
  bailed into the same trap (59.2, run 7), the point-blank aimed stand stood in the line and
  died fastest (71.2 in 3 volleys, run 8) — every stand feeds the shooter at d3. All
  experiments reverted; the test file is back at the merged shape with the ledger + the next
  lever (an OPT-IN walker.rs fights flag: immediate re-engage when the 240-tick cooldown
  delays refighting a guard within 8 u who is hitting Bond) in the ignore message. The d1/d2
  sisters and the whole-03 binary re-run on the final tree (see the report).
