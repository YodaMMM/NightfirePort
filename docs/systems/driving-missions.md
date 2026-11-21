# The driving missions' rules: triggers, actions, events, rules, the end

Status: ported (2026-11-21). Code: `crates/nf-formats/src/trackstart.rs` (`map_script`: the
track's triggers, rules, `MSet` and `sr` paths) and `crates/nf-game/src/driving/script.rs`
(`DriveScript`: the mission manager, the trigger walk, the event queue). `Mission::from_map_script`
+ `Mission::step_facts` (`driving/mission.rs`) wrap it for the existing callers. Evidence: [M] the
DRIVING.ELF decompile (EE addresses), [D] the disc data of the eight tracks. Everything not
tagged is stated as unproven below.

## 1. The pieces

| piece | game | port |
|---|---|---|
| action lists | `el` records of the `Map` window; count at `+0`, 16-byte entries from `+0x10` (`+0` id, `+8` argument offset); the argument block repeats the id, parameters from `+4` [M 0x22c890, D] | `TriggerAction { id, args }`, `word(i)` / `float(i)` |
| creators | `DAT_0036fda0[id]` (`FUN_001757b8`) [M] | `ACTIONS` table: 119 ids, class, creator, handler |
| events | bump-allocated in the frame buffer (`FUN_00175730`), drained in order by `FUN_00175690`, which calls each event's virtual destructor: the destructor is the handler (vtable `+0xc`) [M] | `create` (creator + constructor) and `execute` (handler); events queued while draining run in the same drain |
| triggers | `Trgr`, 0x40-byte records [M 0x22cbe0] | `Trigger` (all fields below) |
| rules | `Rule`, 0x20-byte entries, built by `FUN_00205220` / `FUN_00209390` [M] | `Rule` + `RuleTest` (15 types) |
| mission manager | `DAT_0033a024` | `DriveScript` fields |
| references | argument words `0x7372_nnnn` name `sr` records of the `Map` window whose payload is a path, e.g. `CARP::<<Map>>::{Trgr}::47680` (a byte offset into `Trgr`) or `CARP::<<Map>>::{AIEl}::2304` [D] | `MapScript::refs`, `trigger_ref`, `ref_path` |

### The trigger record [M 0x22eb28 / 0x22e3a0 / 0x22eec0 / 0x22d240 / 0x22ca68]

| offset | field |
|---|---|
| +0x00 | base point x, y, z |
| +0x0c | radius |
| +0x10 | a byte none of the traced trigger code reads |
| +0x11 | shape: 1 oriented box, 2 sphere, 3 vertical cylinder; other values log 0x39af58 and never fire |
| +0x12 | flags (below) |
| +0x14 | height |
| +0x18 | `el` reference: the action list |
| +0x1c | runtime: the walk stamp (one test per walk) |
| +0x20 | box side axis; +0x2c box width |
| +0x30 | direction (start/reset heading, box forward axis, one-way direction); +0x3c box length |

Flags: bit 0 enabled (`EEnableTrigger` / `EDisableTrigger`), bit 1 once (cleared bit 0 after the
list ran, `FUN_0022c890`), bit 2 fired by car kind 1 = the player (spawned as type 0, which
`FUN_001862c0` maps to kind 1 [M 0x176658]), bit 3 car kind 2, bit 6 any other car kind, bit 9
the object class `FUN_0022d5a8` walks (and bit 10 is cleared on it at load [M 0x22cbe0]), bit 10
fires only while the trigger manager's live word is 1 (set at world init, cleared by
`EPlayerWin` / `EPlayerLose`), bit 11 one-way (`dot(+0x30, car velocity) >= 0`), bit 13 never
fires from the walks.

The car test `FUN_0022eb28`: reach = car `+0x7c` radius + trigger radius. Sphere: 3D distance
under the reach. Otherwise a plan-view distance under the reach, then cylinder: the car's
`y ± r` overlaps `[y, y + height]`; box: the car's own oriented box against the trigger's
(`FUN_001f5eb0`). **The port tests the car's bounding sphere against the trigger box**, not the
car's box. Which two components the plan-view distance uses is read from the two-term VU sum;
x and z are inferred.

### The rules [M 0x209318, 0x2096d0, the per-type constructors and tests]

