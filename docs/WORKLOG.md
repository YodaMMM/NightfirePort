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
