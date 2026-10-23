# objective 77 — the tower mission's late AND (0x07000012 ch 0x4d)

Mission 0x07000011's chain (HT Tower2: 07000011 → 07000012 → 0700004a → 07000013) completes its
second 07000012 objective ONE LEVEL AFTER its own record. This doc is the evidence chain; the
campaign runner change is `crates/nf-view/tests/campaign_flow_live.rs` (`drive_12` + the warm-entry
verdict-channel clear), the port fix is the per-tick lock hold in `crates/nf-game/src/drone.rs`.

## The records [L ee_start_05.bin @ 0x2a4188/0x2a41b8/0x2a41e8; docs/formats/mission-data.md]

| part | level | objective | complete ch | enable ch |
|---|---|---|---|---|
| 0 | 07000011 | "Escape from the tower" (0x04000035) | **0x64 = 100** | — |
| 0 | 07000011 | "Gain access to lower office floor" (0x04000036) | 0x48 (72) | — |
| 1 | 07000012 | "Activate elevator override control in security center" (0x04000037) | 0x49 (73) | 0x48 |
| 1 | 07000012 | **"Escape through the elevator shaft" (0x04000038)** | **0x4d = 77** | 0x49 |
| 2 | 0700004a | "Gain access to lobby" (0x04000039) | 0x4a (74) | 0x4d |

Ch 77 is the channel in question: **the writer is NOT on 07000012.**

## Who writes ch 0x4d (77) [D]

- 07000012's placements/triggers/scripts: **no writer** — the level census (census_07000012,
  walkreal_07000012's doc §) shows no TouchOnce/trigger/script/drone whose out is 0x4d. The channel
  graph on 07000012: spawn TouchOnce #679 → 0x48; the keypad locks #656/#694 → 0x0a (the door[19]
  unlock); the console E (#731/#702) → 0x49 + 0x59; the lift call #699 → 0x58; the shaft touch #683
  holds 0x1f gating the exit #669 (0x0700000a's own channel set ends there).
- 0700004a: **TouchOnce #277** (the shaft's first plate, y −9.87..−7.65 [D 0700004a.md §2]) writes
  0x4d on Bond's contact. The spawn drop (#43 at (4.26, −7.71, −4.76) [D]) lands inside its volume.
- The fold is the monitor's case 3 [M Mission_MonitorObjectives 0x1827d8:78-85 — state 3, cond ON →
  4], wherever the level: the game's own "late AND".

So on the real walk: the override console E raises 0x49 → obj 1 of 07000012 folds AND obj 2
ANNOUNCES (its enable is 0x49 [M 0x1827d8:55-67]); the shaft exit E loads 0700004A
[Trigger_Activate 0x1b2318] and #277 raises 0x4d → obj 2 folds there.

## The monitor's AND is unconditional — and one objective is pinned to ch 100 [M 0x1827d8:32-38]

The decompile ANDs `bVar9 &= bVar1` for EVERY record before the state switch — including records
whose condition channel is 100. 07000011's "Escape from the tower" IS ch 100 [L 0x2a4188], so the
monitor's own success write (`if (bVar9) switch_channels[100] = 1` [M 0x1827d8:119-123]) is
self-referenced for this mission: the tower mission completes only when something ELSE writes ch
100 (the level data's own writer — a live-check item below), and firing ch 100 as an "objective
stand-in" is always a fiction.

## The runner invariants (why the campaign table showed "unfired [77]")

1. A real warm entry never finds ch 99/100 ON: ch 100 rises only via the monitor's all-AND or the
   level's own writer, and the arms it triggers end the level themselves (complete → the
   DRIVING/front-end codes [M 0x182c60:85-122], fail → the 0x40000042 menu [M 0x182c60:77-81]) with
   the player paused. The old runner blind-fired ch 100 as 07000011 obj 1's channel stand-in → the
   mission completed mid-loop on 07000011 → the carried ch 100 re-ran the complete arm on
   07000012's FIRST monitor tick (Mission_Init 0x182270 re-arms internal 1 but resets only the
   current+future parts' objective channels — 99/100 are nobody's reset) → internal 7→4→5 → the
   monitor never ran on 07000012 → nothing folded there → "unfired objective channels [77] (forced
   complete below)". Fixes (campaign_flow_live.rs): the warm-entry clear of ch 99/100 before the
   layer's first tick; the fallback never fires channels 99/100; 07000012 is a DRIVEN leg
   (`drive_12` = the real tower2 route) so no stand-in runs at all; the driven cell counts states
   `< 4` (folded), not `< 2` (announced).
2. `Lock_Update` case 1 holds an unlocked lock's out channel EVERY tick [M 0x1ae3d8:192-196 —
   `case 1: switch_channels[puVar1[4]] = 1`]. The port wrote it once at create; a warm load's
   carried channels restore after that, so 07000012's Operative keypad-unlock (param 6 = 1 [M
   0x1ae1f8:67-69] + difficulty 1) sat dead — ch 0xa never held, door[19] unreachable. The port now
   re-holds per tick in the object-pass slot (`Drones::tick_inner`, after Env_Update — the game's
   object handler order [M 0x1c9790:24-34]); channel_persist's owner set gained the same family.

## Live checks (open)

1. The tower mission's ch-100 writer: does 07000013's data (triggers/scripts) write ch 0x64? The
   monitor's self-reference means the mission completes only through a level-side write; watch
   `switch_channels[100]` (0x26fc90+0x64) during a real 07000013 play, expected a 0→1 write from
   the level's script/trigger path (the port's 13 leg fires nothing — its row reads 1/1 with the
   mission still running, exit→0x07300013).
2. 07000012 Operative fresh boot: door[19] (plc 754) — expected the unlock edge at frame 1 (the
   "Door Unlocked" INFO label + open) and the 360-tick auto-close shutting it before the player
   arrives. The port's warm leg shows flags 0x3 (no busy bit 0x10) where the cold Agent burn path
   sets it — check `obj+0xF0` on the disc's path (the busy marker's writer) — gdbwatch on the door
   object's +0xF0 across frames 1..400.
3. The late AND's HUD: on the real 0700004A spawn, "OBJECTIVE COMPLETE" (0x02000053) for "Escape
   through the elevator shaft" (0x04000038) should show within ~2 frames of the #277 touch
   [M 0x1827d8 case 3's 180-tick msg].
