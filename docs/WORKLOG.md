
## level-gates-player
Started 2026-11-06 14:32 UTC. Branch s/level-gates-player (from origin/master 38fd3151e). Task:
the per-level gate census — every branch that compares GameState._12_4_ against a level code in
the Player_*/Bullet_*/Gun*/Weapon*/menu/HUD set, reported PORTED (with the port location) or
MISSING; fix each MISSING with a real-level test proved red on the old code. First two
confirmed missing: Player_Move 0x191dd0:139-141 (07000007, ch 0x48 clear, move scale x0.9 —
own commit, needs a hands-on check) and Player_CollisionHandler 0x1949a8:104-108 (07000012
below y -10, airborne, sets ch 0x7c — the Phoenix Fire atrium fall plane, needs a real ledge
fall).