Entry: `+0` type, `+4` bits (0 active, 1 once, 2 invert), `+8` stage (< 0: the default list),
`+0xc` the `el` list run when the rule fires, `+0x10..` the type payload. A rule fires when
active and `test() != invert`, runs its list (no current trigger: `FUN_002095f8` clears the
context block), then goes inactive if once.

The type numbers (`FUN_00209390` arm → constructor → descriptor `+4` typeinfo): 0 Always,
1 Ammo, 2 Damage, 3 Death, 4 Health, 5 PIP, 6 Prog, **7 Range, 8 Speed, 9 Timer, 10 Property,
11 Collision, 12 PlayerDir, 13 Difficulty**, 14 ProgCounter. (The older write-up
`drive-mission-scripts.md` named 7..13 in typeinfo-string order, which is not the switch order;
`RuleKind` is fixed.)

Selector (`FUN_00209738`): kind 0 the player, kind 4 an AI element (the payload word is an `sr`
reference to an `AIEl` record), kind 3 the AI statistics floats (`DAT_00327c30+0x1c`), any other
kind reads 0. Health = mgr `+0x378` (player) or the element's car `+0x4c`; speed = `|car +0x20|`
or element `+0x3c`; position = car `+0x10` or element `+0x30`.

| type | test |
|---|---|
| Always, PIP | true |
| Collision | false |
| Ammo | weapon slot count, crossing |
| Damage | health lost since the last test above the threshold (kind 3: statistic 2 above it) |
| Death | health (int) == 0 |
| Health, Speed | crossing |
| Range | distance to the player (int) below / above the threshold (level, not crossing) |
| Timer | a running timer's seconds, crossing (float) |
| Prog | programmer event set |
| Property | player: "NOBOOST", "ONEEMP", "2WHEELS"; element: `FUN_0012b138` |
| PlayerDir | element bearing inside [lo, hi] degrees (`FUN_0020abe0`) |
| Difficulty | mask bit 0/1/2 against difficulty 0/1/2 |
| ProgCounter | counter ==, >, <, & != 0, & == 0 |

"Crossing": with direction 0 it fires the frame the value goes below the threshold after being
at or above it; with direction 1 the frame it goes above after being at or below; the previous
value starts on the far side (threshold ∓ 1), so a value that starts past the threshold does not
fire.

### The frame [M 0x159d10]

The sim step is itself an event: cars, AI, the mission manager `FUN_00206b88` (its
`FUN_00206340` runs the default rules, then the current stage's, then current = next, only in
state 1), HUD, then the trigger walk `FUN_0022f0c0` (cars in slot order, then object classes,
then projectiles). The events all of that queued run after it in the same drain. So:
`EChangeStage` from a rule takes effect at the end of that frame's rule pass; from a trigger it
takes effect at the end of the next frame's. `ECallStage` runs the named bucket inside the drain.

### The mission manager

- State `+0x34c`: 0 loading, 1 playing (set at world init after the rules load
  [M 0x22faf8:127-129]), 2 paused, 3 won, 4 lost.
