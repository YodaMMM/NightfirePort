## door-scan-skip-bit (started 2026-11-01 20:52 UTC)

Task: the door auto-close scan (Door_Update 0x1363c8) skips any object whose obj+0xF0 has bit
0x10 [M 0x136678-0x136684]. Live reads: 07000001 player (0x890) and drones (0x8B0) have it SET,
the 07000005 player (0x880) has it clear [L]. The bit's writer is unported, so drone.rs
(~line 17476, the player_actors/actors lists) passes false for every actor and door.rs (~line 305,
DoorActor) documents it. Find every writer of obj+0xF0 bit 0x10 in ACTION.ELF [M: cite each] and
the condition that sets it per level/object class; carry that flag on the port's player and drones
and feed it into the actor lists, so a door's auto-close ignores exactly the bodies the game's
does. Pin the flag per level with tests ([M] for the writer, [L] values above as the check), plus
one door behaviour that changes on 07000001. Branch cut from origin/master 6042ee53.
