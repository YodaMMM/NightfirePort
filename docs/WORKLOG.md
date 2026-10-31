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
