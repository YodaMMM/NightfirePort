# s/nb-scan-live

**This branch moves the shared random stream.**

## What the task assumed
The task said the guards' spacing scan (`DroneMove_NoBunching` 0x150ad0) was still gated behind
`NB_SCAN_LIVE`. It is not. The const was removed by commit 3412a89a3, which runs the scan on every
level. Only stale comments still named it. Those comments are updated here.

## Fix: ChooseCombatMove no longer parks a guard in 0x5E
`choose_combat_move` returned COMBAT_WAIT 0x5E whenever the scan's bit 2 ("drone ahead") was set.
The game does not do this:
- `NDrone2_ChooseCombatMove` 0x150c18 never reads +0x31C. It sees the scan only through the +0x17
  return byte (`cVar1`, :58) and the side bits that each Can* probe checks first [M].
- The game's only bit-2 readers are `DroneAnim_SetCombatMoveAnim` 0x13cfb0:35-39 (SetState 0x5E)
  and `DroneFunc_CombatState`'s 0x5A arm, which plays anim 0x21 [M 0x148338 case 2]. CombatState
  therefore reaches 0x5E only from 0x5C's move arm [M 0x148338 case 4].
- I checked the port's `combat_state` against 0x148338, arm by arm. Every arm matches (0x58, 0x5A,
  0x5C, 0x5E, 0x68, 0x70).

Effect in play: a guard standing next to another guard in CombatNoRoute, or in a fire state, now
rolls its normal combat move instead of freezing in 0x5E.

New test: `drone::combat_move_tests::a_drone_ahead_does_not_send_the_combat_move_choice_to_0x5e`.
It fails with the old line restored and passes on the new code.

## The 07000014 door hold (still ignored, blockers rewritten)
- The old test doc said the scan sets both side bits (8 and 0x10). That is wrong. The scan sets one
  side bit per tick, on the neighbor's side [M 0x150ad0:42-48].
- A port probe at the [L] spots (-51.6,41.2) and (-51.2,38.8), facing Bond, found one refused side
  for each guard. The north guard's left step and the south guard's right step both fail
  `bounds_test`. The step toward the other guard carries the side bit. So at those spots every step
  is refused and 0x5F holds, which matches the live picture.
- In the port at Agent, the walkers stop on their mission-path ends [D paths 5/7 dest
  (-50.98,38.98) / (-51.07,41.06)]. The first walker waits in 0x5F while the second is still more
  than 3.0 away, so it has no side bit. It steps toward the second walker, and the pair never
  settles where both sides refuse.
- At Operative only #783 reaches the door.
- Unproven, needs a live check: the [L] boot's difficulty, and whether the game's first walker
  steps before the second one arrives.

## nav_kick_live drone_directly_ahead_parks_the_mover_in_0x5e
This ignored test passed only through the removed bit-2 return. It keeps Bond in view 6 u away, and
the decompile has no 0x5E path for a seen guard. Its ignore message now says this.

## Walk re-tune
- `walk_the_infiltration_to_the_delta_exit_real_damage` (07000014, Operative) died t29712 at the
  escape-touch leg with the 20000/60 pause.
- Master passes with 20000/60 (re-run here).
- I swept 9 values. 5000/60 and 15000/60 pass. 1000, 2000, 10000, 25000 and 28000 /60, plus
  20000/30 and 20000/90, die.
- Committed: 15000/60.

## Tests
See the final report lines.
