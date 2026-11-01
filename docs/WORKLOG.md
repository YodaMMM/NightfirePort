## onfoot-moving-accuracy (2026-11-01, s/onfoot-moving-accuracy)

Branch from origin/s/1b-real-health (b08e6d84), merged origin/master (bd00a727). Task: guard
accuracy's "target is moving" x0.75 [M DoBulletAccuracy 0x179740, the moving byte read at
LAB_00179830/00179850] — the game reads Bond's MOVE VECTOR [M DroneWeap_DoOpponentTargetting
0x179a48:33-41: the target's class byte obj+0xFF == 3 is the player object [M Player_Init
0x1910f8:76], so DL+0x1DC = |(BL+0x10, 0, BL+0x18)| != 0 on foot exactly as in zero-g]; the port
read a change of position on foot. Port the walk/strafe input vector the player code stores
[M Player_SSWalk 0x192310 → Player_Move 0x191dd0:166-170], so pushing into a wall or sliding reads
as the game reads it. Pin with tests (moving / standing / pushing into a wall; sister).
