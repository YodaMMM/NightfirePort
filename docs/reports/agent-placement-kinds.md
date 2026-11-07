# Placement kinds and control_funcs against the port

Scope: every case of `parsemap_create_dynamic_objects` 0x1d04f0 and every row of `control_funcs`
0x26f890 (85 rows of 12 bytes: update, collide, delete). Counts are [D]: `Level::parse`, the main
map only, flags & 0xa000 clear. Only the last map's dynamic block reaches the creators, so the
per-level template rows in the sub-maps are not counted.

## Result

Every kind placed on an on-foot level has a port create arm, and every class with such objects
has a port tick or contact path. No class is missing. Four arms inside those ticks were missing
or wrong; all are fixed here, none draws from the shared random stream:

- Class 0x44 MusicTrigger: `obj+0xFE` bit 0 was read as a status flag and the object was never
  deleted. A p3 = 1 record posts its event for one tick in the game [M 0x1b31a0:26-27,
  0x1336f8:86-87]. The port posted every tick while the channel held and 30 ticks after. Records
  affected [D]: 07000001 (event 2), 07000003 (event 2), 07000008 (0xd), 0700000b (0x29, 0x11).
  The p1 kill channel (07000007, 0x96) was dead the same way.
- Class 0x43 Hint: a set off-channel only skipped the tick. The game deletes the marker
  [M 0x1b3380:15, 42-44].
- Class 0x42 ThirdIcon: `obj+0xF4` latches on the on-channel [M 0x196e80:18-20]. The port read the
  live channel, so 07000012 #680 went dark when channel 0x3C dropped.
- Class 0x42 ThirdIcon: param 2, the kill channel, was not read [M 0x196e80:21-26]. Only 07000041
  #105 carries one [D].

## Classes checked with no port object needed

- 6 `Bubble_Update`, 0x19 `Sunreflection_Update`, 0x45 `Corona_Update`: no `obj+0xFF` writer in
  the decompile, so no object of these classes exists.
- 0x23 Creature, 0x4a Sub, 0x4d MiniSub creators: no placements (0x4d is also written by
  `NDrone2_CreateObj`, which the drone code owns).
- 0x4c `OneSided_Update`, 0x26 Flicker: 07000041 and arena 07000029 only.
- 0x28 Car: placed on the arenas and 07000046. Its collide slot is unported, as `car.rs` states.
  Out of scope here.
- Kinds 0xdd, 0xf6, 0xf7, 0xff: no case in the switch.
- 0x4b Mine: the `obj+0x54` spin is unported, but param 2 is 0 on all nine of 07000015's [D], so it
  never turns on an on-foot level.

## Unproven

- Which script drops channels 0x3C (07000012) and the hint off-channels on a real run: the tests
  drive the channels by hand, so how often a player meets these two arms is not measured.
- `ThirdIcon_Update` reads `switch_channels[0]` for icons with param 2 = 0. The port skips the read
  (`kill_channel != 0`). Whether anything writes channel 0 in the game is not traced.
- The large updates (Sensor, Lock, Drone, Door, Searchlight, GT, Copter, SP, RB) were checked for
  presence of the tick and contact path, not line by line.

## Census (main map)

