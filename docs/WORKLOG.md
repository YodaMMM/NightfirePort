# WORKLOG

## level-16-stuck-overlook (branch s/level-16-stuck-overlook)

- start (master 521d9bb2). Task: on 07000016 at Bond (-17.86, 6.81, -97.16) facing yaw 6.98
  (the walkway beside the big control-room window, around tick 3700 of a real run) the
  player cannot continue the level, and the held rifle's view model visibly clips into the
  glass/wall. Find what the game expects here [D the route on from this walkway: doors,
  triggers, the channel that opens the way; M the classes], reproduce on real input from
  the level start with the port's walker, fix any port gap with the address cited, and pin
  a walk that passes this spot. Also check the view model's near-clip / wall push-back at
  that spot against the game [M the view-model draw path]. Run walkreal_07000016_live,
  interactables_07000016_live and nf-game; quote the totals.
- The stuck spot decoded: (−17.86, 6.81, −97.16) is the SW dead end of the trench hall's upper
  WEST walkway (deck y 5.7..6.8 [D §7]), ~3.5 m SW of the SecurityStation trigger #597
  (kind 0xEA TOUCH_ONCE, out 0x24, mask 0x04) at (−15.20, 6.52, −94.56) [D]. What the game
  expects there: walk back NE to the station and press E — ch 0x24 rises [M Trigger_Activate
  0x1b2318:38-49 gates the E dispatch on mask&4] and door[0] "Door_1" (1.84, −0.10, −114.23,
  lock 0x24 [D]) unlocks AND opens on the channel's rising edge [M Door_Update 0x1363c8, the
  lock arm ~0x13660c → Door_Activate 0x135be0]. The way on from the overlook IS the station.
- Pinned the arc on real input from the level start:
  `walkreal_07000016_live::the_overlook_dead_end_returns_to_the_station_and_door0_opens` —
  the reported dead-end spot reached grounded (−17.50, 6.84, −96.37 on the raised end), the
  return NE, the station E (ch 0x24, door[0] lock cleared), down and through door[0]. NO port
  gap on the route: the interactive blocker was route knowledge (the station is the way on —
  the walkreal route already visits it; this pins the dead end too). Binary totals on the
  final tree: walkreal_07000016_live 10 passed / 0 failed; interactables_07000016_live
  8 passed / 0 failed; nf-game 964 passed / 0 failed / 9 ignored (97 binaries).
- The view-model check at that spot [M the view-model draw path]: the game has NO wall
  push-back — the gun path (Player_PositionGun 0x1a3b40, Player_SetWeaponAnimObj 0x1a2e38,
  Player_GetHeadPos 0x195aa0) never touches Collide_*; the anti-clip is draw-order: the gun's
  flags word 0x2000 sends it to draw list 2 [M View_AddObjects 0x1e62d8:57-60] and Game_Draw
  0x1e6f50:157-163 runs psiClearZ 0x10b9a0 (a fullscreen write-Z-only sprite) before list 2 —
  the world (glass included) can never occlude it; the keep-out is the 0.55 player capsule
  [M Player_Collision 0x194df0]. The port already mirrors the depth-clear (gpu.rs list-2 VMZ
  trick, scene.rs VIEW_MODEL_NODE → pass 7). Two faithful gaps closed: the delta-repeater
  bolts were the one fp attachment on the default node (world-occludable) — now VIEW_MODEL_NODE;
  the camera near plane 0.05 → the game's Ps2Near2 = 0.1 [L ee_start_05.bin,
  docs/systems/render.md — PS2WorldViewVU1Kick2 0x10b2b0's depth mapping]. Both need the main
  agent's render check (GPU-less sandbox); render spots in the report.
- Suite: the task's nohup background start is killed by the sandbox between tool calls
  (verified twice), so the fast runner's build+run is executed by an untracked 2-worker
  foreground chunk runner (tools/test/chunk_par_tmp.py, resume-state across invocations) over
  the same 1013 test binaries; totals quoted in the report. Suite state at the window's end:
  126/1013 binaries run, 426 passed / 0 failed / 26 ignored; the 2 CRASH entries
  (walkreal_07000014_live, interact_0700004a_live) both pass solo — the 4 GB sandbox OOMs
  concurrent heavy walkreal pairs; one chunk ran without the per-shell env exports and its
  489 ISO-test failures were purged (NF_ISO is lost between tool-call shells here).
