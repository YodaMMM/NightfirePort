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

Session 1 progress (~11:30 UTC):
- Two port gaps fixed: (1) turret rounds fly the game's Bullet_init cone (spread 4.0 [L
  ee_start_05.bin @0x2c55d0+0x24], 3 draws + GT_Update's post-shot frand(2pi) [M 0x12c5a0:166-184,
  0x180498:349/:522]) — drone.rs's two GTEvent::Fire consumers; (2) the trigger toucher takes the
  game's crouch pick arm (Player_Collision 0x194df0 case 3 via drone::Player::body; new
  drone::Player::crouched field) — the crouched body tops ~feet+1.26, under beam #607's 1.49-1.76
  band [D], while the standing 1.86 crosses.
- The real-damage walk un-ignored: crouches the #607 crossings; the low beams (#614/#621/the 0x25
  fence/#625 [probe tris]) are dodged at their ends. With the crouch alone the death moved from gun
  #640 to gun #639 (42x2.4, armed by #621 -> OR #668 -> ch 0x10) — the low-beam discovery. The walk
  still fails: the outbound A* crosses #614 (arms #640) and the band legs strand Bond short of the
  stand. nf-game 635/0/5i with both fixes. Remaining: the outbound nav via + fatal band legs; the
  other 0a binaries + suite totals (sandbox disk filled twice; background jobs killed).

## level-0a-ceiling-gun-route-2 (branch s/level-0a-ceiling-gun-route-2)

- start (the ceiling-gun-route tip 42227169 + master f3dd2eba merged, 2026-11-01 ~13:23 UTC).
  Task: finish the crouch-under-#607 real-damage route — the outbound nav via east of #614's east
  end, the fatal band legs, un-ignore walk_office_floor_qworm_keyboard_hatch_real_damage, pin the
  laser band rule, run every *0700000a* binary + nf-game.
- The laser band rule pinned live both #607 ways (standing raises 0x1F via the interactables'
  exact through-crossing leg; crouched passes silent with the crouch held clear of the beam's
  0.55 standing reach — the walker's 1.2-arrive swing east into the reach fired the beam, the
  new (−12, 14.5) crouched leg fixes it). The outbound #614 via (the A* west lane crossed the
  low line at x −22.1 — now via (−15.8, 10)) + the fatal band legs (walk_to). The low-beam live
  crossing pin pulled: the port's toucher reaches the low band only marginally on both stances
  (standing 0.47 off, crouched 0.03 inside — neither fired) — the dodge + the route's no-alert
  pin is the low-beam pin; live check filed. The branch's drone::Player::crouched compile break
  fixed (25 nf-game test files' initializers). The route's last bug diagnosed: the crouched
  outbound's slow pace lets the panel-side challenge civilian run and press (level_exit before
  the install; health 76, 10 hits — the turrets never armed, the beam pin held). Totals:
  interactables 7/0, walk 3/0, fix_0a_rain 2/0, walkreal 6/4/2i (the 3 route walks share the
  runner fail + the mouth-edge repro), nf-game 636/0/5i. Report:
  docs/reports/level-0a-ceiling-gun-route-2.md.
