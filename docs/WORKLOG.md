# Worklog

## 986-find-opponent-exact

- (11:36 UTC, 10-24) Branch created from origin/master (26543436). Task: NDrone2_FindOpponent's team-2 scan — port the exact alive filter (+0x4F8 0x600 clear / 0x100 set, +0xAC health > 0, +0xFF != 0x11, +0xFE & 1 [M 0x1424e0:210-226]) over the port's present/enabled/corpse fold, and the shared per-frame LOS budget (DAT_00299092) over the port's one-LOS-per-drone. Sandbox reused (repo, work/, ISO intact).