- origin/master moved 521d9bb2 → babf9247 mid-session (140 files, walker edits among them);
  NOT merged — the merge would invalidate every quoted run with no window to re-validate.
  First step for the next continuation: merge origin/master, rebuild, re-run
  walkreal_07000016_live / interactables_07000016_live / nf-game and the remaining suite tail.
- level-16-chaser-real-damage (s/level-16-chaser-real-damage, 2026-11-01): started from
  origin/master 43752669. Goal: make the 07000016 chaser fight survivable on real health via
  walker tactics (return fire with the carried Delta Repeater, range keeping, cover, the pinned
  blast-room shelters — never game code without a cited difference) and add the real-damage
  variant of walk_the_chasers_defeat_completes_the_mission. Chaser punch is 50 raw / 20 a hit [M].
- level-16-chaser-real-damage (s/level-16-chaser-real-damage, 2026-11-01): the chaser fight on
  real health. The melee chain pinned: skin 0x0500001e → kind 0xb, ANIM SET 7, fists [M
  0x14b300's switch; the placement's weapon (param 9 = 0x12 [D]) does not replace the set — the
  0x12/0x14/0x15 arm guards on kind != 0xb] → DroneWeap_CloseCombatImpact 0x1789c0:26-40 takes
  the kick arm (record 99, the 2.0 band, no facing gate); record 99 damage = 50.0 [D +0x0C,
  chaser_melee_records.rs pins the bytes], records 100/0x65/0x66 = 15.0 [D]. Player side:
  Player_HandlePain 0x1902c8 × damage_factor(0x16, diff) — the 14–16 Plr_DMod row 0.4/0.6/1.0
  [M 0x1d4424] → the punch is 20.0 health a hit at Operative, 30.0 at Agent. The star (weapon
  0x12, record 1.5 [D]) measured through the real path: 0.6 ranged / 1.2 point-blank (= ×2.0
  DroneFiring_TooClose [M 0x139fac-0x139fc8]) at Operative — the_chasers_punch_lands_the_
  record_damage pins every landed hit to the chain. NO game code changed.
- The real-damage variant (walk_the_chasers_defeat_completes_the_mission_on_real_damage, with_
  real_damage): the route survived on return fire alone (8 fights, 22 health taken), both
  shelters held (sit_out_pulse zero-contact), the chaser fell to the range-keeping kill (+3364
  — hold her outside 3.0 [M 0x1789c0 the 2.0 melee band; 0x179740:113-115 under-3.0 accuracy]),
  pulse 2 held — Bond died only in the pit-squad phase at ~48 health (the 4-squad + pit shooter
  #8 alive at the room's door). The test is #[ignore]d with that reason; the squad-phase cover
  tactics are the next leg. The pit floor's placed soldiers (#6/#7/#8, skins 0x93/0x87/0x90,
  weapon 0x14) shoot the fight stand — the allowance walks top through them (the header's
  "damage is suppressed"); clear_the_pit_shooters is the explicit return-fire answer.
- HER POINT-BLANK BEHAVIOUR (live check): in both runs she parked at d 1.1-1.5 cycling 0xaa/0xb2
  star bursts every 6 ticks and never reached the sword 0xac, although 0xaa's UPDATE routes ≤2 →
  0xa7 [M 0x16c650] and the close band rolls 0xac at ~50%/cycle [M 0x16bb50]. Either the port's
  close-band roll distribution differs or her tracked +0x1A0 dist lags — the main agent's live
  PCSX2 run should log her state sequence at melee range (0xaa/0xb2/0xac transitions, +0x1A0).
- Totals on the final tree: walkreal_07000016_live 13 passed / 0 failed / 1 ignored (57s);
  interactables_07000016_live 8 passed / 0 failed (10s). nf-game was NOT run in-window (the
  build + the fight iterations consumed it — the sandbox rebuild from scratch took ~90 min):
  the next continuation runs it before pushing further work. Suite: not run (the nohup start
  is killed by the sandbox between tool calls — the known issue; the disk also filled twice
  during the 1013-binary build: stripped executables + staged resumes got nf-game/formats/
  audio/nf-view ~85% built before the window closed).
