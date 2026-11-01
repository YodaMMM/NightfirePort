# WORKLOG

## level-0a-ceiling-gun-route (branch s/level-0a-ceiling-gun-route)

- start (master 721b88ac, 2026-11-01 ~10:12 UTC). Task: on 0700000a laser #607 feeds OR #667 -> ch 0x0F ->
  ceiling gun #640 enable (-22.1, 2.3, 17.2); since GT_Update case-3 fix [M 0x180498] the gun re-arms and two
  bursts (27+15 rounds x 2.4) outlast Bond's health; only explosions damage a turret (GT_ApplyDamage 0x1816d8
  <- Explode_Propagate 0x17c518) and 0a has none. Find how a player gets through (avoid #607: crouch/step-over
  vs Player_Collision 0x194df0's 1.86/1.3 body, route around, gun cone/LOS, Bullet_init 0x12c5a0 spread for
  owner -1 rounds — port fires dead-on). Fix the gap, make walkreal_0700000a_live::walk_office_floor_qworm_
  keyboard_hatch_real_damage take the route and un-ignore it. Run every *0700000a* binary + nf-game.
- Setup reused from prior session (repo, work/, ISO, nf-queue, toolchain all present).
