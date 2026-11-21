# s/pickup-pushout-walks-2

Shared random stream: this branch changes test scripts only. It sits on s/pickup-pushout-walks,
whose base (the dropped-gun push-out, `Pickup_Create` 0x18dda0:166-183) moves the stream; that
change is the base branch's. No game code, no draw code.

## The red

walkreal_07000015_live `walk_spawn_to_the_omega_sector_exit_real_damage_00agent` on
s/pickup-pushout-walks 077faf779: Bond dies at t15214 at (15.43, 41.41, 71.65), −55.08 from the
west-room lob's own frag. Re-run on this lane before any change: the same death.

## What the level offers (counted)

`Level::parse` over 07000015's maps (the 0xF0 placements, all on map54) and the d3 load's live
lists (`zz` probe, not committed):

| what | where | gate |
|---|---|---|
| BodyArmourHeavy 100 | (12.18, 40.35, 0.18) | room between door[35] (ch_lock 0xc8, no writer) and door[37] (ch_lock 0xf7, keypad #765) |
| BodyArmourHeavy 100 | (22.53, 40.35, 35.53) | behind door[43] (ch_lock 0xc8, no writer) |
| 0x1a ammo 30 | (2.18, 40.32, −9.70) | exit hall, after the last fight |
| 0x1a ammo 30 | (16.24, 40.32, −1.36) | the same x 10.4..25.1 room as the first vest (by position) |
| 0x1a ammo 30 | (58.36, 40.32, 72.02) | keypad room behind door[33] (ch_lock 0xf9, keypad #711) |
| frag 0x34 ×5, 1 round each | (57.6..58.4, 40.4, 70.3..71.0) | the same keypad room |
| bonus kind 6 ×2 | (−1.15, −7.81), (53.40, 71.14) | gold-medal gated, not created on a fresh load |

Guards at d3: 20 (0x15 ×7, 0x1a ×13; #0/#1 held by spawner 388). The keypad room holds no guard.
Doors the walk can open: the plain kind-2 doors (ch_lock 0) by E; the three keypad doors
(0x95, 0xf7, 0xf9) through their keypads; the six 0xc8 doors never. door[33]'s #711 keypad sits
2.6 u off the walk's nav lane top, so the keypad room is the one cache on the route.

## The new plan (00 Agent real damage only, all d3-gated)

1. **keypad_cache** at the nav lane top: Q-Decryptor drawn by the real G cycle, the keypad code
   entered through the keypad page (`Keypad::arm` + digits, the tower2 / interactables_07000015
   idiom), ch 0xf9 opens door[33], the box and the frags are walked over, back to the lane.
   Measured: 0x1a carried 20 → 50, frags 6 → 9 (two frag pickups stay on the floor), no hit.
2. **West room one grenadier at a time** (replaces the pokes, the frag lobs and the solo line).
   The eye-line scan over the ring walkway (from the eye 1.83 over the floor to each spawn head)
   gives each guard his own spot: (5.0, 71.25) sees #11 only, (6.0, 66.0) #10 only, (3.0, 65.0)
   #10 and #18. The reacting walk (`walk_react`) answers any guard the gun rays reach with the
   head-first aimed stand. Measured: #11 t10319, #18 t10526, #10 t10720, no hit, health 58.6.
3. **door[50]**: the 300-tick still wait is a reacting hold (the still wait took 33.5 from #4
   and #19 through the doorway); the old frag lob at #5 from (0, 44) is replaced by a reacting
   walk to (0, 44) and a 360-tick hold (the lob's frag came back through the doorway and popped
   at (−0.4, 59.5) for −46.46 on the retreat spot). Measured: #4, #19, #5 dead, no hit.
4. **door[51] hold**: the 0x1a in hand and the head-first aimed stand on the guard in sight; the
   hold runs on (to 2400 ticks) while a guard stands within 12 u. With the P2K the stand on #13
   landed 2.1-2.8 body rounds every other tap, went dry at hp 5 and his bursts killed Bond;
   with the 0x1a #14, #12, #13 died to one round each.

Result: green, health 58.6 from the silo crossing to the exit (the opening crossing still costs
41.0). The ledger is in the 00 Agent test's doc comment and at each changed block.

## Tests run

All with `CARGO_TARGET_DIR=target-o cargo test --offline --release -p nf-view --features
nf-view/movies,nf-formats/movies`, NF_SILENT=1:

- walkreal_07000015_live `..._real_damage_00agent`: red on the base (t15214, re-run on this
  lane before any change), green on this branch (health 58.6 at the exit).
- Every walkreal_0*_live binary (28) on this branch, `--test-threads 4`: all pass except
  walkreal_07000013_live `walk_the_warm_chain_to_the_front_entrance_and_out_real_damage_agent`,
  the same death at t4583 the base report already lists as red on master (not this branch).
  07000015 9/0; 01 5/0 (3 ignored); 03 7/0 (3 ignored); 06 9/0; 11 4/0; 16 19/0; 1b 19/0.

## Unproven

- The new stands and holds are measured on this tree's stream; a later stream change can move
  the guards again. The reacting walk/hold shape is less timing-bound than the old waits (it
  answers whoever is in sight), but it is not proven on any other stream.
- The keypad code is entered through the keypad page state directly (the same shortcut the
  other 15/12 walks use), not by walking the page's cursor.
- Why two of the five frags were left on the floor (carry 9 refusing more) is not traced.
