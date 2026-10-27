# WORKLOG

## level-13-guards-start-alerted-2

Branch s/level-13-guards-start-alerted-2 (from origin/s/level-13-guards-start-alerted @ e52fac44, merged origin/master @ 08f06953). Task: the prior two AI fixes (vision-bit sight gate [M 0x176a28], EnemyRunToPoint Attack resume [M 0x168910:22-36]) turned eight tests red on 07000001/4a within seconds of level start — find what the game does at start on those levels (which states set the vision bit on 01's starting guards and the escort; whether a test leaned on the bypass), fix the port or re-derive the test with the cite, get all eight green plus the 13 start test.

## level-13-guards-start-alerted-3

Branch s/level-13-guards-start-alerted-3 (from origin/s/level-13-guards-start-alerted-2 @ 1cc9ea59, merged origin/master @ fee3d546). Task: round 2 added the vision bit to PlayScript and the abseil ENTERs (8 of the reds green) but 6 remain (escort_follow_07000001_live, walkreal_07000001_live, fix_npc_doors_live, walkreal_07000008_live, fix_pickups_ammo_live, drone_weapon_drop_live). Round 2's census: 139 state functions write `+0x4F8 |= 0x10000` on ENTER [M]; this round ports the bit on EVERY one of those ENTERs (ally/escort, combat, cover, death/drop paths), not just the ones a red test exposes, and gets all six green plus the earlier eight and the 13 start test.
