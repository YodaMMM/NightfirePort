# 09 security-room alarm (security-room-alarm-09)

Why the 07000009 security guards never raise the alarm on Bond in the port (playtest 2026-10-17,
07000009: Bond at the elevator panel — the game raises ch 0x61 within seconds; the port ran 900
ticks with no alarm and no fail).

## The census (port probes, disc data)

Six 07000009 drones (type 0x0f), all skin kind 0x27/0x28 (0x05000027 / 0x05000028), weapon 1
(unarmed — the weapon id ≤ 1 that `DroneFunc_AttackNoWeapon` 0x14af80 treats as no gun), team 3,
mode 9 (start state 0x14 CivilianPatrol), anim set 0xb, sight range 12.0, half-FOV π/2
(`NDrone2_DefaultInit` 0x14b300 LAB_0014bb44 sets +0xE4 = 0x3fc90fdb for every non-blind mode;
the port's hardcoded HALF_PI matches). All six carry behaviour props 0x1a and 0x56
(`DoModeSettingsNEW` sets 0x1A and 0x56 — the alarm-raiser — on 07000009/0A [M, cited at the
port's `init_properties`]) and prop 0x3c; alt_state +0x5A4 = 0x89 (the alert block's RunToAlarm).

Patrol: path-0 nav loop (nodes 0–8) through the security room — door nodes 5/6 at (−27.2, 36.6)
/ (−27.3, 38.0), room interior x −21..−25 z 34..40. Alarm infrastructure [D placements/AI points]:
11 kind-8 AI points; the security room's own at (−23.07, −369.42, 36.90) yaw 0; the switch beside
it at (−23.06, −368.05, 37.20) on ch 0x7D (125). ch 0x61 = OR mux #278 (ch 0x7C, ch 0x7D) [D];
the standing monitor hook turns ch 0x61 into FailLabel 0x4000032 "Alarm Raised" + ch 99 [M
0x1827d8].

## The game's chain (all [M] unless tagged)

1. `NDrone2_NearArmedPerson` 0x1787c8: on 07000009/0A — behaviour prop 0x3c AND the sight flag
   (+0x228 & 4) — **no radius (the caller's 6.0 arg is dead) and no armed check** (the
   `OpponentHasWeapon` gate runs only on the other levels; the voice-0xD check is skipped too).
2. `NDrone2_DSTATE_CivilianPatrol` 0x15d610 case 3 (UPDATE): `UpdatePatrolRoute`, then
   NearArmedPerson → `+0x4F8 |= 0x8000000` (own sight); prop 0x33 once-broadcast
   (`+0x4FC |= 1`, `DroneAlertToObject(20, 20, 0.75, msg 0x11, Bond)`, `+0x4F8 |= 0x200000`);
   voice 0x10 → `+0x44 = 1` then SetState(0x56) [not our guards: voice 0x17]; else female
   PlaySFX(0x48); **SetState(0x56 Attack)** in every arm.
3. `NDrone2_DSTATE_Attack` 0x164210 case 1 (ENTER): `DroneFunc_FirstAttack` 0x148010 →
   (own-sight bit set) `DroneFunc_FirstSightState` 0x147c48: prop 0x43 surrender arm first, then
   **prop 0x1A → return 0x3E on 07000009/0A** (0x3C Interogate on 07/41) — before
   `SeenPlayerTalk` and the 0x1B/0x18/0x19 combat-move rolls. FirstAttack stamps the first-attack
   bits, alertness 1.0, sight range raised to combat range, half-FOV 0.
4. `NDrone2_DSTATE_CivilianChallenge` 0x162558 (state 0x3E, `NDrone2_StateFuncs` EE 0x29b300
   [L]): ENTER — `+0x4F8 |= 0x10000`, `+0x4FC |= 0x20`, **`CallAnim(0, 0x3E, follow 0x89)`** +
   the challenge bark `SpeechSFX(0xC3 + Rand(9), 0, 0, 0)`; msg 3 faces the opponent
   (`SetAngleToObj`); bullets/explosive resume 0x89. Anim 0x3E done → **state 0x89
   RunToAlarm**.
5. `NDrone2_DSTATE_RunToAlarm` 0x1736f8 → `NDrone2_FindAlarmPoint` 0x153008 (the closest kind-8
   AI point) → run → `NDrone2_DSTATE_PressAlarm` 0x168ea8 →
   `DroneFunc_SetMissionFailReason(0x10)` 0x13e560 case 0x10: `Switch_GetNearest` 0x1b0398 →
   `switch_channels[ch] = 1` — the security-room switch's ch 0x7D → mux #278 → **ch 0x61** → the
   fail objective → the Mission Failed menu.

## The port's divergences (all fixed here)

- `CIVILIAN_PATROL` UPDATE sent the guard to `CIVILIAN_SCARED` (→ frozen `CIVILIAN_HIDING`)
  instead of **ATTACK** [M 0x15d610 case 3]. The guard reached the room at ~t2400, latched seen,
  and cowered forever — no alarm, no fail.
- `near_armed_person` gated on `bond_armed` on 09/0A where the game does not [M 0x1787c8].
- The `ATTACK` ENTER FirstAttack port lacked FirstSightState's prop-0x1A arm (the 09/0A
  CivilianChallenge divert) [M 0x147c48:142-153 + 0x148010].
- State 0x3E CivilianChallenge was not ported [M 0x162558].

## Live checks for the main agent

- `tools/trace/fieldprobe.py`: place Bond at the panel (−18.1, −367.4, 26.1) in a session where
  the guards have patrolled ~1 min, and watch the closest guard's DSTATE (+0x5A0) walk
  0x14 → 0x56 → 0x3E → 0x89 → 0x8A and ch 0x7D rise at the press. Expected: the challenge anim
  (0x3E) plays with a "Halt!"-style bark (voice lines 0xC3..0xCB) before the run.
- gdbwatch.py on `switch_channels[0x7D]` (EE 0x2a04?? per the channel base) during the same
  window: the only writer should be `SetMissionFailReason` case 0x10 (0x13e560).

## Tests

`crates/nf-view/tests/security_alarm_09_live.rs` (ISO-gated): the repro — Bond at the panel →
the guarding drone passes 0x3E → 0x89 → 0x8A and ch 0x7D → 0x61 → ch 99 → objective 1 failed;
sister — Bond at the game's own start for the same window: no guard ever latches seen and no
alarm channel fires.