| Kind | Creator | Placements per level |
|---|---|---|
| 0xf | Drone_Create | 01:20 02:25 03:20 04:1 05:16 06:32 07:58 08:26 09:6 0a:20 0b:10 0c:40 0d:41 11:37 12:32 13:12 14:61 15:20 16:20 1b:20 4a:38 |
| 0x20 | Break_Create | 01:55 02:50 03:97 06:1 07:8 08:3 09:17 0a:45 12:97 13:33 14:3 26:9 27:8 29:1 41:16 |
| 0x21 | Ripples_Create | 01:7 02:16 04:4 0b:5 41:1 |
| 0x22 | Ladder_Create | 02:4 03:1 05:1 08:2 0a:1 0c:2 0d:3 15:3 16:1 41:2 |
| 0x24 | Player_AddNewStartPos | 01:1 02:1 03:1 04:1 05:2 06:3 07:2 08:2 09:1 0a:1 0b:1 0c:1 0d:1 11:1 12:1 13:1 14:2 15:1 16:1 1b:1 24:3 25:1 27:1 28:1 29:1 41:1 43:1 46:1 49:1 4a:1 |
| 0x25 | MP_RegisterSpawnPoint | 23:32 24:32 25:32 26:30 27:31 28:31 29:32 4b:32 |
| 0x26 | MP_RegisterMPObject | 23:27 24:25 25:18 26:22 27:28 28:26 29:24 4b:28 |
| 0x27 | GT_Create | 0a:6 0b:5 12:3 |
| 0x28 | Sensor_Create | 01:1 02:2 0a:2 0b:2 12:5 14:14 46:3 |
| 0x29 | Switch_Create | 09:12 0a:9 0b:1 14:2 16:1 46:8 |
| 0x2a | View_AddSkyObj | 01:2 02:2 03:3 04:3 05:9 06:10 07:2 08:5 09:3 0a:4 0b:3 0c:2 0d:2 11:3 12:3 13:4 14:2 16:2 1b:2 23:1 24:2 29:1 41:2 4b:2 |
| 0x2b | LeafGen_Create | 04:2 41:1 |
| 0x2c | Lightning_Create | 01:11 41:4 |
| 0x2d | Player_AddNewStartPos | 01:1 02:1 03:3 07:4 0a:1 0b:2 0c:3 0d:1 16:1 41:1 |
| 0x2e | Lock_Create | 08:2 0a:4 0b:1 0c:5 11:1 12:2 14:3 15:3 46:3 |
| 0x2f | Copter_Create | 05:2 08:1 0b:1 12:1 |
| 0x30 | Monitor_Create | 01:1 02:1 0a:1 12:1 14:12 46:1 |
| 0x31 | FuseBox_Create | 06:1 08:1 14:5 46:2 |
| 0x32 | RainBox_Create | 01:7 02:15 03:9 04:3 05:18 06:21 07:3 08:14 09:12 0a:7 0b:17 14:1 41:2 |
| 0x33 | Hint_Create | 01:1 02:2 03:2 06:3 08:2 09:2 0a:2 0b:1 0d:1 11:1 12:4 13:2 14:24 1b:1 4a:1 |
| 0x34 | GunImp_Create | 0d:1 23:2 24:2 25:2 26:2 27:2 29:2 4b:2 |
| 0x35 | PCQWorm_Create | 0a:3 0b:1 11:1 12:1 46:1 |
| 0x38 | Create_SpaceLaser | 1b:1 |
| 0x39 | Create_SpaceMissile | 1b:8 |
| 0x3a | Grapple_Create | 05:2 0a:1 0c:5 0d:3 11:1 12:1 14:3 23:2 24:7 25:4 26:4 27:8 28:4 29:3 41:1 4b:8 |
| 0x3b | DynamicObject_Create | 02:1 03:1 05:2 07:9 0c:5 0d:5 14:45 16:5 1b:1 41:1 |
| 0x3e | Wire_Create | 02:1 05:1 08:1 0c:4 0d:3 41:5 |
| 0x3f | CreepWall_Create | 05:2 0d:1 41:2 |
| 0x41 | InverterTrigger_Create | 06:1 12:1 14:4 |
| 0x4a | Apocalypse_Create | 14:11 15:4 16:2 1b:8 41:1 |
| 0x4e | Light_Create | 01:4 |
| 0x71 | rotor_init | 01:2 09:1 0a:1 0c:1 0d:1 11:3 12:1 13:1 41:4 |
| 0xd2 | Shooter_Create | 01:1 15:1 41:1 4a:2 |
| 0xd4 | Cloud_Create | 06:1 41:1 |
| 0xd5 | OneSided_Create | 41:1 |
| 0xd8 | RB_Create | 01:4 02:21 03:3 04:2 06:1 0d:3 41:1 |
| 0xd9 | SP_CreateScriptPlayer | 01:3 02:2 03:3 04:2 05:6 06:4 07:3 08:6 09:3 0a:5 0b:10 0c:2 0d:3 11:2 12:4 13:2 14:15 16:15 1b:74 24:1 25:3 4a:4 |
| 0xda | Car_Create | 23:4 24:4 26:4 27:4 28:4 29:4 46:1 4b:4 |
| 0xdb | Door_Create | 01:5 02:3 03:13 05:5 06:12 07:15 08:26 09:6 0a:19 0b:9 0c:2 0d:6 11:12 12:20 13:2 14:55 15:54 16:8 25:6 46:3 4a:162 |
| 0xdc | Trigger_Create | 0c:3 |
| 0xde | Searchlight_Create | 05:29 06:20 41:5 |
| 0xdf | Env_Create | 01:1 02:1 03:1 04:1 05:2 06:1 07:1 08:1 09:1 0a:1 0b:1 0c:1 0d:1 14:2 41:1 |
| 0xe1 | Destroy_Create | 01:11 02:6 03:5 06:6 08:4 0c:12 0d:11 12:1 14:48 15:11 16:7 1b:6 23:10 24:8 25:11 26:6 27:12 29:8 4b:6 |
| 0xe2 | SS_Create | 01:16 02:15 03:8 06:28 08:2 09:2 0a:3 0b:1 0c:8 0d:1 11:2 12:8 13:1 46:10 4b:5 |
| 0xe3 | Flicker_Create | 29:1 41:1 |
| 0xe4 | Hurt_Create | 02:1 05:13 06:1 08:3 0a:5 11:3 15:9 16:2 1b:2 25:6 26:4 46:7 4b:8 |
| 0xe5 | Drone_CoverCornerNode | 01:12 02:3 03:16 05:18 06:80 07:22 08:17 0c:8 0d:56 11:2 12:7 13:19 14:247 15:6 16:38 |
| 0xe6 | Drone_CoverLowNode | 01:8 02:6 03:8 06:34 07:5 08:25 0b:11 0c:78 0d:68 11:24 12:29 13:18 15:11 16:26 |
| 0xe7 | ThirdCam_Create | 02:1 05:4 08:1 0c:4 0d:4 41:2 |
| 0xe8 | Trigger_LoadLevelCreate | 01:1 02:1 03:1 05:2 07:1 09:1 0a:1 0c:1 11:2 12:1 13:1 14:1 15:2 |
| 0xe9 | Drone_AIPoint | 01:11 02:6 03:9 05:3 06:6 07:33 08:1 09:11 0a:8 0c:1 11:12 12:13 13:6 14:2 15:1 16:4 4a:2 |
| 0xea | Trigger_TouchOnce | 01:6 02:24 03:19 05:28 06:49 07:14 08:14 09:3 0a:19 0b:17 0c:7 0d:8 11:22 12:28 13:8 14:42 15:13 16:10 41:2 4a:13 |
| 0xeb | Trigger_Touch | 01:1 02:3 03:2 05:9 06:1 07:1 08:1 0a:30 0b:30 0c:21 0d:19 11:2 12:20 14:13 25:18 46:3 |
| 0xec | Trigger_MultiplexIn | 01:1 02:4 03:2 05:6 06:1 07:2 08:4 09:1 0a:2 0b:4 0c:2 0d:1 14:5 16:4 1b:7 |
| 0xed | Trigger_MultiplexSIn | 02:4 03:2 0a:1 0b:3 11:1 12:2 14:8 |
| 0xee | Trigger_MultiplexOut | 01:2 04:1 0b:2 |
| 0xef | Trigger_MultiplexOrIn | 05:3 06:5 09:1 0a:9 0b:4 14:6 |
| 0xf0 | Pickup_Create | 01:6 02:8 03:5 04:4 05:6 06:11 07:3 08:8 09:3 0a:5 0b:5 0c:15 0d:16 11:13 12:18 13:7 14:32 15:12 16:17 1b:7 23:20 24:28 25:24 26:18 27:19 28:17 29:20 41:1 46:2 4b:28 |
| 0xf1 | DroneSpawner_Create | 01:1 05:2 06:4 08:5 0a:6 0b:2 0c:6 0d:6 11:7 12:5 13:2 14:14 15:1 16:4 1b:8 4a:6 |
| 0xf2 | Emitter_CreatePlist | 02:1 14:35 15:16 46:17 |
| 0xf3 | Env_Tree_Create | 02:6 04:2 |
| 0xf4 | Trigger_MoviePlayer | 41:1 |
| 0xf5 | Drone_AIVolume_Create | 02:21 03:1 05:8 06:109 07:1 08:3 09:1 0a:53 0c:4 14:7 16:14 1b:1 |
| 0xf9 | SoundTrigger_Create | 01:1 02:5 03:4 05:6 06:4 07:6 08:2 09:4 0a:6 0b:12 0c:5 0d:1 11:6 12:2 13:2 14:18 15:1 16:12 1b:5 41:2 |
| 0xfa | ThirdIcon_Create | 02:1 05:9 08:1 0a:1 0c:15 0d:9 11:2 12:1 14:3 41:2 |
| 0xfb | MusicTrigger_Create | 01:1 02:2 03:3 07:1 08:1 0a:1 0b:2 0c:2 0d:1 11:1 |
| 0xfc | Corona_Create | 04:2 05:13 15:76 16:21 41:1 |
| 0xfd | CamSubject_Create | 0c:2 0d:2 41:1 |
| 0xfe | Mine_Create | 15:9 41:1 |
| 0xff | (no case) | 0c:1 0d:2 25:1 |
