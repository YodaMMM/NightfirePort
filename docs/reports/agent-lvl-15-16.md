# agent-lvl-15-16: levels 07000015 and 07000016

Three commits on this branch move the shared random stream or actor paths. See "Walk results".

## Commits, in order

1. **Kiko and the death plates.** Safe to take alone.
   - Kiko #612 and #634 (skin 0x0500005e, voice 0x13) take no impacts on 07000016. `NDrone2_DefaultInit`
     sets `Drone+0x1B` [M 0x14b300:699-700] and `DroneFunc_HandleImpact` returns on it [M 0x1469b8:23-25].
     Before, Bond could shoot her dead, and then the tripwire #613 could never fail the mission.
   - 07000015's death plates carry the one-shot flag and no keys [D]. `Hurt_Update` returns before the
     expiry on a zero duration [M 0x17e5d0:106-108]. Before, a guard landing on a plate retired it, and
     Bond then fell through alive.
2. **Guard calls on 14, 15, 16.** `NDrone2_SeenPlayerTalk` and `NDrone2_AttackTalk` had no arm for these
   levels, so guards said nothing on sight. Added both, with the second voice from the id's parity
   [M 0x13f1d0, 0x13eac0]. A spawned drone's id is the port's index, so its voice choice is [S].
3. **Spawner distance.** Param 8 is an integer [M `cvt.s.w` at 0x1379d4; D values 0..14]. The port read
   its bits as a float, so the screen was off on all 59 spawners that set it. 15 and 16 carry 1.
4. **DynamicObject collision.** Kind 0x3b objects were never solid. `Collide_PickObj` tests them unless
   obj+0xF0 has 0x20 [M 0x1eae80]. They now join the moving set, and the sight ray skips the 0x10 ones.
   On 16 this is the lift-shaft blocker #532, solid from t2249 to t2803 of 0600090a.
   One stand-in [S]: a 0x10 part stays out of the set while Bond is inside its box. The 05 window crawl
   exit leaves him 0.78 inside blocker #325, and the port's closest-point push would hold him there.
   The game's way out of that box is not traced.

## Findings with no port change

- Corrected in the level docs: ch 0x32 is the group-1 squad's death channel, so spawner #690 is live.
  Ch 0x34, 0x15 and 0xf1 are spawner kill channels. Ch 0xf4, 0xf5, 0xcc, 0xcd, 0x50 and 0x1b all have
  script writers. Kiko #612 is what sets off #613, not the group-6 chaser.
- Kiko's walk to #613 works: about 480 ticks after ch 0x1b, then ch 0x7b and the mission fail.
- Dead in both game and port: 15's ch 0xc8 doors and ch 0x42 exit, 16's tower_door3 (0xb5 has only the
  0x06000633 write, and no placement plays that script).
- No divergence found in Door_Create, Switch, Apocalypse, Shooter, SoundTrigger, Pickup_Create,
  the difficulty gate, the music events or the watch clock.

## Walk results

Each new test was run red on the old code and green on the new.

Green on the whole branch: walkreal 01-04, 06, 07, 09, 0a, 0b, 11, 12, 14, 15, 1b, 41, 4a,
interactables 15 and 16, worldobj_live, kiko_button_real_walk, fix_finale, blast_room_shelter_16,
pack_cinema_16. The 14 and 15 runs include commit 2; the rest ran before the last commits were split.

Red on the whole branch, every one a real-damage walk where Bond dies:

| Binary | Test | Red with commit 4 alone (no 2, 3) |
|---|---|---|
| walkreal_07000005 | the_door_band_exit..._real_damage_00agent | yes |
| walkreal_0700000d | walk_arrival_to_the_c5_door_real_damage | yes |
| walkreal_07000008 | ..._real_damage_agent, ..._00agent | no |
| walkreal_0700000c | walk_the_whole_mission..._real_damage | no |
| walkreal_07000013 | walk_the_warm_chain..._real_damage | no |
| walkreal_07000016 | the_chasers_punch_lands_the_record_damage, ..._on_real_damage | no |

So commits 2 and 3 cause the last four rows, and commit 4 or a red that predates this branch causes the
first two. None of them was run on untouched master, and 2 and 3 were not separated. They need a re-pin.

## Unproven

- How the game gets Bond out of blocker #325 after the window crawl.
- Whether 05's and 0d's two reds exist on master.
- Draw side: nothing changed. Debris on blasts stays unported (GAP-15-1).
