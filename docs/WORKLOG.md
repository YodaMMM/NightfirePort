
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
- (06:20) Rule-10 coverage run: walkreal_07000013_live whole binary — the d2 agent sister 1/0,
  the d1 operative sister 1/0, the plain sister 1/0 (all d3-gated blocks skip, legs
  byte-identical); interactables_07000013_live 10/0. Report updated with the results.
- (06:45) Runs 11-12: the fights' aimed_fights tight gate at the door phase (run 11) changed
  nothing (the killer is the aim-raise statue inside the stands); the file-local pool-watch
  abort in aimed_stand (run 12, abort at 6.0 health+armour) handed the stall to the fights'
  stand — same death t4310. The next session's first item: the rule-7 opt-in pool abort in
  damage_log.rs::stand_and_fire. Ledger updated.

## level-13-warm-chain-margin-8 (started 2026-11-14 07:4x UTC)

- (07:40) Branch s/level-13-warm-chain-margin-8 created from origin/master; origin/s/level-13-warm-chain-margin-7 merged (was not on master; fast-forward to 194df6378). Reading margin-7 report + the ignore message next; ISO download starting. Plan: take the pool-abort lever (rule-7 opt-in in stand_and_fire) the report names for the door-phase aim-raise stall, judge on its own real run, then carry the warm chain out the front entrance.
- (07:55) The margin-7 lever LANDED: `pool_abort_stands` (opt-in, DamageLog field + the stand_and_fire bar) + the file-local aimed_stand bar re-law + the glass-band drain skip for #4. The walk stays RED - the runs 13-17 ledger (flat 6.0 aborted the fights at t1 on #4s single rounds; the fraction bar traded fight#4 to 54% then the reload-window burst; the drain stand on #4 cost 57 pool for 0.5 landed; the skip moved the drain to the mover #6 and #4 killed Bond at the band t4234). The sharp next lever: the #4 LINE probe (his box opens at d 37-42 but the zero-spread taps land ~0.5 total). The ignore message carries the ledger + the next lever.
- (09:05) Coverage: walkreal_07000013_live whole binary 3 passed / 0 failed / 1 ignored (the sisters byte-identical), interactables_07000013_live 10/0. The full nf-view test-tree compile (every walkreal binary per rule 7) ran 2x10-minute build windows without finishing - NOT verified this window; the field is a derive(Default) bool read only behind the flag. The suite (tools/test/suite.py) could not run backgrounded in this sandbox (nohup killed between tool calls, verified twice) and did not fit in the foreground window.
