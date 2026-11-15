# WORKLOG

## real-health-00agent-03-captors-5 (started 2026-11-15 ~01:40 UTC, branch s/real-health-00agent-03-captors-5)

- Claimed. Base: origin/master afec3f768. Task: 07000003's 00 Agent full-route real-damage walk
  (`walkreal_07000003_live::walk_the_full_route_and_complete_every_objective_real_damage_00agent`,
  still ignored) goes green. captors-4's ledger (on the master ignore message, commit b0d18af5d):
  run 20 reached 80288, run 21 died 74135 DRY at the saddle doorstep via the wedging
  (22.1,-2.0) anchor. This round's levers per the task: (2) the saddle route WITHOUT the
  (22.1,-2.0) anchor (keep only the saddle nav from the east-lane legs' end), then (3) the #7
  aimed stand once armed. Prior captors-5 sandboxes (queue claimed/*.md) measured and shipped
  nothing; their lead: Bond arrives 3+13 rounds — where the 0xe/0x1c shotgun rounds went
  upstream is the arming lever. Helpers gated to 07000003 + difficulty, other walks byte-identical;
  real input only, no top-ups, never lower damage or health. Judge each lever on its own real run.
- (01:40) Branch checked out at origin/master afec3f768 (local branch existed from a prior
  sandbox in this sandbox tree, pointing at the same commit; no captors-5 commits existed),
  WORKLOG started, first checkpoint.
- (01:43-02:07) Levers measured on real runs (each judged on its own run, real input only):
  (22) the -4 ledger item 2 AS WRITTEN (the anchorless saddle route) REGRESSED byte-identically
  to run 20's 80288 death — the saddle nav wedges 22x on the lane doors at (11.88,6.46,-20.90)
  without the anchor's wedged-bridge repositioning; the (22.1,-2.0) anchor stays.
  (23) the arming loot on the lane/moat legs' d3 clears found NO visible drop on any leg (the
  moat kills' guns land in the ditch, outside the loot's ±1.5 y floor band); the west-dragging
  resupply is gated to difficulty == 2 (d1/d2 byte-identical); the (20,3) aimed stand on #7
  engaged but its tap trace showed the gun already FISTS (clip 1 spare 0) — the return-fire
  fights on the approach walk drained the rounds; death 75056, #7 still hp 12.
  (25) the si-1 approach runs COLD (return_fire off across the anchor + the (20,3) walk, rounds
  re-checked at the stand): the #7 aimed stand KILLED #7 — first death list without him, 35 hits
  (was 59) — died 73614 PAST the saddle to the 0x23 trio wake (#5 d 3.3 point-blank).
  (26-28) the 0x23 plate-wake block (the door-8 cluster block's shape): byte-identical no-op
  until the trio gate widened to 12 u (the trio LOADS at (26.3-29.4,16.3-16.4), 8-10.5 u from
  the plate centre (22.9,6.1,9.8) — the census); with it live #5's charge STILL won the race
  (d 1.2 at hp 12, death 73318, the sweep's switch+raise never fired).
- (02:07) walkreal_07000003_live WHOLE on the final tree: 6 passed / 0 failed / 2 ignored
  (305.29 s); interactables_07000003_live WHOLE: 9 passed / 0 failed (42.28 s). The refreshed
  ledger is in the ignore message; the walk stays ignored.
- (02:20-03:06) The full suite did NOT fit: the four-crate build ran in six 570 s foreground
  chunks (background jobs die between tool calls here; the build was not Finished by the stop),
  twice pruned stale per-unit hash dirs under target/release/build (nf-view grew to 3.8 G /
  1263 dirs on the 9.9 G disk). The suite's run phase never started.
