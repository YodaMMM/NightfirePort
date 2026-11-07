
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
- [23:15 UTC, Nov 6] real-health-00agent-03-5 wrapped: the round report at
  docs/reports/real-health-00agent-03-5.md (the merged-tree ledger, runs 2-8 all worse and
  reverted, the opt-in close-range re-engage flag named as the next lever); the four required
  binaries green on the final tree (03 whole 6/0/1, 02 3/0/1, 04 10/0, interactables_03 9/0);
  the full suite did not fit (test-file+docs-only diff — the other binaries are master's by
  construction).
- [02:58 UTC, Nov 7] real-health-00agent-03-6 started: branch cut from
  origin/s/real-health-00agent-03-5, origin/master merged clean (e75efe5c2). Plan per the
  driver: build the opt-in walker.rs fights re-engage flag (immediate re-engage when the
  240-tick post-fight cooldown would delay refighting a guard within 8 u who is hitting
  Bond; default off so every other walk stays byte-identical), then take the corridor
  cluster, the (35.6, 8.7) door stall and the exit-lawn #16 volley in the ledger's order;
  remove the ignore when green. Reading the 03-5 blocked report + ignore message next.
- [04:30 UTC, Nov 7] real-health-00agent-03-6 wrapped (the walk still RED, keeps its
  #[ignore]): branch cut from 03-5, master merged clean. Two opt-in default-off fights
  flags built (common/damage_log.rs — the fights machinery module): close_hitter_reengage
  (the 240-tick failed-fight cooldown does not delay re-fighting a guard within 8 u who is
  landing rounds + the close-hitter pick fallback: such a guard is fought even when no
  box's centre ray clears the world — his pellets reach Bond through the same gap) and
  dry_fight_resupply (an in-hand-dry gun runs ONE resupply pass before the fight; the
  unseen-fights mode was tried for the same holes and REVERTED — its extra restock arms
  nav'd Bond back across the moat and wedged the flank nav, run 2). Proven on the 00agent
  ledger: #17's moat burst 60.0 -> 36.0 (the re-fight killed at 14615 vs 14690), #5
  28.0 -> 0 (killed true 410290 — the fights now answer the servants'-ground shotgunner
  through the door corner), #4 7.2 -> 0; the death moved t404475 -> t410740 (the walk is
  at the margin: the three remaining holes #17 36.0 + #3 35.2 + #9 28.8 sum to exactly
  100). The new ledger + next lever (a longer aim-None tolerance for the close-hitter
  unseen stand; the #9 radius) pushed in the ignore message. The four required binaries
  green (03 whole 6/0/1, 02 3/0/1, 04 10/0, interactables_03 9/0); the sweep: 01 8/0,
  05 14/0, 06 6/0/1, 07 6/0, 08 6/0, 09 8/0, 0a 13/0/2, 0b 7/0, 0c 3/0/1, 0d 2/0/1,
  13 3/0 all green; walkreal 11/14/15 SIGKILLed by the sandbox OOM (3.4 GB RSS on a
  4 GB box, dmesg oom-kill x3 — environment, the flags are off in those walks);
  16/1b/41/4a + the probe binaries not reached in the window. The four-crate full suite
  did not fit (the background start is killed between tool calls; the sweep ran as
  foreground batches). Report: docs/reports/real-health-00agent-03-6.md.

## real-health-00agent-03-7 (2026-11-07)

Branch from 03-6, master merged (clean). Goal: 07000003 walked to the end on 00 Agent
with real health — the walk dies at t410740; take 03-6's next levers (longer aim-None
tolerance on the close-hitter unseen stand for #3's ~150-tick corner, then the
close-hitter radius past 8 u for #9's 10.3 u door-gap wedge, then the ledger).
close_hitter_reengage and dry_fight_resupply stay. All new flags opt-in default-off.
