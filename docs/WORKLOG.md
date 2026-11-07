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

- (03:25, 11-07) The arm shipped: the destroy leaf carries `smashed_hash` (the glist SIBLING
  +0x50 — the walker passes pCelList+idx·0x50+0x50 0x1d1b68:63,96, the dispatcher's move $t0,$t1
  0x1d04fc, Destroy_Create 0x12c318:25-30 with glist+0x40 == 0 [L ee_snapshot 4/4], the smash
  swaps it in 0x12bbb8-bc4); `Props::smash` sets the scratch after the channel write; the sim's
  `smash_stat_check` (drone.rs, all four smash paths) gates on 0x07000002 and logs
  PlrStat_LogShotHitScenery(0) [M 0x12bc44-48]. [D] the named hash 0x020001d9 = "Computer2",
  map 51 object 12, the sibling of pl[14]'s "Computer1". The pin: the real load + the real
  trigger — the computer's break nets the denominator one lower than the sister cable light's
  (delta presses−1 vs presses), red-proved with the log disabled. walkreal_07000002 + the stats
  binaries + nf-game next, then the suite.
