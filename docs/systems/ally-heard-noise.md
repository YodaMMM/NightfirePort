# Ally heard-noise combat: the team-2 opponent model (FindOpponent's drone arm)

Symptom: Zoe fired at Bond at the gondola (07000008). The port's drone combat
aimed every drone at Bond and had no opponent concept, so the dtype-0xC ally (Zoe, team 2
[M 0x14cbd8:57-63 — `switch(dtype) { 10|0xb|0xd|0xe → 3, 0xc → 2, default → 1 }` → `+0x44`])
entered the heard-noise attack chain and fired at Bond, then thrashed CombatNoMove 0x59 ↔
CombatNoSight 0x5c every tick. This document is the game's rule set; the port folds follow it.

## 1. The opponent pointer (`Drone+0x170`), picked per tick

`NDrone2_FindOpponent(Drone*)` 0x1424e0 runs every tick from `NDrone2_ControlSTANDARD`
0x1490f0:167 (after the team counters, before `DoTracking`). SP (`MPSettings+0x388 == 0`):

- **Team 2 never takes the player.** The player set (`NDrone2_SetOpponent(drone, player)`,
  the PlrStat/dead/in-vehicle gates) sits behind `if (*(char *)(drone + 0x44) != 2)`
  [M 0x1424e0:101-121]. A team-2 drone falls through to the object walks:
  - a walk of the control objects for class 0x3b whose short `+0xf4` is in (0,4) with the
    direction pitch in [−0.5235988, +0.17453294] [M 0x1424e0:124-146] — *not ported*: the port
    has no per-object class model for that kind; the gondola repro never involves it;
  - re-validation of the current opponent [M 0x1424e0:148-190]: `NDrone2_CanSeeObject(…, 5, 0)`
    against `+0x170`; a drone-class opponent additionally re-checks the alive filter
    (+0x4f8 & 0x600 == 0, & 0x100, health +0xac > 0, obj class ≠ 0x11, +0xfe & 1 == 0)
    [M 0x1424e0:155-166]; fail → `SetOpponent(0)` and the scan;
  - the drone-list scan [M 0x1424e0:210-268]: from the rotating cursor `+0x2c0` (start = the
    node after it), skipping self and the current opponent's node, the first node whose drone
    is team 1 (`+0x44 == 1`), passes the same alive filter, and is within the squared range —
    800² (640000.0) when there was no opponent to re-validate, `+0x1a0²` (the engage range)
    when there was [M 0x1424e0:191-209, 251-255] — and whose `NDrone2_CanSeeObject(…, 5, 0)`
    passes, wins `SetOpponent`. ONE candidate per tick reaches the LOS check: the per-frame
    sight counter `DAT_00299092` (bumped by every `CanSeeObjectFrom` LOS call
    [M 0x176f18:53]) against the value captured at FindOpponent's entry stops the walk
    [M 0x1424e0:259-262]; the cursor remembers where it stopped.

`NDrone2_CanSeeObject(drone, obj, 5, 0)` 0x1785e0 → `DroneVision_CanSeeObjectFrom`
0x176f18: for class-2/3 targets the point is `DroneFunc_GetBonePos(obj, 5)` (the head bone)
clamped to the target's collision cylinder, then `Collide_LineOfSight` (seer head → target
bone) [M 0x176f18:26-46]. No cone, no range inside — the callers range-gate.

