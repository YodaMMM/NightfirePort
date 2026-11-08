# agent-music-machines: the per-level music machines

## Every `UpdateMFX_*`, its level and the port

The dispatcher is `UpdateMusicalEvents` 0x114e70:219-285 [M]. 21 machines; nothing else is keyed.

| Machine | Address | Level | Port (`crates/nf-game/src/music.rs`) |
|---|---|---|---|
| 02MayhewA | 0x111ed8 | 07000001 | missing on master; `mayhew_a` on s/agent-lvl-01-02 |
| 02MayhewB | 0x112270 | 07000002 | `mayhew_b` |
| 02MayhewC | 0x112730 | 07000003 | `mayhew_c` |
| 02MayhewD | 0x112bf8 | 07000004 | missing on master; `mayhew_d` on s/agent-lvl-03-04 |
| 01CastleA | 0x112d78 | 07000005 | `castle_a` |
| 01CastleB | 0x113060 | 07000006 | `castle_b` |
| 01CastleC | 0x113188 | 07000007 | `castle_c` |
| 01CastleD | 0x1133b8 | 07000008 | `castle_d` |
| 03TowerA | 0x113558 | 07000009 | **new: `tower_a`** (the exit head was already in) |
| 03TowerB | 0x1137b0 | 0700000a | **new: `tower_b`** (the exit head was already in) |
| 03TowerC | 0x113a20 | 0700000b | `tower_c` |
| PowerStationA1 | 0x113c18 | 0700000c | **new: `power_station_a1`** (the exit head was already in) |
| PowerStationA2 | 0x113fc0 | 0700000d | **new: `power_station_a2`** (the exit head was already in) |
| HT_Level_Tower2A | 0x114250 | 07000011 | shared exit block, which is the whole function |
| HT_Level_Tower2B | 0x114348 | 07000012 | `tower2_b` |
| HT_Level_Tower2C | 0x114640 | 07000013 | the alarm jump in `update` + the exit block |
| HT_Level_Tower2Elevator | 0x114528 | 0700004a | `tower2_elevator` |
| HT_Level_EvilBase | 0x114728 | 07000014 | `evil_base` |
| HT_Level_EvilSilo | 0x114a38 | 07000015 | `evil_silo` |
| HT_Level_EvilBaseC | 0x114b38 | 07000016 | `evil_base_c` |
| HT_Level_SpaceStationD | 0x114d60 | 0700001b | `space_station_d` |

The other MapMusic rows (the 0x23-0x29, 0x41-0x4c group) have no case in the switch [M].

## What each new machine listens to

Event slots are `(addr - 0x2d34f8) / 4`. Producers counted on every level [D]:

| Level | Slot | Producers |
|---|---|---|
| 09 | e[43] | zone cels 0, 1, 4 |
| 0a | e[40] | zone cels 0, 9; MusicTrigger on ch 48 |
| 0c | e[45] | zone cels 0-4; MusicTrigger on ch 5 (the turbine door's out channel) |
| 0c | e[13] | MusicTrigger on ch 0x24 (the yard volumes); wire and crawl |
| 0d | e[45] | zone cels 1-9 |
| 0d | e[13] | MusicTrigger on ch 9 (the crane-yard volumes); wire and crawl |

The game's `&o_fx_int`, `&o_cd`, `&o_sadd` and `&o_left` requests are absolute symbols with the values
4, 0xc, 0x10 and 0x14 [D symtab], so they are plain section numbers. TowerA and TowerB compare them.

## 0d's trigger is never cleared

`UpdateMFX_PowerStationA2` tests `SfxNisMusicTrigger` and does not clear it: the function has no store to
it [M disasm], while MayhewD and the Castle machines do. The only other write is the restart. The intro
0x600068f plays sound 0x172 (tick 696 on the real level), so a played intro sets it for good. The arm then
runs every frame, the 600-tick hold is re-armed every frame, and the alarm, e[13] and e[45] arms never run:
the score stays on section 0x2b. A skipped intro plays no 0x172, so its end jumps 0x2b once and the rest of
the machine runs. The port does exactly this.

Unproven: this reading comes from the code, not a live capture. The live question: on 0700000d with the
intro played through, does `SfxNisMusicTrigger` 0x30c8b4 stay 1 and the IOP section stay 0x2b for the level?

## Found, not changed

`UpdateMusicalEvents` itself has attack-gate writes the port lacks: e[10] sets `MusicVars = 1` [M :125-127];
e[2] == 2 seats the -99 `MapDroneData` row (count 1, window 10, distance 50) [M :130-152]; bonus < 0 and the
alert expiry re-seat the level's row [M :103-123, :169-188]. They touch the levels with an e[10] zone (01, 06,
07, 12) or an e[2] trigger (01, 03), not the four here. `GateWrite::Restore`'s no-row fallback in drone.rs
writes floor 8 where the game writes 0 to the floor and 8 to the window; 02, its only user, has a row.

## Tests

`crates/nf-view/tests/music_machines_live.rs` runs the real levels through the real zones, triggers and
intro. With master's music.rs, 6 of its 7 tests fail; the seventh pins the disc rows. Headless there is no
IOP, so the test reports back the section last requested. The alarm is written to slot 2 directly.
