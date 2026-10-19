# objectives-all-levels — the mission layer's objective census (every on-foot level)

The mission layer is ported in `crates/nf-game/src/mission.rs` + `mission/c070000XX.rs`:

- `Mission_Init` 0x182270 arms the records (current + future parts reset; prior parts keep their
  state, fabricated announced/complete at a cold mid-mission arrival [M 0x182270:68-129]);
- `Mission_MonitorObjectives` 0x1827d8 runs every tick from internal 1: per record the condition is
  flags&2 → channel OFF, else channel ON (the raw table byte; the game has no ch-0 guard
  [M 0x1827d8:32-38]); the state-1 enable arm has no level gate [M 0x1827d8:55-67]; the state-0
  announce arm gates on `record.level <= level` [M 0x1827d8:41-54]; the AND of every condition
  gates ch 100 [M 0x1827d8:119-123];
- `Mission_Update` 0x182c60: ch 99 → fail (state 2), ch 100 → case 6: MissionState 6, the per-level
  `LevelToEndTo` (0x07000016 → 0x07300017 etc. [M 0x182c60:85-122]), the "Mission Complete" type-3
  message, timeout 240 → `ResetMap_LevelToLoad` hand-off [M 0x182c60:65-76];
- `Mission_NumVisObjectives` 0x182540 / `Mission_ObjectiveState` 0x182630 walk the LEVELS table in
  order (prior parts included, future parts cut by the part gate) — the pause OBJECTIVES tab's rows
  and the mission-results data the view reads.

## The census — 55 objective records, 9 missions, 24 on-foot levels