`NDrone2_SetOpponent` 0x1422d0: writes `+0x170`, zeroes the aim offset `+0x210` (0xff target
id) when the opponent is cleared [M :25-29], and for class-2/3 opponents calls
`NDrone2_SetOpponentAimPos` 0x141fe8: the aim offset `+0x210` = the opponent's bone-2 world
position (the anim datum's bone live gate [M :62-81], else the object position); the aim
position `+0x1f0` = opponent pos + `+0x210` [M 0x1422d0:70-73].

## 2. Where the ally's shots aim

`DroneWeap_NextBulletTime` 0x179d40:130 re-runs `SetOpponentAimPos` before every shot — the
shot's aim is the OPPONENT's chest, i.e. for a team-2 ally with a team-1 drone opponent, the
GUARD's bone-2 position, never Bond's. The port's `fire_weapon` fold aims at `self.opp_target`
(the port's `+0x1f0`), which `opponent_info` refreshes per tick from the opponent — Bond for
team 1, the picked guard for team 2, and (with no opponent) the game's `SetOpponent(0)` zero
offset [M 0x1422d0:25-29]. The port's drone-opponent chest fold is the opponent record's
position (the port's body-cylinder centre); the bone-2 world position is not modelled
([S] fold, the bullet's cylinder hit test is the same volume).

## 3. The sight state is opponent-generic

`DroneVision_ProcessDroneSight` 0x176a28 runs per tick per drone against `+0x170`:
`+0x274` (the port's `unseen_ticks`) resets to 0 on every opponent-visible tick
[M 0x176a28:216-223 + disasm 0x176de8], and `DroneVision_HaveOpponentSight` 0x175ef0 sets the
seen flag (+0x228 bit 2) behind the reaction-time gate and stamps `+0x238 = now` (the
had-opponent stamp) [M 0x175ef0:40-44, 66-68]. `DroneVision_EnemyLookForOpponent` 0x177ed0
(the sight→Attack router used by the HeardNoise*/Investigate UPDATE arms):
- `+0x170 == 0` → 0 — **no opponent, no sight routing** [M 0x177ed0:18-20];
- prop 0x3c and the weapon bit (+0x228 & 4) gates [M :21-28];
- the alert < 0.66 / prop-0x51 gate sits INSIDE the opponent-is-player arm [M :40-51] — a
  drone opponent needs no alertness;
- team 3 → 0 on every level except 0x07000009/0x0700000a [M :29-38];
- `+0x4f8 & 0x800` → 0; `+0x4fc & 0x20` set → 0; else Attack 0x56 with the
  0x4000000/0x8000000 first-time stamps [M :52-83].

## 4. The heard-noise chain still enters Attack — the combat states then decide

`NDrone2_DSTATE_HeardNoiseSuspect` 0x16b0e0 case 0x14 (a further sound): `+0x4f8 |= 0x4000000
| 0x10000000`, `SetState(0x56)` — unconditional [M :118-124]. `HeardNoiseAlert` 0x16b540 case
0xc (the 60-tick timer): `SetState(0x56)` — unconditional [M :78-82]. The UPDATE arms call
`EnemyLookForOpponent` [M 0x16b0e0:95-99, 0x16b540:56-61]. So the ally DOES enter Attack
0x56 when the guards fight; what she does there follows the opponent.

`NDrone2_DSTATE_Attack` 0x164210 ENTER: `FirstAttack` → `AttackNoWeapon` → alertness 1.0,
`+0x4f8 |= 0x400000`, and (mode ∉ {2, 0x19}) vision on, `InvalidateAttackRoute`, then prop 0x18
with `+0xf0 <= +0x1a0` → Combat 0x58 (the +0xf0 distance is opponent-derived), else
`DroneInSights` fail → `+0x5a2 = 0x68`, AimStand 0x67 [M :24-90].

`DroneFunc_CombatState` 0x148338 — the combat router every combat state's UPDATE calls:
- **the no-opponent gate first**: `+0x170 == 0` and (`+0x238 == 0` or `+0x238 + 30 < now`) →
  **return 0x60 NoOpponent** [M :20-31; the grace semantics per `NDrone2_HasOpponent`
  0x14ad58:7-15];
- `DroneAnim_InTransition` → 0 [M :32-35];
- from Combat 0x58 (case 0): stationary (+0x4f8 & 0x10) or prop 0x40 → 0x59; in range
  (+0xf0 <= +0x1a0) → 0x5a; unseen > 15 → 0x5c else 0x5f; weapon bit → 0x5b [M :54-72];
- from CombatNoSight 0x5c (case 4): stationary/prop 0x40 → 0x59; cover → 0x8c; weapon bit →
  0x5b; unseen > 15 → the MoveToObject result (move anims / 0x5f / 0x7f); else aim
  [M :135-180].

`NDrone2_DSTATE_CombatNoMove` 0x1643e8 (state 0x59):
- ENTER [M :18-42]: `AlertStatusSet(1)`, mode 2 → SniperFire 0x2b; else the aim-stand anim 10,
  vision on, `AttackTalk`, then `DroneWeap_Fire(1)` — except anim-set 0x13 skins, whose first
  shot defers one frame via timer C with `+0x3b` cleared;
- UPDATE [M :49-101]: `SeenAndAttacking`; movable (not +0x10 and not prop 0x40) →
  **SetState(0x68 AimStand)**; else turn to the opponent (`DroneMove_TurnToObj` 0.525),
  `CanAltAttack` → 0x72, `NDrone2_HasOpponent(0x72)` false → **SetState(0x60 NoOpponent)**,
  else `+0x3b = seen ? (+0x1e == 0) : 0` and the fire/PlayFiringAnim arms, with the anim-step
  0xc `Rand(3) == 0` → 0x6e roll. **There is no 0x5b/0x5c exit** — the port's old
  `unseen < 16 → 0x5b else 0x5c` UPDATE was the 0x59 ↔ 0x5c thrash (with
  `DroneFunc_CombatState`'s stationary arm sending 0x5c back to 0x59 every tick).

`NDrone2_DSTATE_NoOpponent` 0x1650a0 (state 0x60): ENTER `AlertStatusSet(0)` + the look-about
anim 0x21 + timer C = 60 ticks [M :15-28]; UPDATE and TIMER_C return to Attack 0x56 the moment
`+0x170 != 0` [M :30-48].

## 5. The port's folds

- Per tick, before the opponent-info pass, each present team-2 drone runs the game's team-2
  arm: re-validate the held opponent (head-to-head LOS through the port's collision world —
  the documented stand-in for `AINetwork_TestRayCels`+`Collide_LineOfSight`), then the
  rotating one-LOS-per-tick scan (skip self and the held opponent; team 1, alive
  (present/enabled/not corpse), 2D within the range — unlimited (800²) on acquisition,
  the engage range when switching) [M 0x1424e0 as above]. The picked guard's position feeds
  `dist`/`yaw_to_opp`/`opp_target`; no pick clears them (`SetOpponent(0)`).
- The sight pass overwrites the team-2 drones' opponent-derived sight state afterwards:
  opponent + LOS → unseen 0, the reaction-ack seen flag, the `+0x238` stamp; opponent without
  LOS → unseen ticks up; no opponent → unseen ticks forced ≥ 16 (every `+0x274` gate refuses,
  `DoFiring` 0x17a080:24-25 included — the shot can never leave).
- `combat_state` opens with the 0x60 gate [M 0x148338:20-31] (team 1 keeps its implicit
  player opponent — Bond alive — exactly as before); Attack 0x56's prop-0x18 arm requires the
  opponent-derived distance [M 0x164210:75]; CombatNoMove's UPDATE takes the game's exits
  [M 0x1643e8:49-101] (the anim-step 0xc → 0x6e roll is not ported — the port's anim steps are
  not exposed to the state arms [S]); NoOpponent 0x60 is ported [M 0x1650a0].
- `look_for_opponent` (the `EnemyLookForOpponent` fold) lets a team-2 drone with a visible
  drone opponent sight-enter Attack (prop 0x3c, seen, the 0x20 alert-bit gate) and keeps
  refusing team 2 without one and team 3 (the 09/0a exception is not ported — existing gap).
- Not ported (documented gaps): FindOpponent's class-0x3b control-object arm (no object-class
  model); the 0x4f8 & 0x800 gate; MoveToObject routed at the drone opponent (the port's
  combat route stays Bond-node-keyed — a stationary ally at the gondola mouth never walks it;
  the guards' own chase is untouched).

## Live checks (open)

- `gdbwatch` on Drone #21 (07000008, the gondola mouth) `+0x170` during the guard fight: the
  expectation is the pointer to a team-1 guard's obj while he is within range and visible,
  0 when he is not; `+0x1f0` follows the guard's chest, never the player.
- `fieldprobe` the ally's `+0x59` state byte through the ride: the expectation is the
  heard-noise chain → 0x56 → 0x58 → 0x60 (no 0x59/0x5c thrash) with the guards fighting Bond.
