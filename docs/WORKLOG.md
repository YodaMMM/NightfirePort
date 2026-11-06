
## level-14-ignored-walks
Started 2026-11-06 06:20 UTC. Branch s/level-14-ignored-walks (from origin/master c2c4a6efd). Task:
walkreal_07000014_live's ignored tests — read each reason, fix what can be fixed test-side on current
master, remove the ignore when green. Real input, no teleports; no [L] pin loosened. On this master the
file carries one real #[ignore]: walk_the_infiltration_to_the_delta_exit_real_damage_00agent (the 1.72
landing resupply wedge); the other two named by the driver were already un-ignored (stale-note comment at
the top of the file).
  Results (2026-11-06 07:45 UTC): on this master the driver's "3 ignored" is ONE real #[ignore]
  (the other two matches are the stale-note comment lines; both tests sharing the helper were
  un-ignored by the merged work). The 00agent run MEASURED NEW on this tree: not the recorded
  26340 wedge — a real-damage DEATH at tick 4587, 20 hits for 100.0, of which the trench guard
  (placement #270) took 65.0 in 13 hits at d 1.2 (the torso-gated loose taps lost the race) and
  the dry walk died at the corridor fetch. Three test-side levers landed, all d3-gated behind
  thread-locals in the level's own file (the d1/d2/real-damage paths byte-identical): (1) the
  trench fight answers with the return-fire stand (switch_to_gun + stand_and_fire — the head-hunt
  box, the back_off 4.0 band, the real R reload): 15.0 in 3 hits; (2) the guard's own drop is
  scooped with the bounded loot pass (the clear_hostiles pattern, 9 u / |dy| < 1.0, the 0.6
  arrive the take radius needs [M 0x18e148:203-214]) — [take] tick 643 weapon 0x14 rounds 11;
  (3) the taken gun is DRAWN with the wheel (the direct switch_to press is the gadget draw and
  was a measured no-op on a gun; weapon_cycle + the 60-tick draw wait): clip 21 spare 0 in hand.
  Shape after: 15.0/3 hits through 48 return-fire fights (all free), health 85.0 held to tick
  15300 — the NEW blocker is the walk_to "the 1.72 landing" 6000-tick cap from (32.13,2.70,39.83)
  (the dry stands' crowd-backpedal displacement; the direct steer's segment cut 0.36 along). The
  next levers are shared-file opt-ins (damage_log.rs, lesson 7's whole-suite re-run): the AIM
  guards' stands fail "unseen" on the gun-ray pick while the plain eye-ray is clear, and the
  proactive-fight gate fights guards that never dealt damage (48 fights × the backpedal = the
  displacement). The recorded 26340 hover pocket was not reached. Whole binary: 9 passed; 0
  failed; 1 ignored (151.22 s). Report: docs/reports/level-14-ignored-walks.md.
