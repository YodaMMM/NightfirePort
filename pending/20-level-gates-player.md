The game hard-codes per-level behaviour: 127 comparisons of `GameState._12_4_` against a level code in the decompile. One unported case was found by accident: jump refused on 07000007 before channel 0x48, now on master. Others are likely.

For every branch in your set, read it, trace what it does, and find the port's code. Report one line each: PORTED with the port location, or MISSING. A comment describing the branch does not count as ported; look for the code. Fix each MISSING one with a test on the real level, proved red on the old code. Do not rely on hand-built geometry.

Your set: every `Player_*`, `Bullet_*`, `Gun*`, `Weapon*`, menu and HUD function.

Start with these two, already confirmed missing:
1. `Player_Move` 0x191dd0:139-141: on 07000007 with channel 0x48 clear the move scale is x0.9. Read which axes it reaches. This changes how walking feels, so keep it in its own commit and say so; it will not merge without a hands-on check.
2. `Player_CollisionHandler` 0x1949a8:104-108: on 07000012, below y -10, channel 0x7c is set. Find what reads 0x7c.
