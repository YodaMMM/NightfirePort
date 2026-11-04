
## audit-activate-side-effects
Started 2026-11-04 18:32 UTC. Branch s/audit-activate-side-effects cut from origin/master
2b02ff2ec. Task: walk every *_Activate/*_Deactivate/*_Create/*_Delete in the decompile for
on-foot objects (monitors, switches, turrets, locks, Q-worm, ladders, lifts, doors, pickups,
cameras); for each body call (camera, HUD, input, weapon, sound, player state) find the
port's counterpart; report a table game call / address / port site or MISSING; fix each
player-visible or audible MISSING with the address and a real-input test. Skip driving.
Motivating example: GunImp_Activate 0x132f00 calls Camera_CalcViewAngles(30) and the port
never did. Done = the table, fixes tested, every walkreal_*_live green, the suite total line.
- (18:32) branch created, WORKLOG started.
- (19:00) the audit sweep done (two subagents + own reads): the only Activate-family functions that touch
  camera/HUD/input/sound/player directly are Monitor_Activate 0x1b0058, Lock_Activate 0x1aed00,
  Door_Activate 0x135be0, GunImp_Activate 0x132f00, GunImp_Deactivate 0x132c18, Car_Activate 0x131c48
  (driving — skipped per task) and Trigger_Activate 0x1b2318 (level flow). The GunImp 30° mount FOV the
  task cites is ALREADY in the port (nf-view/src/lib.rs view_fov, tested by
  0700000d_gunimp_door_live the_mount_narrows_the_view_and_the_dismount_restores_it). Lock_Activate's
  two Sound_Play3D ids are 0xffff on every shipped lock [D, lock.rs sweep citation] — unobservable; the
  port's door refusal sound, monitor pane swap + input freeze, gunimp sound stop + weapon restore are
  all present. The full suite is grinding in the background (suite.out).
