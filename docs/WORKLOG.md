## mine-shield-by-guard (2026-11-01, s/mine-shield-by-guard)

Branch from origin/master (d43c28ce). Task: on origin/master the player-round pass tests 0xFE
mines (Mine::segment_t) before drones, so a guard standing between Bond and a mine does not block
the round as in the game, where the round takes the NEAREST hit along its path [M Bullet_update
0x12e7b0 -> Collide_Pick 0x1eb320 / Collide_PickObj 0x1eae80, the first-hit rule;
Bullet_CollisionHandler 0x12ce80]. Make mines, drones, stuck tripbombs and world geometry compete
by hit distance in update_player_bullets (crates/nf-game/src/drone.rs), and pin: a guard in front
shields the mine (sister: step the guard aside and the same round detonates it). Run the
mine/tripbomb tests, every *07000015* binary, ai_live and nf-game; quote the totals.
