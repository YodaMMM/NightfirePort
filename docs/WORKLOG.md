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
