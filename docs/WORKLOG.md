# WORKLOG

## stuck-guard-combat-sweep

- Task: extend the stuck-guard sweep to combat movement and the 07000005 helipad.
- Branch `s/stuck-guard-combat-sweep` created from `origin/master` (26543436).
- Plan: (1) add the combat movers to the sweep's moving set (cover seek / run to cover,
  `DroneMove_NoBunching` waits, `DroneAnim_SetCombatMoveAnim` 0x13cfb0 anims, strafe/approach);
  (2) on 07000005 walk Bond into the helipad courtyard with real input, engage guards, log guards
  stuck in a moving combat state; (3) fix the class in shared code with [M] cites, sister tests.
