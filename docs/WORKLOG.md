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

## guard-push-every-tick (branch s/guard-push-every-tick)

- start (master a69712bd, 2026-11-01 ~14:17 UTC). Task: on 07000003 the two-guard courtyard flank
  builds an overlap the port only discharges airborne — port the game's every-tick drone-player
  push (Collide_Jointy 0x1ec180, no ground gate) behind NDrone2_DoCollision 0x149ec8's gates
  (+0x4f8 bits 0x20/0x200, +0x570 bit 2), un-ignore the flank repro, keep walkreal_07000003 x2 /
  walkreal_07000001 the_plate_refuses_bond_alone / exit_aim_01 as-is.
- The gate read settled: Collide_Jointy 0x1ec180's 0x800 arm has no ground gate and DoCollision
  0x149ec8's per-bot arms (0x200 dead OFF; 0x20 special-physics from DefaultInit 0x801e2,
  cleared by Enable(0); then 0xd8 0xc/0x13, +0x538, near-player < 2.0 [0x14b190], obj+0xFC,
  +0x570&2) keep the capsule ON for every bot the push can touch (0.95 << 2.0) — the airborne
  only gate was level-14-sleeper-push's [S] port choice and is lifted. The flank fall is FIXED
  (the test un-ignored, 2/2 green with the lip sister's pin re-scoped off-push). The 03 walks
  pass 4/4 unchanged; on 01 the meet chain's activated hall guards (SP #276 → ch 0xE, state
  0x68) really block — the walkers now clear them (walker.rs clear_pushing_guards[_lane],
  opt-in): exit_aim_01 1/1, walkreal_07000001 4/6 (the_plate sister 15 + the real-damage escort
  still wedge mid-leg — the next pass's item). MP digests structurally untouched (empty
  bot_capsules on both MP paths) but unrun. Report: docs/reports/guard-push-every-tick.md.

## guard-push-every-tick-2 (branch s/guard-push-every-tick-2)

- start (s/guard-push-every-tick 76172aff merged with master 71b7b6b9 → 7c6ac4ce, 2026-11-01
  ~16:08 UTC). Task: the every-tick drone-player push landed on master red 20 binaries —
  walkreal 01 (x3 + real_damage timeout), 02 (timeout), 07 (x2), 0c (x2), 11 lower floor;
  escort_follow_07000001; exit_aim_01 (timeout); interactables_07000001 camera sweep,
  interactables_07000003 breakables; cutscene_timer_pause; party_door_lock_07000007 (x2);
  speech_facial_fallback; guest_faces; probe_07_route. Many are non-walk: check which drones
  the Collide_Jointy 0x1ec180 / NDrone2_DoCollision 0x149ec8 gates admit (allies, party
  guests, scripted/cut-scene actors, non-combat NPCs — voice +0xd8, +0x538, obj+0xFC,
  +0x570&2), cut-scene / disabled-player windows, and push magnitude/direction vs the game.
  Fix the port where it differs; fix walks only where fragile.
- Diagnosis pass (16:10-17:30 UTC): reproduced cutscene_timer_pause on the merged tree — the
  walker wedges at the hall door against drone #22 (team 3, CIVILIAN_MISSION 0x16) parked
  0.88 u off Bond's spine (pushdbg: equilibrium, push ~0.065/tick out of the doorway). Researched
  the gates with two decompile sweeps: the near-arm distances live at Drone+0x154..0x160 f32[4]
  (PreDroneControl 0x148db0:35-50 rewrites them EVERY tick for every live NPC, 100000.0 sentinel
  for empty slots — no stale-zero hazard; the decompile's `param_1 + 0x154` is a byte offset,
  raw MIPS 0x14b19c `addiu $a0,$a0,0x154`); `+0x570` is the current danim's Drone_AnimInfo flags
  word (bit 2 = in-place anim, set on danim 0x11A the universal start idle — every drone spawns
  with it, moving clears it; writers only DroneAnim_SetDAnimInternal 0x13aee8:50 and
  DroneAnim_SetScript 0x13e088:26); `+0x538` is the blend-out script pointer (SetHTAnim
  0x13b1c8:76); `+0xd8` is the VOICE SET not a state word (DefaultInit skin switch 0x14b300:201/
  :232: 0xc = Zoe-class VIP, 0x13 = Drake — always-ON arms). Re-derived DoCollision 0x149ec8 for
  a live drone: 0x20 set (DefaultInit 0x801e2 / Enable(1) 0x801a0 restores, Enable(0) 0xffe7fe5f
  clears) + not dead → ON if voice 0xc/0x13, mid-blend, near < 2.0, or drawn (obj+0xFC draw
  counter, View_AddObjects 0x1e62d8); the +0x570&2 OFF arm only bites far+off-screen. So every
  bot the push can touch is ON in the game — the port's admission (enabled && !corpse && < 2.0)
  is FAITHFUL, and the push math matches Collide_Jointy 0x1ec180:253-266 (normalized
  (p−q)/dist, overlap = r-sum − dist, fVar12 1.0 SP vs class-2). VERDICT: the port does not
  differ; the 20 reds are the harness fragility the driver's walk-fix carve-out covers (walks
  wedged on bodies they cannot pass: the walker's stuck ladder sidesteps blind and its
  clear_pushing_guards only shoots team 1; the facial harnesses pick the nearest guest whose
  talk script need not be present once the push shifts Bond's intro-end position). Next: a
  direction-aware push-dodge arm in the stuck ladder (safe-by-default: fires only on a live
  body within reach of a stalled leg), then the facial harness pick.
