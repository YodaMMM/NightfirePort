# Stream batch: void recovery out, guards re-acquire

Branch from `origin/s/stream-batch-4`, merge current `origin/master`, push to `s/stream-batch-5`. Read docs/reports/stream-batch-4.md.

1. Take void recovery OUT of this branch: revert b5e916d84's changes to player.rs, world.rs, scene.rs and drop void_recovery_07000003_live. It is held: it pushes Bond back at valid spots (`find_cel` gaps; 4422 passed, 12 failed on its own). Do not merge `s/void-recovery*` unless it is on origin/master.
2. Re-enable the re-acquire exit in `combat_state` (`if false &&` in drone.rs) with the game's own 0x5c shape [M DroneFunc_CombatState 0x148338:67-70/151/235, NDrone2_DSTATE_CombatNoSight 0x164838:26-35], your report's reading. Guards freezing in no-sight is a player-visible bug. It once broke the abseil, door-chase and bathhouse guards on 06/09/16: find out why, fix those causes, re-prove those walks. If it is the port's LOS ray failing at those spots, say where, compared with `RecurseBoxesSph` 0x1EDAE0.
3. Re-verify the 03 composed blocks, then the plain walks 01/06/08/0b/0c/11/12/13/14/15/16, `mayhew_hold`, `fix_npc_doors`, the enemy_accuracy re-pin, the start_corridor d3 owner point.
4. Then real damage, Agent, 00 Agent. An ignore with a measured reason is allowed for 00 Agent walks only.

Do not loosen pins tagged [L] and do not change game code to suit a walk.
