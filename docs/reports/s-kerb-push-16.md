# s/kerb-push-16: the 16 ring kerb foot holds

Trajectory-shifting change: every walk's floating-point path moves, so the long real-damage walks
on 01, 0c and 16 need re-tuning before this merges (list below). It does not touch the random stream itself.

## Cause
At the kerb foot Bond rests with |v| on the knife edge r = 0.55 of `Intersect_CylGeom` 0x109da0's
push arm, and the pick bit that sets the feet flag lives inside that arm [M :447-513;
Player_FeetOnPoint 0x194188 ORs flag 8 on HITTEST+0x9c]. The port did the arm in
round-to-nearest f32: each tick the sub-micro push still moved the root up one ulp, |v| reached r
after about 8 ticks, the contact dropped, one gravity tick fell, and the slope's push walked Bond
0.001 down and 0.0013 south. Repeated, the 70-tick walk ended at (6.53, 6.8567, -74.9925) against
the game's (_, 6.88037, -74.96214) [L, test comment].

## Change
- `cyl_game_contact` and `game_dist2tri` (world.rs) compute in the VU/FPU rounding of ps2float
  (toward zero), with the decompile's op order: `vmul` + two `vaddbc` dots ((x+y)+z), v̂ = v·(1/|v|),
  mag = r − |v| [M 0x109da0, 0x109770].
- The walk's `pos += push` is the handler's one VU `vadd` (chop) [M 0x1949a8:136-140].
Both halves are needed: either alone still fails the test (6.8691 / 6.8690).

## Tests
- ring16_kerb_live: un-ignored; fails on master (6.8567, -74.9925), passes here.
- rest_height_live 3/3, collide_push_live 6/6, nf-game lib world:: 15/15 and player:: 21/21: green.
- walkreal_07000005_live 17/17 green.
- walkreal_07000016_live 15/3 (master 18/0): the_chasers_punch_lands_the_record_damage and
  walk_the_chasers_defeat_completes_the_mission_on_real_damage die in the fights (t4916, t16228);
  walk_the_chasers_defeat_completes_the_mission now also owns weapon 26 (0x1a) at the chaser fight.
- walkreal_07000001_live 4/3 (master 7/0): both real-damage walks and following_mayhew_down fail.
- walkreal_0700000c_live 4/2 (master 6/0): real_damage_00agent dies t3609; real_damage_agent
  wedges at "the deck south" (51.17, 1.73, -162.12), 3.1 from the target.

## Unproven
- That ps2float's chop model holds for every op here (the EE `sqrt.s` and the `vopmsub` cross are
  modelled as chopped mul/sub). The test passing is the only check.
- Whether the 0c deck-south wedge is a route change or a real step-up regression: not traced.
