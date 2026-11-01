
## tripwires-shot-from-range — 2026-11-01

Task: on the tripwire levels (0700000a, 0700000b, 07000014) shooting a tripwire mine from range
does not set it off in the port. Find what the game does when a round hits a tripwire's
mine/emitter [M the tripbomb class's hit/damage handler, Bullet_CollisionHandler 0x12ce80's class
dispatch; D the tripwire placements' collision], port it with the address cited, and pin: a P2K
round from 15+ units at a mine detonates it (blast + beam gone), a round that misses does not.
Run the tripwire/trigger binaries you touch, every *0700000b* / *07000014* walk and nf-game;
quote the totals. Branch from origin/master 521d9bb2.

Progress (same day): ported the from-range detonation of the placed 0xFE tripbomb mines —
the mirrored bullet contact (`Collide_Update` 0x1ea398:160-234 lands the node on the mine's
obj+0xD0 with +0x58 = the bullet; `Mine_Update` 0x17c930:31-45 explodes on ANY +0x58 != 0),
the nearest-solid contest in `update_player_bullets` [M Collide_PickObj 0x1eae80's sphere pick +
Collide_Intersect 0x1ebaa8's one-sided ray]. Census (probe, committed): 0700000a/0b/14 have NO
0xFE placements in any of their 38/51/55 maps — the task's "tripwire" on those levels is the
PLACED laser tripbomb (the stuck class-5 grenade object), whose shot-detonation rule is
`Bullet_handle_object_destruction` 0x12cd98 (class 5 + record+0x68 & 0x4000 + record+0x70 &
0x80204 → delete + Explode_Create(record radius/damage, script 0x6000052) + the 0x80000 laser
bit deletes the beam object at BU+0x38), called from `Bullet_CollisionHandler` 0x12ce80's solid
object-hit arm. That leg is designed, not yet ported — next message continues.
Ran: fix_satchel_tripbomb_live 6/6, interactables_07000015_live 14/14,
interactables_07000041_live 14/14 + 2 ignored, nf-game --lib 593 passed / 0 failed.