Every record below is `[L ee_start_05.bin @ <addr>]` (the per-file header cites the exact address;
the quoted texts resolve from USATxt.dat by the port's `Lang::bind` [D]). `req`/`FAIL` = flags bit 2
(fail objective: its channel ON fails the mission). `en` = the reveal (enable) channel.

| Level | Mission | Part | # | Records (text id | channel | kind | en | init) |
|---|---|---|---|---|
| 0x07000001 | 0x07000001 | 0 | 1 | 0x04000006 "Escort Mayhew to bunker" ch 0x47 req |
| 0x07000002 | 0x07000001 | 1 | 3 | 0x04000007 "Destroy Mayhew's computer" ch 0x48 en 0x47; 0x04000008 "Rescue geisha girls" ch 0x4b en 0x47; 0x04000009 "Find door to servants quarters" ch 0x4d en 0x4c |
| 0x07000003 | 0x07000001 | 2 | 3 | 0x0400000a "Retrieve Dragon Safe contents" ch 0x4e en 0x4d; 0x0400000b "Rescue Mayhew's servants" ch 0x51 en 0x4d; 0x0400000c "Find door leading to Mayhew" ch 0x53 en 0x52 |
| 0x07000004 | 0x07000001 | 3 | 1 | 0x0400000d "Defeat the assassin" ch 0x54 en 0x53 |
| 0x07000005 | 0x07000005 | 0 | 1 | 0x04000000 "Breach the castle walls" ch 0x46 en 0x45 |
| 0x07000006 | 0x07000005 | 1 | 1 | 0x04000001 "Find a way inside to the party" ch 0x02 en 0x46 |
| 0x07000007 | 0x07000005 | 2 | 2 | 0x04000002 "Rendezvous with undercover agents…" ch 0x48 en 0x47; 0x04000003 "Spy on secret meeting" ch 0x49 en 0x48 |
| 0x07000008 | 0x07000005 | 3 | 2 | 0x04000004 "Retrieve guidance chip from Safe Room" ch 0x4a en 0x49; 0x04000005 "Escape with Zoe in gondola" ch 0x4b en 0x4a |
| 0x07000009 | 0x07000009 | 0 | 3 | 0x04000016 "Don't let the security guards raise the alarm" ch 0x61 **FAIL**; 0x04000017 "Get to the lobby…" ch 0x46; 0x04000018 "Take main elevator to office floor" ch 0x47 en 0x46 |
| 0x0700000a | 0x07000009 | 1 | 3 | 0x04000019 "Install Q-Worm…" ch 0x48 en 0x47; 0x0400001a "Find the security center…" ch 0x49 en 0x47; 0x0400001b "Find door to outer balcony" ch 0x4b en 0x4a |
| 0x0700000b | 0x07000009 | 2 | 5 | 0x0400001c "Install Q-Worm on Mayhew's computer" ch 0x4c en 0x4b; 0x0400001d "Locate the Secure Terminal Room" ch 0x4d en 0x4b; 0x0400001e "Retrieve the NightFire files" ch 0x4e en 0x4d; 0x0400001f "Escape to the roof" ch 0x59 en 0x4f; 0x04000020 "Acquire parachute… jump off tower roof" ch 0x5a en 0x59 |
| 0x0700000c | 0x0700000c | 0 | 3 | 0x0400002c "Investigate building ST-1" ch 0x1b; 0x0400002d "Photograph the Jetpack Prototype" ch 0x1c en 0x1b; 0x0400002e "Advance through Warehouse TR-2" ch 0x23 en 0x1c |
| 0x0700000d | 0x0700000c | 1 | 3 | 0x0400002f "Investigate building ST-3" ch 0x1d en 0x23; 0x04000030 "Photograph the Laser Prototype" ch 0x1e en 0x1d; 0x04000031 "Destroy door C5 with mounted laser" ch 0x1f en 0x1d |
| 0x0700000e | 0x0700000e | 0 | 4 | 0x04000041 "Disable all security measures…" ch 0x0f; 0x04000042 "Disable system 1 of 3" ch 0x0a en 0x07; 0x04000043 "Disable system 2 of 3" ch 0x0b en 0x08; 0x04000044 "Disable system 3 of 3" ch 0x0c en 0x09 |
| 0x0700000f | 0x0700000e | 1 | 1 | 0x04000045 "Mission Objective 5" **ch 0x00** [gap] |
| 0x07000010 | 0x07000010 | 0 | 3 | 0x04000046 "Mission Objective 1" **ch 0x00** [gap]; 0x04000047 "Mission Objective 2" **ch 0x00** [gap]; 0x04000048 "Mission Objective 3" **ch 0x00** [gap] |
| 0x07000011 | 0x07000011 | 0 | 2 | 0x04000035 "Escape from the tower" ch 0x64; 0x04000036 "Gain access to lower office floor" ch 0x48 |
| 0x07000012 | 0x07000011 | 1 | 2 | 0x04000037 "Activate elevator override control" ch 0x49 en 0x48; 0x04000038 "Escape through the elevator shaft" ch 0x4d en 0x49 |
| 0x07000013 | 0x07000011 | 3 | 1 | 0x0400003a "Raise security gate and escape" ch 0x50 en 0x4a |
| 0x07000014 | 0x07000014 | 0 | 5 | 0x04000049 "Infiltrate Drake's facility undetected" ch 0x47; 0x0400004a "Follow Kiko to the Security Control Center" ch 0x48 en 0x47; 0x0400004b "Sabotage base security and escape" ch 0x4a en 0x48; 0x0400004c "Prevent Rook from cutting the power" ch 0x54 en 0x4a; 0x04000069 "Take the exit to Delta Sector" ch 0xce en 0x54 |
| 0x07000015 | 0x07000014 | 1 | 1 | 0x0400004d "Find the exit to Omega Sector" ch 0x8e en 0x4b |
| 0x07000016 | 0x07000014 | 2 | 2 | 0x0400004e "Reach Drake's launch silo" ch 0x8f en 0x8e; 0x0400004f "Prevent Kiko from launching into space" ch 0x90 en 0x8f [gap: no writer on the disc] |
| 0x0700001b | 0x0700001b | 0 | 2 | 0x04000059 "Disable the missiles…" ch 0x09; 0x0400005a "Defeat Drake…" ch 0x0b en 0x09 |
| 0x0700004a | 0x07000011 | 2 | 1 | 0x04000039 "Gain access to lobby" ch 0x4a en 0x4d [gap: no writer found] |

## Writers (how each channel fires) and port status

The writer for each channel is the level's own interactive layer — TouchOnce plates, use targets
(`Player_Activate` 0x1a5170 / `SP_Activate` 0x1dc338 / `Script_EntityStart` 0x1d8d78), doors, deaths
(`Player_HandleDeath` 0x190bb0 writes ch 0x62), switch triggers, script players and monitors. The
per-level mapping with addresses lives in `docs/levels/<map>.md` §Objectives for the levels already
walked (07000009/0a/16/4a/4b…), and the walkreal tests pin the writers through the real input path.
The port runs the whole mission layer above (this task's census test exercises all 55 records).

Known gaps (objectives that cannot complete in a real walk today):

1. **07000016 ch 0x90** "Prevent Kiko from launching into space" — no writer in the level's
   triggers/scripts [D, docs/levels/07000016.md]; the port carries the documented stand-in (chaser
   defeat → AND #271 → ch 100 path). Needs a live check (gdbwatch ch 0x90 during the Kiko fight) to
   find the real writer.
2. **0700004a ch 0x4a** "Gain access to lobby" — no writer in the level's triggers, 11 scripts or
   the 4 script players [D, docs/levels/0700004a.md §Objectives/§6].
3. **0700000f + 07000010 ch 0x00 records** — the disc ships channel 0x00 (the monitor reads it as a
   real index [M 0x1827d8:32-38]); no ch-0 writer mapped on those levels yet. These are the two
   levels missing docs (docs/levels/0700000e.md, -0f, -10 not written).
4. **007 bonus conditions** — no Bonus symbol exists in the mission layer (`work/action-symtab.tsv`
   scan: no `*Bonus*` function); the pause tab's per-objective hints are the records' hint field
   [M ObjectiveState 0x182630]. Not a mission-layer feature.

## Test

`crates/nf-game/src/mission.rs::tests::every_level_objectives_complete_through_the_monitor_and_gate_ch_100`
— for every LEVELS entry: cold `Mission_Init`, every condition channel set the way a writer would
(normal ON, the fail record's channel OFF, enables fired), two `Mission_Update` ticks → asserts
ch 100 [M 0x1827d8:119-123], MissionState 6 + the `MissionComplete` event [M 0x182c60:85-122], and
every reached record announced-or-complete. Sister: the existing `prior_parts_fabricate_complete…`
test fails the old fabrication behaviour.
