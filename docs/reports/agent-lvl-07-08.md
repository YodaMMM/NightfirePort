# agent-lvl-07-08

Levels 07000007 and 07000008: placements, channels, objectives and level-gated decompile sites checked
against the port.

## Fixed

1. 07000008 zip line (#19, the game's one type-2 wire). The rider faced uphill and accelerated from a
   crawl (about 5 s). The grab faces down the slope [M 0x197130] and the ride is a constant 0.15 per tick
   along the facing, about 2.5 s: `Player_Update` zeroes BL+0x10 on every live tick, because the gate is
   obj+0xF4, the object state [M disasm 0x1937e0, 0x193830], not the substate.
2. 07000007 rope barriers and the doorway blocker. 0x3b objects had no collision. They now stand solid
   while obj+0xF0 lacks bit 0x20: #495 turns solid on ch 21, #496 between ch 9 and ch 0x48
   [M 0x18dbf8, 0x18d3b8, 0x1eae80].
3. Placed glass and breakables on 07000007 and 07000008. Bond walked through the intact 07000008
   control-room panes and fell out of the building. Placed 0x20 objects there are now solid while alive
   [M 0x12c230, 0x12b970:60]. The law is the same on every level; only these two are switched on.

4. 07000007, the waiter frozen in the west library doorway (#426, door #245 at (80.00, −49.87)). He
   wakes on ch 23 and walks Path_3 west through the doorway, pushing the door open from the east so
   the leaf lies along z −49.87, x 78.7..80.0. His creep then cuts toward node 13 (77.48, −49.77)
   across the leaf's face, and the port's `move_by` rejected that step every tick: he walked on the
   spot at (79.51, −50.33) for good. The game pushes a drone out of what it walks into
   [M Drone_CollisionHandler 0x140968:204-208, the DeltaP add], so a step into a door leaf now keeps
   its along-the-leaf part. Why earlier rounds did not hold: neither landed. The first round was in a
   held batch that was never pushed, and the second was never merged, so master had no fix at all.
5. 07000007, the library door #244 opened and Bond walked straight in past its two guards. #404/#417
   (CivilianDoorGuard 0x31) stand at z −48.06 in front of it until ch 21, Drake's reveal, takes them
   off [D p6 = 21, p8 = 100]. A drone within 2.0 of Bond has collision ON [M NDrone2_DoCollision
   0x149ec8, DroneFunc_NearPlayerCollision 0x14b190] and the SP Jointy push moves the player off its
   capsule [M Collide_Jointy 0x1ec180:229-281]. The port pushed only an airborne Bond, so a walking
   Bond passed through both. Door guards now push a grounded Bond too (lateral only). The door itself
   still opens on E: `Door_Activate` 0x135be0 has no gate for it, #244 has no lock channel [D p3 = 0],
   and the use pick is a sphere, not a ray. The condition is the reveal: go upstairs first.
   Same data, same rule: the east corridor pair #441/#442 holds until ch 21, and #448/#449 then hold
   it until ch 0x48, and ch 21 ropes the west doorway (#495). So the real order is party, reveal,
   library, east stairs, exit. The 07 walks (`walkreal_07000007_live`, `party_door_lock`,
   `cutscene_timer_pause`) now take the reveal first through `walker::drakes_reveal_07`.

Tests: `zipline_07000008_live`, `solid_objects_07_08_live`, `library_door_07000007_live` (the waiter
walks through; the guards hold the doorway before ch 21). All six fail on the code before each fix.

## 07000008 safe hinges: no port bug found

Reproduced standing at (160.52, 43.12, −117.33) with the laser held at 19 aim points on and
around the lower hinge. The bolts #207/#208 are thin pins (0.06 wide, 0.41 tall) with a 0.208 world
sphere [D glist bounds], and the burn needs the beam's per-tick segment inside that sphere for 84 ticks
[M Lock_Update 0x1ae3d8:57-101, Collide_PickObj 0x1eae80:91-107, the sphere radius alone]. Every aim
from y 42.55 to 42.95 on the lower pin burns it and writes ch 135; aims at 42.40-42.50 (the bottom tip,
the knuckle end) and at 43.0 and above miss. That matches the decompile. Two visible gaps
remain, both draw-side and left alone here:
- `Lock_Update` spawns the burn sparks `Emitter_Create(0xc00000a, contact point, normal)` on every
  contact tick [M 0x1ae3d8, the 0x4e/0x4f arm]. The port spawns none on a lock, so a beam that IS
  burning looks the same as one that misses.
- A burnt bolt shows no change (the open framelist pose is not drawn); with two bolts, the first burn
  gives only the 0x249 sound.

## Scoped out, open

- Mode-1 wall-hug blockers (07000005, 0700000d) stay non-solid: with them in, the walks there stop.
  Question for the real game: does a walking Bond stop at #324 on 07000005 while ch 113 is off?
- Panes stay non-solid off 07000007/8. Solid everywhere, 07000001 and 07000014's walks stop at panes they
  do not break first, and the real-damage walks of 07000006 (Agent) and 07000011 die. 09, 0a, 0b, 0d and the
  other walks stayed green. Question: do the 01 and 14 routes meet a whole pane in the real game?
- `Bond::tick_ctx` zeroes the move quad on climb ticks only, reading the obj+0xF4 gate as the substate. The
  game zeroes it every live tick, so a hand-over-hand rider should read as still to guard accuracy. Left as
  is: a 0700000d walk was tuned on it.
- 0xE1 destroyables are not in the collision set either [M 0x12c318 links them like the others]. Not changed.
- `DroneFunc_CheckAlarmRaised`'s ch 0x61 arm runs on every level [M 0x1453a8]; the port runs it on
  07000005/6 only. No 07/08 placement writes ch 0x61, so not changed.
- The leaf slide is switched on for 07000007 only. On every level it moves combat guards that now stand
  frozen at door leaves on 07000008 (states 0x68/0x73 at (105.9, 28.2, −121.2) and (161.5, 46.0,
  −101.6)), and the 07000008 00 Agent walk then runs out of ammo before the cabin panes.
- The door-guard pairing leaves #404 and #417 both as 0xff mirrors (the earlier [S] watch-across
  deferral), so they never speak the zone-2 challenge or raise the zone-3 fail (reason 0xc). Not changed.
- `the_prop52_guard_first_attack_fails_the_mission` is ignored: with the guards holding the library
  door, no 0x52 carrier is in sight of an armed Bond on the real path (ch 21 takes the hall carriers
  off and ropes the west doorway). `probe_07_route` (assertion-free) is ignored for the same reroute.

Unproven: nothing here was checked against PCSX2; the fixes rest on the decompile and disc data.
Questions for the real game: does a walking Bond stop at #404/#417 before the reveal; does the laser on
the lower hinge's bottom tip (y about 42.45) burn it.
