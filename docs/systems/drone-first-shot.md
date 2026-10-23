# The guard's first-shot timeline (register → first bullet)

Symptom: guards fired almost instantly, before the starting guard could even be stunned.
The chain below is the game's own, from the tick a guard first registers Bond to the tick his
first bullet spawns, with the port's status per step. Evidence tags: [M] decompile (+address),
[D] disc bytes, [L] a live measurement (fresh-boot PCSX2 or the repo's game traces), [S] suggested.

## 0. Register: sight or hearing

- **Sight** — `DroneVision_ProcessDroneSight` 0x176a28 (the global per-tick pass): on a tick where
  the opponent is set (+0x170) and flag 2 (this tick's visibility) holds, `+0x274 = 0`, `+0x270 += 1`,
  then `HaveOpponentSight` [M 0x176a28:221-223, disasm 0x176de8]. Visibility itself is
  `DroneVision_OpponentVisibility` 0x175af0: range-scaled `0.7 − (dist − r)/r` with
  `r = sight_range·(2·alertness+1)`, the view-cone sine (half_fov + 0.3926991 pad), and the
  AI-volume light factor `+0x22c`; at `dist < 2.0` the cone is replaced by a `CanSeeObject` RAY
  test (a touching Bond is felt even behind the guard) [M 0x175af0:22-34].
- **The ack** — `DroneVision_HaveOpponentSight` 0x175ef0: returns until
  `+0x270 >= DroneFunc_ReactionTime`; then flag 4 (+0x228 |= 4), `LogSeenOpponent`, the opponent's
  position latched (+0x250..) [M 0x175ef0:23-31].
- **ReactionTime** — `DroneFunc_ReactionTime` 0x14ac60: the default arm `FRAME_RATE_DIV·10` = **10
  ticks on 07000001**; levels 2/3/5/9/a scale `(1−alertness)·30·0.01·(200−level)` (59 at alertness 0
  on 05) [M 0x14ac60:15-24].
- **Hearing** — the sound pushes (`HandleSoundAlerts` 0x148a30 → the alert broadcast) reach a
  scripted guard too: `DSTATE_PlayScript` cases 0x11/0x12/0x14/0x15/0x16/0x1e run
  `DroneVision_EnemyAlerts` and a non-zero result leaves the script via `SetState` [M
  0x15ba68:180-195]; the alarm (0x1e) arm is Attack [M 0x177c18 case 0x1e]. Non-scripted guards:
  the same `EnemyAlerts` from their state's msg arms; a heard-noise chain (0xa2-0xa5) for the
  share > 0.2 / alertness > 0.66 cases.

## 1. The script hold (the step the port got wrong)

`DroneVision_EnemyLookForOpponent` 0x177ed0 — the sight→Attack interrupt shared by Idle/Alert/
Patrol/PlayScript/SniperIdle/... — opens with a gate the port was missing:

- **`if (current AI state == PlayScript(3) && alertness >= 1.0) return 0`** [M 0x177ed0:12-14] — a
  FULLY-ALERT scripted guard never sight-interrupts; his start script plays to its end. Only a
  scripted guard at alertness < 1.0 can sight-interrupt (then the normal `alert ≥ 0.66 || prop 0x51`
  gate applies [M 0x177ed0:41-46]).
- 07000001's starting guard (drone 0, placement #158): behaviour prop 0x20 = 3 → alertness 1.0 at
  creation (the float table {0, 0, 0.66, 1.0} [D ee_start_07000001.bin @ 0x2f5cc8]; the behaviour
  words reach the drone through `behaviour_util_get` 0x1c7a98's structured block: header
  params[15..17], normal props params[18..20], alert props params[23..25] [M 0x1c7a98:44-66]) —
  so he holds state 3 through his whole scripted walk.
- **[L] tests/traces/01-doors.trace lines 0-150 and 01-raptor.trace lines 0-167**: the guard stays
  in state 3 while the player walks from the spawn to 5.2-5.4 u in his view cone; both recordings
  show the first state change (0x68) exactly at the script's end, not at the sight ack.
- The scripted walk itself: script 0x06000169 walks him ~6.8 u over its 181 frames [D nf-extract
  script-info; L 01-doors: the guard drifts x −4.919 → −7.17 across lines 0-150].

Port status: **was missing the gate** — the guard interrupted at the ack (control +17) and his
first bullet landed at control +20. Fixed (drone.rs `look_for_opponent`), plus the
missing PlayScript sound/alarm arms [M 0x15ba68:180-195] that the gunfire path needs (the cover
suite's 06 guard pins them).

## 2. The script's own end → the aim stance

`DSTATE_PlayScript` case 5 (the script-complete msg 5): for start scripts 0x6000168/0x6000169 →
`CallAnim(0, 10, 0, 0x68, 0, 0)` + AttackTalk — the guard finishes his walk INTO AimStandFire
(0x68) [M 0x15ba68 case 5]. Other scripts → `SetState(+0x5a2)` (the start state). Port: ported
(drone.rs `(PLAY_SCRIPT, msg::ANIM_DONE)`).

## 3. The aim/fire state and the fire gate

From 0x68 the game's chain [M]:

- `DSTATE_AimStandFire` 0x166328 case 1 (ENTER): `AlertStatusSet(1)`; if not stays-put, the aim
  anim (state 10) + vision flag; **`if (+0x274 < 0x10) DroneWeap_Fire(1)`** — the entry fire with
  fresh LOS — else → 0x5c (CombatNoSight) [M 0x166328 case 1].
- `DoFiring` 0x17a080: the sight gate `+0x274 < 0x10 || prop 0x17` [M 0x17a080:24-25]; the fire
  request is `+0x3b` (armed here and by CombatState's flag-4 arms [M 0x148338:108-113]).
- `DroneWeap_Ready2Fire` 0x178c48 (SP): the anim class must be 2 (or 4), no blend-out (+0x538);
  **`if (dist < 2.0) return 1`** — a touching target is always "ready" (no muzzle test) [M
  0x178c48 SP arm]; otherwise the muzzle-vs-aim direction test: the muzzle bone's direction vs
  the aim vector within 0.349 rad (0.5236 for dtype 13) [M 0x178c48:171-190].
- `DroneWeap_NextBulletTime` 0x179d40: the burst schedule — `+0xBC4 = max(+0xBC8 + BurstDelay +
  interval, now)`; the interval from the weapon's +0x2c, the first-shot delay from the weapon's
  +0x40 [M 0x179d40:100-155]. The port's `next_bullet_time` models this.
- The first bullet's travel: `Bullet_init` from the muzzle with the accuracy roll [M 0x178f68..].

**The measured game chains** [L docs/live/2026-10-14-weapon-measurements-pcsx2.md]:
- A fresh-sight 06 guard (the alert → the first bullet): **+97 frames** at 2.8 u (walking into him
  from behind), another +45 frames at 16.7 u — the reaction + the state/anim chain hold the bullet
  well past the 10-tick reaction.
- The stunner contact (05 gate-trench guard): the guard reacts (0x72 — alerted by the contact),
  turns toward Bond, and hits him WHILE the tase drains (~1.5 s of contact to stun; the drain
  ~32/s [L the same doc]).
- The starting guard's scripted walk (01): the player has control for the whole 181-frame script
  (~3 s) before the guard's first combat state — the stun window [L 01-doors.trace].

## 4. The stun race (Bond touching the guard's back with the stunner)

The tase is silent (the fire script has no gunshot — the guards hear only the real sound plays
[M AnimProcessScriptCmds 0x1c1b38 op 1; Sound_Alertness 0x1ddc48]; the port's hearing fix is in
tase_silent_live's header). At contact (dist < 2) the guard's visibility is the ray test → 1.0
[M 0x175af0:22-34], so the LOS starts at the touch; the ack 10 ticks later (01's reaction);
the 0x68 enter fires with fresh LOS [M 0x166328]. The tase accumulator drops the guard at
2·health [M 0x1469b8:126-130]. Port status: the race itself (the impact-reaction chain the game
runs on the tase hit — AimStandFire's impact arms → 0x72) is **not fully ported yet**; see the
report's "what's left".
