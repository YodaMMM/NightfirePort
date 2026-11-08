# agent-lvl-12-13-b: second audit pass on 07000012 and 07000013

No change here moves the shared random stream. No gameplay divergence was found beyond the ones
s/agent-lvl-12 and s/agent-lvl-13 already fix or list as open. This branch adds two walks only.

## Added
- `walkreal_07000012_live::walk_spawn_to_the_shaft_exit_real_damage_operative` and
  `walkreal_07000013_live::walk_the_warm_chain_to_the_front_entrance_and_out_real_damage_operative`:
  the real-input routes at difficulty 1. With the existing difficulty 2 and 3 walks, both levels'
  objectives are now walked on real damage at all three difficulties. Both pass on origin/master
  (12 ends at 96.5 health). They pin coverage, not a fix.

## Checked, no divergence found
- Level-gated code: every `GameState+0x0C` compare that takes in 0x07000012/13 (exact and range):
  `Player_CollisionHandler` (the y < -10 ch 0x7C latch), `Player_InitWeapon` (RamLoad, the 11 kit
  when owned[0x10] is clear), `Player_SetWatch`, `DroneFunc_OnInitDeath` (the frag drop),
  `NDrone2_AttackTalk` (the 0x0b/0x13 split), `UnderCover(Aim)Talk`, `CivilianScaredTalk`,
  `EnemyRunToPoint`, `CivilianMission`, `DroneSpawner_Control`'s kill split, `Copter_Update`,
  `Mission_Update` (0x07300013), `Trigger_Activate`'s load arm (cancel channels 0x60-0x63, gate
  p2, fallback p3). Each is in the port with the same gate.
- `GameState+0x28` readers that touch these levels: `Drone_Create` (p2 <= difficulty),
  `Lock_Create`, `Player_RamLoad`'s continue boost, `SoundTrigger_Create`, `Hint_Create`.
- ScriptPlayer parameter map and state machine (`SP_CreateScriptPlayer` 0x1db9a8, `SP_Update`
  0x1dbda8, `SP_Hit` 0x1dc470): p5 is the trigger mask (1 channel, 2 bullet, 4 use, 8/0x10 touch),
  p6 is read and written, p9 is the gate. The six SPs of the two levels read the same way in the port.
- Trigger parameter shifts in `Trigger_TouchOnce` 0x1b2b88 and `Trigger_LoadLevelCreate` 0x1b2950,
  the Touch clear in `Trigger_Calc` 0x1b1900, the SEQ_AND timing in `Trigger_Update` case 4.
- Channel web, all placements of both main maps at difficulties 1, 2, 3 [D]. One wiring the
  earlier docs do not name: BodyArmour #736 on 07000012 writes ch 0x21 when taken (p3), which
  wakes drones #712/#713 and starts spawner #774 (group 9, mode 3, max 2). The port runs it
  (probe: ch 0x21 on, #712 enabled and spawned at difficulty 2 and 3). On origin/master only one of
  the two guards comes until the first is down; s/agent-lvl-13's mode-3 latch fill brings both.
  At difficulty 1 neither template exists.

## Tests
Green on this branch: the two new walks; walk_07000012 (4), walk_07000013 (2),
interactables_07000012 (12), interactables_07000013 (10), scripts_07000012 (2), gate13 (1).

## Unproven
- The ch 0x21 ambush size has no live capture; it rests on the decompile and s/agent-lvl-13's
  reading of the mode-3 latch.
- The new walks were run on origin/master only. After the stream batch they need a run like the
  other real-damage walks.
