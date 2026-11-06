
## level-gates-player
Started 2026-11-06 14:32 UTC. Branch s/level-gates-player (from origin/master 38fd3151e). Task:
the per-level gate census — every branch that compares GameState._12_4_ against a level code in
the Player_*/Bullet_*/Gun*/Weapon*/menu/HUD set, reported PORTED (with the port location) or
MISSING; fix each MISSING with a real-level test proved red on the old code. First two
confirmed missing: Player_Move 0x191dd0:139-141 (07000007, ch 0x48 clear, move scale x0.9 —
own commit, needs a hands-on check) and Player_CollisionHandler 0x1949a8:104-108 (07000012
below y -10, airborne, sets ch 0x7c — the Phoenix Fire atrium fall plane, needs a real ledge
fall).
level-gates-player OUTCOME (2026-11-06 ~15:53 UTC): the census — 14 GameState._12_4_ sites in
the Player/Bullet/Gun/Weapon/menu/HUD set: 11 PORTED (cited per site in the report), 2 fixed
this window, 1 (MenuManager_Monitor 0x21d508:47) MISSING with its guarded block itself unported
(controller-hotplug/attract — nothing to gate). FIX 1 (own commit): Player_Move 0x191dd0:139-141
— the 07000007/ch-0x48 party slow-walk, the mult scales ×0.9 into BOTH horizontal wish lanes
(BL+0x10/0x18, walk and swim), the turn and the +0x20/+0x28 smoothers unscaled [M disasm
0x19220c/0x192230/0x192250]; party_slow beside jump_locked (scene.rs), k ×0.9 in physics_tick;
test party_move_scale_07000007_live — red proven (got 0.1 vs 0.09), walkreal 07 whole 6/0 WITH
the scale; NEEDS THE HANDS-ON CHECK before merge. FIX 2 (own commit):
Player_CollisionHandler 0x1949a8:97-108 — airborne (BL+0x940==0) below y −10 on 07000012 latches
switch_channels[0x7C]=1 every tick (0x194c44), no damage/velocity gate; the backstop fires
BEFORE the plunge plates #788-791 (y −10.51) and chains SP #767's death restart; void_fall flag
+ the Sim's level-gated Channels::set; nf-game lib 685/0, walkreal 12 whole 3/0. LEFT: the
level-12 real-fall red/green test (needs the objective-1 chain to the shaft; the route spec is
in the report) and the live checks (the party feel, the shaft-fall watch @0x26fd0c). Report:
docs/reports/level-gates-player.md.
