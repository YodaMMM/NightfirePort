
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