- Mailbox 0x100978 (the drive's result for the outer game): 3 while running [M 0x1feda0], 0
  won, 1 lost, 4 lost with no health; a 3 left at exit becomes 1 [M 0x174bf0:291].
- Ten timers (`+0x150`, 0x18 each): running, beep, start tick, value, rate, last second. Value
  now = value + rate × (now − start). `ESetTimer` value = secs × ticks/s + 1; `EStartTimer` sets
  the rate (−1 counts down) if not running; timer 1 beeps each whole second; `EStopTimer`
  freezes it.
- Programmer events `+0x24c` (64 flags, `SRuleProg`), set by game code (`FUN_00208540`, many
  callers) and cleared by `EClearProgrammerEvent`; counters `+0x3e4` with limits `+0x4e4`
  (6, 7, 10 → 6; 8 → 3; 9 → 2; 0x13 → 4; others none): `FUN_00206990` counts, sets the event at
  the limit, and counter 12 at 1/2 sets events 0xc/0xd, counter 14 at 1/2/3 sets 0xe/0xf/0x10.
- Objectives (a list, at most 7): add (first becomes current), pass (score 0 += points), set
  current.
- Health `+0x378`/`+0x37c` from `MSet` `+0x280`/`+0x27c` (10000 each when 0) [M 0x206ec0]; the
  difficulty and cheat scalers there are not ported.

### The ported actions

`EStartMission` (0x16), `EResetPlayerCarPos` (0x8c): reported as outputs (the car placement is
`trackstart`'s). `EPlayerWin` (0x2e): unless lost → state 3, live off, mailbox 0.
`EPlayerLose` (0x2d): unless won → state 4, live off, mailbox 1 or 4. `EEndMission` (0x13):
mailbox 0, state 3, exit request. `ECallStage` (0x28), `EChangeStage` (0x29), `ESetTimer`
(0x2f), `EStartTimer` (0x30), `EStopTimer` (0x31), `ETimerDrawOff/On` (0x104/0x105),
`EDisableTrigger` (0x35), `EEnableTrigger` (0x36), `EFireTriggerSpeedCondition` (0xab: fire when
the context car's speed is above, or with the flag below, the threshold), `EFireRandomTrigger`
(0xcb: the driving LCG, ×123456789, low 16 bits kept, percent from bits 8..23 against cumulative
odds), `ESetMagicCounter` (0xc8), `EIncMagicCounter` (0xb9), `EClearProgrammerEvent` (0xd5),
`EObjectiveAdd/Pass/SetCurrent/DisplayText` (0xe1/0xe4/0xe5/0xe7), `ESetObjective` (0x45, the
marker target), `EAddScoreAction` (0x83), `EPlayerSetImmunity` (0x10c). Every other id runs as
`Output::Effect` (camera, audio, AI, traffic, props, HUD look); the disc test prints each with
its handler.

## 2. The eight tracks [D, `tests/driving_script_disc.rs`]

| track | start | a fail | a win |
|---|---|---|---|
| MIS01 | T684 | player health gone (R0 Death) | T562 → stage 1; R1 waits for programmer event 0; timer 5 (3 s), stage 2; R37 below 0.95 s |
| MIS11 | T29 | T65 calls stage 39 with counter 0x2d < 3 (R11) | the same with counter 0x2d at 3 (R10) |
| MIS13A | T28 | health gone (R0) | counter 0xb at 3 (R16) → timer 0 (6 s); R37 stage 8 below 4 s; R15 below 1 s |
| MIS13B | T67 | health gone (R20 Health below 1) | T33 `EEndMission` |
| MIS13C | T16 | health gone (R0) | **not found**: R24 needs timer 6, started only by R9 in stage 3, and no list changes to or calls stage 3; R7 (timer 0) has its active bit clear |
| MIS3 | T214 | health gone (R68) | T244 `EPlayerWin` |
| MIS4 | T252 | health gone (R1) | T260 → stage 11; six AI elements die (R38..R43 count 0x2f); R52 at 6 → timer 2 (18 s); R37 below 1 s |
| RACE | T212 | the rival (kind-2 car) reaches T229 | T214 → timer 9 (5 s), stage 2; R9 below 1 s |

## 3. Not proven / not ported

- The timer clock `DAT_00339f54` advancing once per sim step and `DAT_00339f4c` = 60: inferred
  from the 0.016666668 s step [M 0x159d10:28]; their writers are not in the decompile.
- The order among several triggers hit in one frame (the game walks grid cells).
- The box trigger test against the car's own box (the port uses the car's sphere); box flag
  0x1000 (the up axis from the two axes) is not ported.
- The walks for the object classes (`FUN_0022dae8`, bits 4/7), the bit-9 class (`FUN_0022d5a8`)
  and projectiles (`FUN_0022def0`).
- The `el` lists other records own (AI elements, `AICo`): `run_actions` runs one, but when the
  game runs them is not traced. Who sets programmer event 0 in MIS01 and counts 0xb in MIS13A is
  not found; the disc test sets them directly. How MIS13C's drive ends is not found.
- `SRulePlayerDir`'s angle and `SRuleProperty` for elements are inputs (`ElementFacts`).
- The RNG's seeding and the other systems that draw from it (traffic, AI) before a random
  trigger.
- `EPlayerLose`'s restart debug path and the cheat gate (`DAT_0033a008`) are taken as off; the
  health difficulty scalers are not ported.
- The view (`crates/nf-view/src/driving.rs`) still runs the old channel model; switching it to
  `Mission::from_map_script` + `step_facts` is a caller change outside this port.
