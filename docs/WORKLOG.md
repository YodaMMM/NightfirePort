## retune-nb-03
- Branch s/retune-nb-03 created from origin/master (6fe4339a1, one commit past 3412a89a3).
- Task: re-tune walk_the_full_route_and_complete_every_objective_real_damage and its _00agent sister
  (level 07000003) for the 3412a89a3 guard spacing-scan / close-range alt-attack changes: read both
  ignores, find where the new fights cost the health, change route/stance/fight order there.
  Helpers stay gated to the level and the difficulty. Real input only, no top-ups, never lower
  damage or health, never change guard code.
- Retune landed for the Operative (d1) walk: the fights ride the glass-fights laws (the d2
  sister's own wrapper: head-hunting + glass sight + hurting-first) and three d1-gated
  proactive aimed stands answer the exit-lawn cluster (#5's 19-hit/40.5 volley, #3/#4's
  2.1-2.7 u close) and the east-lawn sniper class before the point-blank legs. GREEN:
  walk_the_full_route_and_complete_every_objective_real_damage finished 70133 ticks at
  69.0 health (9 hits, 31.0, 16 fights); the plain d1 walk and the d2 real-damage sister
  stayed green (6 passed / 0 failed in walkreal_07000003_live; interactables 9/0).
- The 00 Agent (d3) frontier moved t8396 -> t30603 with two levers (measured on this
  branch): the lane double doors shut behind the mid-courtyard crossing (1 press toggles
  both leaves CLOSED [M Door_Activate 0x135be0:71-74]; the run died at t8396 because the
  crossing left the doors open and #9 burst through the doorway at 2.9 u), and
  escalating_cooldowns for the ditch machine gun's unexposed re-fights. The next death is
  t30603 on the moat-west legs: #12 advanced to d 1.0 (37.6 in 4 hits); the ignore message
  carries the ledger and the next lever (the ditch-floor aimed stand after the scoops, and
  arming the door-shut press before the first follower crosses).
- Sandbox notes: background jobs die between tool calls here (the suite ran in foreground
  chunks); rootfs is 9.9 GB and the full four-crate test build does not fit beside the ISO
  (CARGO_PROFILE_RELEASE_STRIP=debuginfo kept the touched binaries buildable; nf-view test
  executables land in build/<pkg>/<hash>/out/). The full suite did not fit the window; the
  touched binaries were run whole instead.
