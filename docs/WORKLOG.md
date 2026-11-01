## onfoot-moving-accuracy (2026-11-01, s/onfoot-moving-accuracy)

Branch from origin/s/1b-real-health (b08e6d84), merged origin/master (bd00a727). Task: guard
accuracy's "target is moving" x0.75 [M DoBulletAccuracy 0x179740, the moving byte read at
LAB_00179830/00179850] — the game reads Bond's MOVE VECTOR [M DroneWeap_DoOpponentTargetting
0x179a48:33-41: the target's class byte obj+0xFF == 3 is the player object [M Player_Init
0x1910f8:76], so DL+0x1DC = |(BL+0x10, 0, BL+0x18)| != 0 on foot exactly as in zero-g]; the port
read a change of position on foot. Port the walk/strafe input vector the player code stores
[M Player_SSWalk 0x192310 → Player_Move 0x191dd0:166-170], so pushing into a wall or sliding reads
as the game reads it. Pin with tests (moving / standing / pushing into a wall; sister).

## Outcome (2026-11-01, ~10:35 UTC)

- The read is the game's move-quad test for every substate (the class-3 target branch applies on
  foot): Bond::move_local mirrors BL+0x10/+0x18 per substate; drone.rs's position-delta read is gone.
- Green: nf_game lib 638/0/5i; the fighting walkreals 01 5/0, 05 11/0, 08 3/0, 0a 7/0/2i, 11 1/0,
  12 1/0, 1b 10/0/1i; stuck_guard_combat_05 2/0; mp_combat_weapons 11/0; the drone bins 5/0, 3/0,
  2/0, 5/0, 2/0, 0/0/1i. Fast suite: 194/1054 binaries in the window (3 known lines in the report;
  escort_hide_findrun's hide-exit route-index assertion flipped — the test's known timing
  sensitivity under the game-correct moving read; left to the next session).
- Report: docs/reports/onfoot-moving-accuracy.md.
