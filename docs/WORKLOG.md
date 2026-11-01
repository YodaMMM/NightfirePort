# WORKLOG

## point-blank-pile-hits (branch s/point-blank-pile-hits)

Task: on origin/s/enemy-stats-3's 07000003 full-route run, 48 player rounds fired at ~1 unit into four guards standing on one spot landed almost no hits. Find what the game does for a player round at that range [M Bullet_init 0x12c5a0 (start point vs muzzle/eye, near clip), Bullet_update 0x12e7b0 -> Collide_Pick 0x1eb320 / Collide_PickObj 0x1eae80 (bounding spheres, overlapping drones, first-hit rule), drone hit-boxes], reproduce with four guards held on one spot on 07000003 (or a synthetic pile), fix the port gap with the address cited, and pin: N rounds at 1 u into the pile land the game's hits (sister: at 5 u). Run the bullet/impact/drone-damage binaries, walkreal_07000003_live and nf-game; quote totals.

Session 1 (sandbox reset at start; fresh clone):
- Branched s/point-blank-pile-hits from origin/master (d07c579e). Sandbox expires ~10:14 UTC; stop new work 09:54 UTC.
- Setup in progress: downloads, rust toolchain, then background suite runner.
