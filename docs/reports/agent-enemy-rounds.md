# agent-enemy-rounds: guard rounds and glass, the guard move rule on every level

This branch moves the shared random stream (it builds on s/agent-lvl-12, which already moves it; guards now
step and strafe where scenery used to refuse them, and their rounds reach targets behind glass).

## Fixed
- Guard rounds pass glass on every level. `Bullet_update` 0x12e7b0:390 arms the flight's pass bit from the
  weapon record alone (`+0x6C & 0x800` clear), with no check on who fired [M]. The guard shot, the melee impact,
  the GT turret, the space laser, the copter and the `Shooter` objects all take the bit from their record now
  (`bullet::glass_pass_of`). The pane takes the contact through the existing guard-bullet pane arm
  (`Break_Update` 0x12bfd8:36-37), and `Collide_FilterBullets` still drops it for panes whose mask refuses
  the weapon: the 07000012 sky bridge panes (mask 0xffdf) pass guard rounds but do not break [M 0x1ece30:24-33].
- The guard move rule is no longer gated to 07000012. Wherever the drone is bounded and the level has AI bounds,
  the move has no world gate, only the bounds push [M DroneAnim_PreTransCallback 0x13b490,
  NDrone2_Collision 0x140e90:169-174; NDrone2_DealWithObjHit 0x13fff0 returns on a contact with no object].
- A real gap the rule exposed: the combat-move choice (step, strafe, dodge, the strafe's blocked exit) used the
  mover's test, which under the rule only sees doors. Guards picked steps into AI walls that the bounds push undid
  every tick. On the 0700000c walk that was 1651 rejected-wall steps, mostly state 0x73 for hundreds of ticks.
  The game's Can* probes end in `NDrone2_MoveTest` → `AINetwork_BoundsTest` with the near test on (every Can*
  call passes 1 [M 0x14f640:92-93 and its seven siblings]). `choice_clear` now asks the bounds that way. After
  the fix, the same walk showed 32.

## Why the stall rule cost the walks
The game's guard has no world collision on its move [M], so the rule is the game's. The gap above was one real
cause. After it was fixed, the remaining failures come from walk margin: the guards' paths stay the same, but
fights end differently. On 07000005 00 Agent, guard #9 took the same path tick for tick with walls and without;
with walls Bond killed him, and without walls the same guard was alive 63 ticks earlier and hit him. On
0700000c, #33 and #30 survive the lane fights because they now step out of Bond's line, and Bond bleeds 0.6 a
round from 20-25 units.
Whether any guard ends up inside solid scenery the bounds do not cover was not measured.

## Tests
- `guard_rounds_glass_live` (new): a guard fires at Bond through a pane on 07000001, 07000003 and 07000012. Red on
  s/agent-lvl-12 (01, 03), green here. 06, 07, 08, 09, 0a, 13 and 14 have no pane with an armed guard and floor on
  both sides at a usable height, so they have no scene.
- `guard_network_walk_live` (new): 17 levels, difficulty 3, alarms every 5 s, Bond moved in front of a different
  guard every 5 s, 60 s each. It fails on any scenery rejection away from a door and on any 2 s step in place. Red
  on s/agent-lvl-12 (05 #6, 0c #15, 13 #9), green here.
- Green: level12_bridge (6), copter_hits_12, copter_paths, enemies_07000012, interactables 12 and 1b,
  glass_pass_every_level, rocket_glass08, guard_move_checks, guard_first_shot_01, stuck_guard_combat_05, nf-game
  bullet/glass/probe units; walkreal 02, 04, 07, 09, 0a, 0b, 11, 12, 41, 4a (every default test).
- Red here and green on s/agent-lvl-12. These need route re-pins with the stream batch:
  - 01 `..._real_damage_00agent`: glass (green with the glass pass off).
  - 05 `the_door_band_exit_chains_to_07000006_walked_real_damage_00agent`: the move rule.
  - 06 `walk_the_castle_interior_to_the_party_real_damage`: the move rule.
  - 08 `walk_the_whole_mission_on_real_input` (Bond reaches the fusebox on the lower floor after a 2686-tick leg,
    when on s/agent-lvl-12 the same leg gives up at its cap) and `..._real_damage_00agent` (a stair-head wedge):
    the move rule.
  - 0c `walk_the_whole_mission_from_the_start_to_the_0d_handoff_real_damage`: the move rule.
  - 0d `walk_arrival_to_the_c5_door_real_damage`: red with either change switched off alone, so both play a part.
  - 14 `walk_the_tripbomb_route_pick_up_place_and_shoot_it`: three guards (#9, #15, #17) block the under-slab
    crawl door.
  - 15 `..._real_damage_agent` and 16 `walk_the_chasers_defeat_completes_the_mission_on_real_damage`: not
    attributed.
- Red on both: 13 d3 (`walk_the_warm_chain_..._real_damage`) and 1b `walk_arrival_to_drakes_defeat*`.
- 03 `..._real_damage_agent` did not finish within the 560 s run limit, so it is unknown.

## Open
- Unproven: in the game, do the 07000014 guards reach the under-slab crawl door? The bounds allow it here. A
  live check would settle it.
- The game pushes on obj+0x30 (the drone's origin, floor + 1.0 [M DroneInit_Collision 0x14b1e8]) against
  walls of the drone's own cel only [M 0x140e90:169-174, 0x157bd0]. The port pushes the feet (origin − 0.6)
  against every wall. That changes only the height band test on stairs and slopes. It was not changed here.
- Guard rounds that pass glass make no impact effect on the pane (`Effect_Create` runs for each passed hit in
  `Bullet_CollisionHandler`). This is draw-side, recorded and not changed.
- Guard rounds still do not damage non-glass breakables (the guard-bullet pane arm covers kind 2 only); the
  game's `Break_Update` takes every class-5 contact.
