# WORKLOG

## drone-delete-turret-blast (2026-11-06, s/drone-delete-turret-blast)

Task: `Drone_Delete` 0x136ce8 (decomp ~90-96) sweeps the object list for class-0x36
turrets owned (ext+0xb8) by the drone being deleted, sets obj+0xfe |= 1 and calls
`Explode_Create(5.0, 5.0, 50.0, obj, obj+0x30, 0x2d6210, 0x6000052, 0xff, 0xff, 100, 0)`.
Guards deploy such a turret via `GT_DeployMiniGun` 0x181560 from
`DroneWeap_FireWeapon` 0x178f68:123 (weapon-datum char 'R'). gt.rs has the MP wreck and
sibling sweep but no port code cites this arm. Plan: read the whole function and the
deploy path, establish whether the port's guards deploy the turret at all (owner stored
as the drone) and which disc placements carry an 'R' weapon [D]; if none does, say so
and keep a unit pin; otherwise land the arm at the port's drone-deletion point (delete,
not death — check the game's timing), through the shared pending-blast path, pinned by a
real-path test (guard deploys → is deleted → turret dies in a blast at its position; a
turret owned by another object stays).

Cut from origin/master 576909ed6.
