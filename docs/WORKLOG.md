## smash-stat-gate-02 (2026-11-07, UTC)

Branch `s/smash-stat-gate-02` cut from `origin/master` (22d5836b7). Task: `Destroy_Smash`
0x12bb60:42-55 calls `PlrStat_LogShotHitScenery(0)` on 0x07000002 when a switch-channeled
smashable is destroyed and its obj+0xd8 equals `hashtable_getitem(0x20001d9)` — the port has
no such arm (the level-gates-world report records it open: destroy leaves carry no object-name
hash). Read `PlrStat_LogShotHitScenery` 0x1998d8 + the port's progress.rs, find the placed
object with that name on 07000002 [D], carry just enough identity to the destroy path (no
level-wide name table if one field will do), run the arm only on that level/object and only
after the channel write, pin through the real path (Bond shoots the object; the results-data
Accuracy matches the game's arithmetic; another smashable changes nothing), then run
walkreal_07000002_live, the stats binaries and nf-game.
