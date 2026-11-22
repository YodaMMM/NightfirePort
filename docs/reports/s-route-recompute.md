# s/route-recompute

Moves the shared random stream: a guard routes from a different place on some ticks, so later
draws can land on different ticks. Kept on this branch.

## When the game rebuilds the guard's combat/goal route (`Drone+0x860`)

The route's valid bit is `route+0 & 0x10` (`AINetwork_RouteIsValid` 0x15a7c0). It is set only by a
successful `AINetwork_CalcRoute` 0x156ba8 (direct MoveTest pass, or A* result < 2). It is cleared by:

1. `AINetwork_InvalidateRoute` 0x15a7e0. Callers on the drone route: FindSafetyFromScaryPosition
   0x152250, FindCover 0x152548, FindAmbushCover 0x152958, FindAlarmPoint 0x153008 (both arms),
   FindRunToPoint 0x153168, ReFindMissionPath 0x153a30, SetSoundAlertRoute 0x154aa8,
   SetDroneAlertRoute 0x154b20, SetRunToPosition 0x155b30, InvalidateAttackRoute 0x155020,
   InterogateAssist 0x1616c0, CombatOutOfRange 0x164690 (blind-cheat arm), SearchArea 0x165bf0.
   InvalidateAttackRoute callers (SP): Attack 0x164210, CombatNoRoute 0x164c08, AimStandFire
   0x166328, ProneFire 0x166a28, AimCrouchFire 0x167060, Obstructed 0x173390, the four ninja flips.
2. `AINetwork_SetupRouteToObject` 0x158948 / `SetupRouteToPosition` 0x158b10 (`& 0xffaf`), reached
   from `VerifyRouteTarget` 0x15ae68 / `CalcRouteToObject` 0x15aee8 when the route's AITarget is
   not the player's global target, or the player target's stamp (`+0x54`, the frame
   `NodesForPosition` 0x1586a8 last ran, which `BuildAITarget` 0x15ac78 re-runs once the player is
   2.0 from the cached spot) is newer than the route's (`route+8`).

`NDrone2_MoveToObject` 0x151618 recomputes on (1) or (2). `NDrone2_MoveToGoalPosition` 0x1517d8
recomputes on (1) only. `ResetRoute` 0x15aa00 clears `0x40`, not the valid bit.

Nothing else touches the bit: no door, link-flag or timer path writes it (`Doors_Calc`'s `& 0xffef`
is the door record's own flag, `LinkCreep_Calc`'s is `route+4`). Door locks and danger-flagged
links only change the next A* cost or result.

## Port gap fixed

- `find_alarm_point` (RunToAlarm): it now drops the kept route in both arms [M 0x153008:20, 48].
  Before, a guard still holding a goal route followed it to the old end on the way to the alarm.

## Port gap found, held back

- AimStandFire 0x68 ENTER and AimCrouchFire 0x70 ENTER, for non-snipers: the game invalidates the
  chase route [M 0x166328:35, 0x167060:26], so the next chase recomputes it even while the player
  stands within 2.0 of the cached spot. The port keeps the route. Porting it (branch
  `s/route-recompute-aimfire`, with its own test) turns two walks red that pass on master:
  walkreal_07000001_live `walk_from_the_spawn_to_the_exit_with_the_contact` (the contact dies and
  the restart exit posts) and its `_real_damage` twin (Bond dies), and walkreal_07000014_live
  `walk_the_tripbomb_route_pick_up_place_and_shoot_it` (retreat 11.995 u against the 12 u floor).
  It needs those walks re-tuned or a live look at the 01 escort fight before it merges.

Unchanged and matching: the player-target stamp gate, every other caller above that the port
models, failed builds leaving no route.

## Not changed

- The game's `MoveToGoalPosition` reuses ANY valid route, a chase route included. The port rebuilds
  when its kept route is a chase route (stamp is not `u32::MAX`), and `set_goal_to_closer_position` /
  `set_goal_to_drone_with_los` drop the route before trying, which the game's 0x155e10 / 0x1543f8
  do not. Here the port rebuilds where the game does not. It is unproven whether a valid chase
  route is ever present at those calls in play, so this is left as is.
- Non-player opponents: the game re-runs SetupRouteToObject every call (the drone's own AITarget
  has `+4 = 0`). The port's chase always targets the player.
- ProneFire, Obstructed, SeenDeadBody and SeenSurrenderedDrone have no live producer or no port
  state.

## Tests

- `drone::abseil_tests::the_alarm_run_drops_a_kept_goal_route`: fails on the old code, passes now.
- enemy_accuracy_live 5/0, walkreal 07000001 (contact walks), 05 17/0, 0c 6/0, 12 4/0, 14
  (tripbomb walk) pass with this change.