- Harness pass (17:30-17:55 UTC): walker.rs push-dodge arm — a live body (any team; enabled &&
  present && !corpse && health > 0, |dy| < 2.5, d < 1.5) inside a stalled leg's stuck ladder now
  triggers a DIAGONAL dodge (strafe away-from-body side, forward 0.45) that runs until the body
  is behind the run or 2.2 clear (1200-tick cap); same-body re-stall flips the side (the away
  side can be the walled one — the 0a civilian's push carries Bond into the south wall, the room
  is north). strafe +1 = LEFT = (cos yaw, −sin yaw) [M Player_Update 0x193600's move rows]. The
  0a hall-door civilian STILL holds the leg after 6 dodge cycles (d oscillates 0.95↔1.43, the
  lane feeds back into his spot) — the dodge persists but the geometry needs a wider swing or a
  nav-level avoidance; left for the next pass with the failing dump in the report. NOT validated
  against the green walks (no suite run — sandbox reaps background jobs; chunked runner queued
  for the next window).
- Final pass (17:37-17:39 UTC): the dodge re-acquires the nearest live presser every tick (the
  drones list reshuffles; a pinned body index ended the dodge while the same civilian stood
  there). The 0a hall-door leg still wedges after 6 dodge cycles — the doorway geometry needs
  nav-level body avoidance (the next pass's item, with the report's dump). FINISH: branch pushed
  through 1e5ab960 + this commit; report docs/reports/guard-push-every-tick-2.md written (with
  the live checks and the ntfy-failed record); finish.sh run `blocked` (the task's red set is
  not green and the 2-hour window closes at ~18:05 UTC).

## guard-push-every-tick-3 (started 2026-11-04 11:05 UTC)

Branch s/guard-push-every-tick-3 cut from origin/s/guard-push-every-tick-2 (c9a93c66), origin/master
(f9f6544c) merged. Continue blocked/10-guard-push-every-tick-2: the every-tick actor push is faithful
[M 0x1ec180:253-266 + 0x149ec8's gates]; the walker push-dodge is implemented but doesn't clear — the
0a hall-door civilian holds the leg after 6 dodge cycles (d swings 0.95..1.43). This pass: widen the
swing (dodge until d > 2.6, biased toward the walkable side) or make the walkers' nav legs cost live
bodies — test-side walker changes, opt-in per level. Then re-run every red binary (walkreal
01/02/07/0c/11, escort_follow_07000001, exit_aim_01, interactables_07000001/03, party_door_lock_07000007,
speech_facial_fallback, guest_faces, probe_07_route, cutscene_timer_pause) and fix each; no assertion
weakened; chunk_suite.py for the full run; nf-game + every walkreal_*_live.

Round (2026-11-04 ~12:18 UTC): the dodge widened and made opt-in. walker.rs: the release 2.2 → 2.6,
the every-10-tick geometry re-read (the away-from-body side first, then the only open side, then the
YIELD — backward off a both-sides-walled doorway squeeze), the pure-lateral gate while the body still
presses, the scatter-shot escalation after a full 1200-tick cycle (the game's own CIVILIAN_MISSION
bullet/sound scatter [M 0x15d860]); the arm + the whole dodge behind the new opt-in `push_dodge` flag
(rule 7), set in cutscene_timer_pause, walkreal 02/07, escort_follow_01, exit_aim_01,
interactables_01, party_door_lock_07, probe_07_route. nav_route prefers routes whose nodes sit
≥ 1.2 u from live bodies (the report's "nav legs cost live bodies", test-side). Green:
escort_follow_07000001 1/0 (was red). Still red on the 07 hall-door civilian #22 (team 3
CIVILIAN_MISSION hp 10, STATIC in the port, the only doorway crossing): cutscene_timer_pause,
probe_07_route, party_door_lock_07000007 (×2) — the kill fired the level exit (the fail state), the
offset miss-shot does not scatter (the scare folds prop 0x3c && seen && bond_armed [M 0x1787c8/0x178768]
— the live check names it). interactables_07000001 7/3 (improved; the camera-latch and redlamp-tip
asserts shifted with the push). exit_aim_01 / walkreal_02 / walkreal_07: the window ended mid-run
(timeouts). 0c/11/facial: not run.
