
## tripwires-shot-from-range — 2026-11-01

Task: on the tripwire levels (0700000a, 0700000b, 07000014) shooting a tripwire mine from range
does not set it off in the port. Find what the game does when a round hits a tripwire's
mine/emitter [M the tripbomb class's hit/damage handler, Bullet_CollisionHandler 0x12ce80's class
dispatch; D the tripwire placements' collision], port it with the address cited, and pin: a P2K
round from 15+ units at a mine detonates it (blast + beam gone), a round that misses does not.
Run the tripwire/trigger binaries you touch, every *0700000b* / *07000014* walk and nf-game;
quote the totals. Branch from origin/master 521d9bb2.
