# WORKLOG

## level-16-finale-pit-3 (branch s/level-16-finale-pit-3)

- start (master c32c53c6). Task: 07000016's finale must complete on real input — the ch-0xf3
  hurt-volume window of script 0x0600090a stream 6 kills Bond in the port; find how the shipped
  game gets the player through [D the script streams, the hurt volume's gates, the items/vehicle
  the finale hands the player; M the trigger/script classes involved and the player state they
  set]; fix the port gap with that evidence; un-ignore
  `walkreal_07000016_live::walk_the_chasers_defeat_completes_the_mission` so the mission
  completes on real input. Read blocked/10-level-16-jetpack-escape-2.md's whole report first.
- outcome (2026-10-31, round 1): the finale mechanism is now DATA-DIRECT. The mission completes
  through the KikoDeathButton #693 (kind 0x29, p0 = ch 0x90, p5 = ch 0x8d the gate [D];
  Switch_Activate 0x1b02a8:22 inhibits the switch while 0x8d is on) → mux #618/#271 (AND, out
  ch 100 ← 144 [D map63 pl271]) → MissionState 6 + the driving hand-off 0x07300017 [M
  0x182c60:111-112] — pinned by interactables_07000016_live 8/0. The WALK cannot press it yet:
  the corridor touch #616 (0xEA out 141 at 2.45,0.84,−145.62 [D]) arms the inhibit on every
  route walk, and the re-arm is the pack cinema 06000632 stream 16 ch-0x8d=0 at its t420 [D
  07900016.bin] — the completion ride goes through the launch-chain cinemas (06000631 750t re-places
  Bond at the pit floor at its t747 [D]; 06000632 1060t) — the port's cinema chain + the chaser
  shoot (out of ammo at +61) are the remaining gaps; the walkreal tail test stays ignored with
  the sharpened citation (walkreal_07000016_live 8 passed/0 failed/1 ignored, 38.89 s). New [D]
  timeline: the launch 0600090a pulse 1 = ch 0xf3 1611..2170, ch 0xf4 @1610 arms the ported
  Apocalypse (world_fx.rs; the doc's row was right — my round-1 "unported" read was wrong),
  ch 0x50 (the lift) @2240, pulse 2 6810..7320. Report: docs/reports/level-16-finale-pit-3.md.

## level-16-finale-pit-4 (branch s/level-16-finale-pit-4)

- start (master b44ac857; merged s/level-16-finale-pit-3 2ed06a3a for its mechanism chain and
  report). Task: continue pit-3 — the walk must press the re-armed KikoDeathButton through the
  launch chain and complete the mission on real input; un-ignore the tail walk.
- outcome (2026-10-31, round 2): the survival-window mechanism is [M]-COMPLETE and the
  contradiction is sharpened to ONE link. NEW [M]: the who-mask is a START gate only
  (Hurt_Update 0x17e5d0:51-104 — gated → uVar5 = the channel byte, 1 != 0 → state 1; no
  per-victim filter); the victim pass is ungated (Player_DealWithObjHit 0x194458:95-102;
  Player_CollisionHandler 0x1949a8:24 gates only on the enabled byte); the collide candidate
  law ACCEPTS the volume while the gate is high (Collide_PickObj 0x1eae80:82-90 — mode 0x100
  short-circuits the +0xFA clause; +0xF0 0x20 gate-low reject clears on the rise; Hurt_Create
  0x17e870 sets no other skip bit) — the shipped game REALLY deals the 1000 damage on contact.
  NEW [M] hand-back chain: event 6 snapshots record(i−1) (0x1d94c0:73-84; Script_Update
  0x1da998:29-31 increments +0xab3 AFTER Script_Run → record(0x80+i·0x28) = the previous
  stream); 06000631's stream 8 Entity (t747, hash 0, obj_type 31 → Script_CreateEntity
  0x1d8968's model-less generic tail) sets kind 1 and poses the dummy AT CREATE
  (Script_EntityStart 0x1d8d78:111 calls Script_Interp) from key 204 = (3.57,−22.76,−179.55)
  [D]; Script_Free 0x1da880:44-48 → Player_Enable 0x195498:54-68 teleports Bond there — the
  shipped Bond re-enables INSIDE the volume box 1611 t before the pulse (door 9 t after it
  drops, lift 70 after — [D]). NEW [D]: ch 0xc0's writer = 0600090a stream 20 @t7840; 07900016.bin
  holds ONLY 06000632 (its [8d,0] @420 is the only 0x8d-clear in the packs) — the button-first
  route is dead. OPEN LINK (the report's live checks): Script_Interp 0x1d9a08 mode-0 for a
  one-key track — the create-time interp either applies key 204 (pit-floor hand-back, the
  port's reading) or leaves the create pose (the ORIGIN). Port code: NONE changed — every
  stage the port implements matches the decompile; the walkreal's ignored citation now carries
  the full chain. walkreal_07000016_live 8/0/1-ignored (42.10 s), interactables_07000016_live
  8/0 (42.98 s), nf-game 588/0/5. Report: docs/reports/level-16-finale-pit-4.md.
