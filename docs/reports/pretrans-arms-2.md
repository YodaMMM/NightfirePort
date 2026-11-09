# pretrans-arms-2: the DroneAnim_PreTransCallback `+0x38` root-motion zero, redone

Moves guard timelines (and with them the shared random stream's consumers): guards in the
setter states no longer root-move, so later AI decisions and their draws shift. The change itself
adds no draw (0x13b490 makes no rand call [M]; `aim_run_arm` calls none). Two real-damage walks
are still red on this branch (see "Walks").

## What the game does [M]

- `NDrone2_PreDroneControl` 0x148db0:23 writes `Drone+0x38 = 0` at the top of every drone frame.
  `Drone_Control` 0x139268:48-91 runs PreDroneControl, then the state machine (ControlSTANDARD →
  UPDATE), then the anim tick whose `AnimFrameResolve` calls the PreTransCallback. A setter in
  UPDATE (or an ENTER/EXIT reached from it) is read the same frame; the next frame starts clear.
- `DroneAnim_PreTransCallback` 0x13b490:85-118 (disasm 0x13b588-0x13b638): zero pending =
  `+0x38 != 0 || +0x23 != 0`; the body-contact latch arm (`+0x26`) clears it (0x13b624
  `v1 = 0`); a standing pending runs `Vec_Zero(root)`. With `+0x1a` (free root) the motion is
  added to `+0xBE0` first (:88-94), so the zero drops it from the object only.
- Setters (every drone-side `+0x38` store in the decompile): AimStandFire case 3 with
  `HasOpponent` and `+0x568 == 2` (0x166328:118-122); `DroneWeap_AimTarget_IsOpponent`
  0x178ae0:47-53 (`+0x56a` 0x1e/0x1f, `CanMoveToRelativePos(0, 0, 0.4)` true; the `sb` is in the
  `b` delay slot at 0x178c0c; that arm also sets `+0x694 = yaw − AngleDifference(atan, yaw)·0.5`
  instead of `SetAngleToObj`); OpenDoor ENTER on a closed door (0x1681f8:25), UPDATE (:74),
  EXIT clear (:71); StandFiddle case 3 (0x1687c0:29 — handled, v0 = 1 in the delay slot at
  0x168878/0x168880; the old port comment calling it unhandled was wrong); UnderCoverInit case 3
  (0x169cc0:37); JustStand case 3 (0x173fc0:16); UnderCoverIdle ENTER clears it for the aiming
  class (0x169e58:52). Prone 0x166940:26 and Tester1 0x16db90:21 have no SP producer;
  BotCoverInit/BotCoverIdle/BotDoorOpen are MP bot states.

## What changed

- `Drone::root_zero` (`+0x38`): cleared at the PreDroneControl site of `Drones::tick` (top of the
  per-drone body, before the UPDATE send), set by every SP setter above, read in `root_motion`
  together with `air` (`+0x23`); the latch arm cancels it. Not sticky: the rejected
  s/pretrans-arms kept the flag across frames.
- `Drone::aim_run_arm`: the IsOpponent run arm, called at every SP caller's UPDATE (DrawWeapon,
  AltAttack, AimStandReload/AimCrouchReload, AimCrouch, SniperAim/Fire/Reload, StepAim*,
  StrafeAim*/StrafeDodge*, Roll*, AimBackoff); the callers keep their existing facing fold when
  the arm does not run. `+0x4F4 & 0x8000000` has no setter in the decompile (only zeroing stores
  at 0x14a2f0:11 / 0x14b300:59) and is not modelled.
- `NF_RZLOG` logs each dropped root step (guard, state, anim, position).
- docs/port-map/guard-ai.md 0x13b490 row updated.

## What a player sees

Guards stop dead when they plant to aim (AimStandFire) instead of skating on the aim/blend root
motion (`NF_RZLOG` on the walks: a 01 hall guard's dropped steps sum to 0.95 u over one fight,
the 02 geisha-1 doorway guard's to 1.70 u over 70 frames); guards
kicking or waiting at a closed door stand still; a guard still in his run anim when he starts
aiming turns half way and stops instead of sliding on.

## Not ported (with the reason)

- The hip ramp (:59-71, `+0xa0 = max(anim+0x5c − 0.02, 0.2)`, `root.y = +0xa0 − +0xa4`).
  `anim+0x5c` = the resolve's root y plus the float read of `anim+0x60` (0x1c35f8:374-378), and
  `anim+0x60` is the u16 flag word (bits 4/8/0x10/0x80 written by AnimObjectUpdate,
  Drone_CollisionHandler, Drone_FeetOnPoint) — read as a float its exponent is zero whenever
  `+0x63` is 0 and `+0x62` < 0x80, i.e. a denormal the EE FPU treats as 0 [S: `+0x63` never
  checked live]. The live walking guard holds `+0xa0` = 1.1467 [L work/patrol_trace.tsv A0] and
  stands 1.16 above the port's floor there [L 07000005 floor_below at the trace's obj.x/z]; the
  port's pose roots give 1.10-1.12 for walk/stand. The port's guard object sits at feet + 1.0
  (`head()` = pos + 0.6, the `pos[1] − 1.0` feet probes, `draw_y` = pos − 1.0 + pose hip), so the
  ramp raises every standing guard's pos by ~0.12 and with it eye, feet probes and draw unless
  those consumers move together. That is a draw change, out of scope here; it needs its own task.
