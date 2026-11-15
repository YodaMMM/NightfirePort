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

## real-health-00agent-03-captors-6 (2026-11-15)

- (03:52) Branch created from origin/master 298f2f96c-line and merged origin/s/real-health-00agent-03-captors-5
  (its lever commit bbde5ed94 carries the ledger; the walk stays ignored). Setup: work-slim + the onfoot ISO
  extracted, rust stable 1.100.0, ALSA headers in a user prefix (no root). Baseline runs next: the levers are
  the trio wake race (the arm must precede the plate) and the lane-mouth health margin (run the mouth colder /
  from the far side), per the captors-5 ledger.
- (03:44-05:25) real-health-00agent-03-captors-6: 21 real-input d3 runs (29-49). The trio wake race
  FELL: the wake sweep found nobody because the east-lawn door (22.3,10.9) stands CLOSED and the trio
  bunches behind the leaf — the door funnel (own-door E-open + the duel line (22.3,9.6) + the serial
  sweep) exposes all three; the arm FELL: run 38's loot diag shows the lane kills' guns sit in the
  moat ditch below the lane's east edge and the loot's sight filter declined them on every leg — the
  d3 ditch scoop takes them (the wheel at the wake went 11 rounds on one gun to 0x12:7 + 0x2:4 +
  pools); run 43: #3/#4/#5 ALL corpse true at zero wake damage. The cold mouth REFUTED (run 31:
  death 10009 — the crossing needs the pre-crossing stands). NEW BLOCKER: the mouth walk's A*
  detours over the door-8 area's own wake ((39,12.9) door open channel / the (38.9,11.1) plate) and
  #14 (0x5000039, 0xe, load disabled (30.7,4.1)) pops and bursts 41.2 in 3 hits — death 67488
  (runs 43/49). The south-legs and leg-0-sweep shapes tried and measured (runs 45-48). The walk
  stays ignored; the refreshed ledger is in the ignore message (follow-up captors-7).
- walkreal_07000003_live WHOLE on the final tree: 6 passed / 0 failed / 2 ignored (222.58 s);
  interactables_07000003_live WHOLE: 9 passed / 0 failed (30.35 s). The full suite did not fit
  the window (the build phase alone exceeds the remaining minutes after the runs above).
