# s/l12-start-gap: the 07000012 idle start, what is left of the gap

No game-logic change. The shared random stream does not move. Only the ignore reason of
`the_idle_start_costs_bond_what_the_games_does` is updated to the current numbers.

## Where master stands (c229c7780)

Idle Bond at d1, health every 120 ticks: 100 / 100 / 100 / 87.75 (360) / 82.5 (480) / 56.25 /
47.5 / 28.25 (840). Game runs [L]: 86-100 at 360, 53-72 at 480, 0-37 at 840. So 360 and 840 are
inside the game's band now, and 480 is not. The gap is the 360-480 window: the port loses 5.25 there
and the game loses 25-33, which is about 8 to 10 landed 3.5 rounds in two seconds.

## Gates read against the decompile (all match)

- Sight counter `+0x270`: +1 per tick while `+0x228 & 2`, cleared on a lost tick
  [M DroneVision_ProcessDroneSight 0x176a28:198-225]. The hit gate is `+0x270 >= fptoui(2.0 * 60)`
  beyond TooClose, `+0x41 == 0`, forced hit under 3 u [M DroneWeap_DoBulletAccuracy 0x179740:93-104].
  The port (`fire_weapon`) is the same. First rolled hit that passes: t294.
- Hit chance: 100 - 5 * acc, TooClose, motion latches, Easy factor [M 0x179740:52-90]. Same.
- Fire path: `+0x3b` request, `DoFiring` `+0x274 < 0x10` gate, `+0xbc4` time, `Ready2Fire`
  [M 0x17a180, 0x17a080]. Same.
- BurstDelay: Normal * aggression factor * (0.5 + FRand(0.5)), Min/Max/MinDist/MaxDist clamps
  [M 0x179ba8]. The port's constants are the tuning values 15/45/60/4/30. Same.
- AimStandFire's re-pick: `DroneFunc_CombatState` case 0x10, clip, route/range, unseen, cover, then
  grenade + `ChooseCombatMove` behind `+0x568 & 0xffff00ff == 0xa0002` [M 0x148338 case 0x10]. Same.
  AimCrouchFire's 120-tick gate [M case 0x18]. Same.
- `ChooseCombatMove`: Rand(15) pre-roll, the option list, the step/crouch/stand cascade, the dedup
  [M 0x150c18]. Same (traced per call on this start: north guard lists [strafe, step, crouch, stand],
  the step fails on both sides, so he never steps).
- Stand/crouch swaps: AimStand ENTER clears `+0x3b` and fires only after the anim-done message 5
  [M 0x166178 case 1/5]. AimCrouch keeps the request but `Ready2Fire` refuses the transition
  class. StepAim ENTER clears `+0x3b` [M 0x167590 case 1]. Same.

## What the port does in the window

- South guard (7, weapon 0x18, 3.5 a hit): t382 `ChooseCombatMove` picks the stand (0x70 to 0x67).
  AimStand plays danim 0x25 (class 5) for 90 ticks with no fire, back to 0x68 at t472. Of its
  four rounds before that (t364-382) one lands.
- North guard (6, weapon 0x19, 1.75 a hit): stands in 0x68 at (13.45, 1.0, 18.54) for the whole
  start. His line to Bond passes about 1 cm from the vending machine's south face (x 17.4-18.5,
  z 18.98). 3 of his 5 rolled hits from t294 to t480 do not land; the earlier census trace
  (s/guard-accuracy-census) put such rounds on the machine face at about (17.85, 1.05, 19.02).

## Unproven, needs a live read (one PCSX2 run, Bond idle at the start, d1)

1. Log both start guards' state (`Drone+0x10c`) and obj+0x30 every frame from 300 to 600. Does the
   0x18 guard go 0x70 to 0x67 in that window, and how many frames does 0x67 last?
2. At `DroneWeap_FireWeapon` 0x178f68, log the muzzle (fStack_150) and obj+0x30 of the 0x19 guard
   for 20 shots. The game's north guard was seen at (13.9, 17.1-17.5) in an earlier capture; the
   port's never leaves (13.45, 18.54). If the game's guard does reach that spot, its step pick
   differs (both of the port's step probes refuse: the left on the 0x8 side bit, the right on the
   move test).
3. obj+0xEC (the miss wobble phase) for both guards: misses can land when the wobble passes zero.

## Tests (this branch, cargo -j 4, NF_SILENT)

All green on this branch: start_pair_07000012_live 1 pass (band test ignored), start_rounds_07000012_live 1,
enemy_accuracy_live 5, walkreal_07000012_live 4, walkreal_07000011_live 4, walkreal_07000013_live 4 (1 ignored, as on master).