- The `+0xc5 == 0x1e` difficulty scale (:77-81): no SP drone carries mode 0x1e (mode, alt mode,
  attack mode and normal mode of every drone over every `dev::LEVELS` load); the MP bots walk
  through mp/bots.rs, not this root motion.

## Tests

New `pretrans_root_zero_live` (2 tests), both red on the base tree ae8d1871e and green here:
- `just_stand_zeroes_the_walk_root_each_frame_and_patrol_walks_again`: a 07000005 patroller put
  in JustStand without its ENTER keeps the walk anim and stays put (old: 0.36 u in 40 ticks),
  then walks again the next frames in Patrol (the per-frame clear, not a latch).
- `a_running_guard_in_an_aim_state_turns_half_way_and_stops_his_run_root`: a 07000001 runner
  put in RollLeftCrouch without its ENTER takes the half-way yaw (2.908 vs half 2.909; old:
  the full 2.800) and does not move (old: 0.0706 in one tick).

`drone_bump_msg22_live` 3/3 green; `nf-game` lib 755 passed, 0 failed, 5 ignored.

## Walks (every walkreal_* binary, one per call, final tree)

Green: 01, 04, 05, 06, 07, 08, 09, 0a, 0b, 0c, 0c_barrelrow, 0c_eastroute, 0d, 11, 11_route, 12,
13, 14, 15, 16, 1b, 41, 41_wall, 4a, 4a_satchel. Base tree for comparison: 01/02/03/14 green.

Repairs, as a player plays (no ignores, no pin or damage changes):
- 01 Agent real damage: Bond shot the escort standing behind guard #3 at the stairwell head
  ("Killed Civilians"): the return-fire stand (`stand_and_fire`) had no friendly-fire shield.
  The shield test is now one helper (`Walker::shield_blocks`) used by `shoot` and the return-fire
  stand (`return_fire_shield_blocks`; walker_b2 has no shield and returns false). With the shot
  held, Bond now steps back through the corridor door while an armed guard stands at the escort's
  shoulder and fights whoever comes through (`lure_off_the_contact`).
- 02 Agent real health: the geisha-1 doorway guard #7 now holds his aim stand in the doorway; the
  body-block arm's keep-off retreat walked Bond out of the only line and six flanks later the
  courtyard sniper killed him. The body-block fight no longer backs off from the blocker in
  Bond's face.
- 02 `courtyard_sniper_pin_stair_bottom_agent`: rifle #8, planted in his aim stand 24 u east,
  fired at the stand and a miss killed the hiding civilian behind Bond; the mission fail cleared
  every guard's opponent and the sniper went quiet (0/3 rolls). Rifles other than the sniper that
  see the stair bottom are answered before the stand.
- 14 tripbomb route: the clear's loot pass re-walked an unreachable drop (Bond up on the sill at
  y 1.30) every round and wedged; a drop the walk could not close on is now skipped.

Still red (not ignored):
- 02 `walk_the_castle_all_objectives_real_health_00agent`: the base tree passed it ending at
  3.6 health. Here the south-hall trio costs 16.6 more (83.4 left), the stair-bottom legs wedge
  six 3000-tick caps under the sniper (83.4/41 armour → 35/0), the bunker-corridor shotgun #13
  takes 25.2, and the geisha-2 plate's captain #17 lands one 9.8 round on the last 9.8. Tried:
  stepping back out of the ch-0x19 doorway (Bond then dies inside the room), the shared
  `hurting_first` retarget (no change); both reverted.
- 03 `walk_the_full_route_and_complete_every_objective_real_damage`: the base tree passed at
  22.2 health. Here the #3/#4/#5 cluster engages the door-8 mouth leg at nav 4-7 (base: nav 17)
  for ~40 health and Bond dies to the exit-lane shotgun #16 at tick 181546 with 16.2 left.

Unproven: whether the game's walking guard root y (anim+0x5c) carries a non-zero `+0x63` byte
(it would make the `+0x60` addend a real number); a live read of `animobj+0x5c` and `+0x60..63`
for a walking 05 patroller would settle it.
