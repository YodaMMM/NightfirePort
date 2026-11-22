# s/sight-enable-bit

Random stream: does not move. The census below shows identical per-drone state traces and
identical end RNG positions on every level before and after the change.

## What the game does
- `DroneVision_ProcessDroneSight` 0x176a28 tests the same live gate in all three loops
  (:81, :165, :188): `+0x4F8 & 0x600 == 0`, `+0x4F8 & 0x100` (enabled), health `+0xAC > 0`,
  class != 0x11, obj+0xFE bit 0 clear. A drone failing it gets no visibility, no LOS probe, and
  the third loop's lost arm (`+0x270 = 0`, `+0x274 += 1`, `+0x228 &= ~4`) whatever its stale
  `+0x228 & 2`.
- `Drone_GetOpponentInfo` 0x138f68:16-18 returns at once while `+0x4F8 & 0x100` is clear.
- `NDrone2_ControlSTANDARD` 0x1490f0:140-143 returns before `NDrone2_FindOpponent` (:167)
  while `+0x4F8 & 0x100` is clear, so a disabled drone keeps its last `+0x170`.
- `NDrone2_DSTATE_CivilianPatrol` 0x15d610 ENTER (:20-25): `+0x9F2 == 0` (no route) goes to
  Idle 4 with no vision bit and no walk anim.

## What changed (crates/nf-game/src/drone.rs)
- Sight eligibility now uses the live gate `present && find_opponent_alive()` (enabled, not a
  corpse, health > 0, not deleted) in the visibility loop and the round robin.
- The third loop (team != 2) and the team-2 fold take the lost arm for a drone failing the live
  gate or without the vision bit, even when the LOS flag or the held drone pick is stale.
- The player-opponent fold (`opp_bond`) and the team-2 opponent scan skip disabled drones; the
  team-2 distance/bearing write skips disabled drones (GetOpponentInfo's gate, already applied
  to the player-opponent path).
- CivilianPatrol ENTER: no route goes straight to Idle.

## Count over every level
`tests/sight_enable_bit_census.rs` (ignored census): every level in `nf_view::dev::LEVELS`,
600 ticks from load and on to 600 ticks of play where the load opens on an intro, Bond standing
at the spawn. Disabled drones with the vision bit on: 07000002 d21, 0700000d d31 and d33, only
during the intro hold, where `Drone_EnableAll(0)` already stops the whole sight pass. Drones
whose behaviour changed: none, on any level. The fix is parity for states the port does not
reach in that run (an in-play per-drone switch-off; a CivilianPatrol entry without a route, e.g.
the 09/0A stun resume of an unsighted guard).

## Tests
- `tests/sight_enable_bit_live.rs`: `disabled_guards_get_no_sight_on_0700000d` (a guard switched
  off beside Bond with its LOS flag held up gets no sight tick, counts unseen ticks) and
  `civilian_patrol_without_a_route_goes_idle`. Both fail on the old code (seen ticks 4 vs 0;
  state 0x14 vs 4) and pass on the new.

## Unproven
- Whether any real play path enters CivilianPatrol without a route (the 09/0A stun resume is the
  candidate); not exercised by the census.
