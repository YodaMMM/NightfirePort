# agent-01-sight: 07000001 guards beyond the paper screens

**This branch changes guard behaviour and turns the 01 walks red.** `walkreal_07000001_live` fails 4 of 8
(the three contact walks and the exit fade) and `escort_follow_07000001_live` fails 1 of 1. Both binaries
pass on master. Every guard released from WaitSwitch now enters Combat 0x58 (it ran to AimStand 0x67
before), so the 01 hall fight plays out differently. In the top-up walk, hall guard 5 holds AimStandFire
0x68 for about 2000 ticks before he dies. After that the contact waits in AllyLeadWait 0x20 at
(3.5, 6.3, 6.6), and Bond's close-in nav stalls at the door at (7.9, 5.3, 11.1). The walker's routes need
re-tuning for the game's release behaviour, or the fix needs a narrower reading. Not done here.

Spot: Bond at (11.43, 6.47, -3.51); guards at (6.95, 6.34, 2.20), (6.98, 6.34, 8.48), (5.42, 6.34, 14.44).
Test: crates/nf-view/tests/sight_screens_07000001_live.rs (two tests, both on the real level).

## Pass 1: no channels raised
Game [L live-1108 section 6]: all three stay WaitSwitch 0x01 for 600 frames. Port: the same, every tick.
They are disabled (switch channels 0x5, 0x5, 0xe), so this pass never tests sight.

## Pass 2: ch 0x5 / 0xe raised (the game's second run)
Game [L live-1108 section 9]: the guard at (6.95, 2.20) leaves 0x01 for Combat 0x58 five frames after Bond
is placed, goes 0x5c -> 0x81 -> 0x5c -> AimStandFire 0x68 at +53, holds 0x68 for 547 of 600 frames, and his
rounds reach Bond. So the game guard does see and shoot Bond here, through an opening.

Port before the fix: the released guard went 0x56 -> AimStand 0x67 and stood there 75 ticks with no sight,
reaching 0x68 only at +159.

Cause: `Drone_GetOpponentInfo` 0x138f68 returns at once unless `+0x4F8 & 0x100` (the port's `enabled`), so a
WaitSwitch drone keeps the distance it had when it was disabled (about 17.5 from the load spot). On release
`NDrone2_DSTATE_Attack` 0x164210 compares that stale distance with the engage range (15.7): out of range,
so Combat 0x58. The port refreshed the distance for disabled drones every tick (7.3 here), which picked
AimStand. Fix: `Drones::tick` runs `opponent_info` only for enabled drones (the aim point stays live).

Port after the fix: 0x01 -> 0x58 at the release tick, 0x5c, 0x68 at +34, 0x68 for 473 of 600 ticks, about 36
rounds, all aimed at Bond, some reaching him. The second guard (released at the game's +748) goes 0x58 ->
0x5a -> 0x5f as the game's did.

How the test releases them: ch 0x5 is the OUT copy of ch 0xd (trigger #5, targets [4, 5]), rewritten every
frame, so in the game run a poked ch 0x5 is only seen when the poke lands between that copy and the drone's
update. The port steps whole ticks, so the test points each guard's switch channel at ch 0xe (held on) at
the game's own release frame. The WaitSwitch release path is the same.

## Differences left (not fixed)
- The game's guard steps through a door (0x81) before aiming. The port's guard sees Bond sooner (+34 against
  +53) and does not take that step.
- The port's guard ends near (9.6, 2.15) at yaw 2.83. The game's turned to yaw 2.27, with his muzzle at
  (7.83, 6.47, 1.73).
- The game's round count (24) comes from a separate 6 s window, so it is not compared one to one.

Unproven: whether the game's sight line passes through a paper screen or through the open doorway. The game
run's rounds met no surface in between.

## Tests run
- sight_screens_07000001_live: 2/2 pass. The second test fails on the old code (release went to 0x67).
- Pass: walk_07000001_live, ai_traces_live, guards13_start_live, stairwell_guards_07000011_live,
  start_corridor_guard_07000011_live, security_alarm_09_live, searchlight_reveal_06_live, rook_door_06_live,
  inert_guards_02_live, hostage_killer_07000002_live, civilian_mission_wait_live, enemy_accuracy_live,
  nf-game ai_live.
- Fail: walkreal_07000001_live (4 of 8) and escort_follow_07000001_live (1 of 1), as above.
