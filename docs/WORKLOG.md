## explode-debris-spawn

- (14:50 UTC, 10-26) Branch created from origin/master (d4f15b9e), pushed. Task: give each
  explosion a persistent object (age/kind/script) ticked at the game's object-update position
  and wire the debris arm — game chain: Explode_Update 0x17c350 spawns debris for script-0x0600052
  blasts in the first quarter of lifetime, draws Rand_Rand(5) and on GameState_time % (r+10) == 0
  calls Debris_CreateEx 0x1ac808 (MetalHash_132) for kinds 5/6, stone 8/10/0xB only on 0x0700008
  with ch 0x53 [M 0x17c350:31-67]. Port has create_debris (explode.rs) with the exact RNG draws
  but no production caller; the explosion is a per-frame queue entry (drone.rs) with no per-tick
  state. Tests: debris after a mine/barrel blast on 07000015 with the game's kinds and counts [M];
  none for non-0x0600052 blasts. Pin Rand_Rand draw order vs tests/calls/Rand_Rand.calls first.

- (15:56 UTC, 10-26) explode-debris-spawn DONE. Persistent blast objects (`Drones::blasts`,
  `blast_debris`) ticked newest-first at the walk head [M 0x1336f8 + 0x134698]; the
  `Explode_Update` debris arm over `debris_arm` (all gates [M 0x17c350:21-67], MetalHash_132/
  StoneHash_133 [L 0x2a3718/0x2a3728]); debris kinds plumbed at every source — mines 0
  [M 0x17c930:128], Destroy_Smash 0 [M 0x12bcf8], fuse 0 [M 0x12ce14], flesh 0 [M 0x1ec180],
  impact-explode = hit material & 0x3F [M 0x12d084-0x12d0bc], satchel timer 0 [S residue].
  FIXED create_debris spin constant → 0x3E71463B. Task-premise correction: mine/barrel blasts
  carry kind 0 → the game spawns NO debris for them (report §premise). Tests: nf-game lib 561/0
  (6 new), interactables_07000015_live 12/12 (new persistent-blast pin), fix_blast_scripts_live
  3/3, diff_calls 6/6; all 4 crates build. Report: docs/reports/explode-debris-spawn.md.
