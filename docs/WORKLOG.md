
## level-13-warm-chain-margin-7 (2026-11-14T04:41Z, branch s/level-13-warm-chain-margin-7)

- Claimed. Base: origin/master 385ebb66d. Task: the 07000013 real-damage walk
  (`walkreal_07000013_live::walk_the_warm_chain_to_the_front_entrance_and_out_real_damage`)
  un-ignored — read the ignore message's update-4 ledger and take its next lever (enter the
  security room with a full clip, or answer #9 from a line that reaches the east corner).
  Real input only; the other walks byte-identical. Tests: walkreal_07000013_live,
  interactables_07000013_live.
- (04:41) branch cut from origin/master 385ebb66d, WORKLOG started, checkpoint pushed.
- (04:41-06:05) Reproduced the t4958 death byte-identical; re-dated the wake law (#9 wakes on
  the drain's gunfire at room range, not on movement — the -20.0 lands t4869 INSIDE the drain's
  waits, byte-identical with and without post-drain legs); measured the sleep dump DEAD (the
  dump's rounds land ZERO on #9 — the mid-room furniture blocks the inside-spot line; the
  gunfire wakes both guards); measured the spawn resupply DEAD (the nearest pickup is the
  placed tripbomb trio — its take EQUIPS the gadget in-hand, the lobby churn died t5390);
  fixed the game's basis [x, y=UP, z] (the panel aim's pitch formula) after the floor-level
  aim hit the ramp at #9's feet; FOUND AND LANDED the real ammo lever: the pocket drops rest
  at (4.85,-369.43,22.23)/(4.81,-369.13,23.02) — a d3-only walk-over leg to (4.8,23.0) r0.6
  takes both (run 8: taken 2, clip 13 spare 9, inventory 64) — but the leg's +163-tick shift
  re-rolls the stream/#4 volley schedules and the glass band/door melts (run 8 t4378, run 10
  t4314 to the stream shotgun #7's four-pellet tick); moved the radius-45 drain to the glass
  band (d3-only). Update-5 ledger + the next levers in the ignore message. The test stays
  ignored; the sisters/interactables not run (all edits d3-gated, no shared helper touched);
  the suite did not fit. Report: docs/reports/level-13-warm-chain-margin-7.md.
