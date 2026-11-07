# guard-dead-body-shout — the dead-body and surrender shouts

2026-11-07, branch `s/guard-dead-body-shout` (from origin/master 22d5836b7).

The task: `react_to_drone_alert_msg` (`crates/nf-game/src/mp/bots.rs`, the SetState tail of
`NDrone2_ReactToDroneAlertMsg` 0x1437a0 for alerts 0x15/0x13) is called only by
guard_ai_other4_live.rs; find whether anything moves a single-player guard into
`NDrone2_DSTATE_SeenDeadBody` 0x16acd0 (0x9e) or `NDrone2_DSTATE_SeenSurrenderedDrone` 0x16ade0
(0x9f), audit the receivers' gates, then delete the unused copy or wire the shared rule in.

## The finding: 0x9e/0x9f are unreachable in the disc

**Nobody — single-player guard or MP bot — can ever enter 0x9e or 0x9f in this build.** Both the
sender side and every receiver path check out; the two arms that compute those state ids gate
their only possible callers away.

### Who raises the shouts (every caller read)

- **Alert 0x15 (dead body)** — `NDrone2_DSTATE_Dead` 0x163fe8, twice [M]:
  - the ENTER arm (case 1): `behaviour_util_getProperty(0x31, Drone+0x4D8)` set and the drone's
    `+0x3A` clear → `+0x4FC |= 8` → `NDrone2_DroneAlertToObject(0, 0, 1.0, dcv, 0x15, own obj)`
    [M 0x163fe8 case 1];
  - the msg 0xd arm: the same prop/`+0x3A` gates → the same broadcast [M 0x163fe8 case 0xd].
  - Also `NDrone2_DSTATE_BotDead` 0x171518 broadcasts 0x15 twice (the MP bot death) [M].
- **Alert 0x13 (surrender)** — `NDrone2_DSTATE_Surrender_Anim` 0x1626a0, once: its msg 0xc arm
  sets `+0x4FC |= 4` and calls `DroneAlertToObject(0, 0, 1.0, dcv, 0x13, own obj)`, then arms the
  `+0xCC4` timer at now + 15/60 s [M 0x1626a0 case 0xc] (the task's ":74" line).
- No other senders: the remaining `Drone_SM_BroadcastMsg` callers send 0x11 (FirstSightState
  0x147c48), 0x12 (SendHurtMessage 0x145e28, ExplosiveImpact 0x146480) and 0x16 (SendAttackMessage
  0x14a410); the `Drone_Message`/`MessageObjVicinity` callers carry 0xe/0x10/0x18/0x1a/0x1e/0x1f/0x22
  — none send 0x13/0x15 [M, every caller read].

`NDrone2_DroneAlertToObject` 0x14a538 / `NDrone2_DroneAlertToPosition` 0x14a498 fill the global
alert block at 0x2990a0 (the sender obj, the position words, `+0x20` from the obj), run
`NDrone2_SetupAlertTarget` 0x155d48 and call `Drone_SM_BroadcastMsg(alert, block, 0x1e, sender_id)`
0x175aa8, which builds a MsgObject with the target field 0 and hands it to `Drone_SM_RouteMsg`
0x172128 → every drone's current state handler, skipping receivers with id `+0x10C` in
{0xf, 1, 3, 0x47, 0x48} [M 0x172128].

### The SP receiver path discards exactly these two shouts

The SP states' alert arm (Idle 0x15bf18, Patrol 0x15c498, Alert 0x15c248, SearchArea 0x165bf0,
AlertToPosition 0x165270, HeardNoise* 0x16af20/0x16b0e0/0x16b540, EnemyMission 0x15e1e8, …) calls
`DroneVision_EnemyAlerts` 0x177c18, which:

1. returns 0 when the receiver's `+0x4F8 & 0x800000` (fleeing) is set [M 0x177c18:14-17];
2. saves the resume state `+0x5A0` (the SM's current state), the home `+0x5B0` (own feet) and the
   cel `+0x5C0` (own `obj+0x20`) when `+0x33`/`+0x2B` are clear [M 0x177c18:20-33];
3. gates the shout ids on the receiver's properties — 0x13 → prop 0x4F, 0x15 → prop 0x29, 0x11 →
   0x28, 0x12 → 0x27, 0x16 → none — and forwards to `DroneVision_ShoutFromOtherDrone` 0x177a30
   [M 0x177c18:41-77];
4. its 0x14 case goes to `DroneVision_AlertSound` 0x177800 and its 0x1E case is the MP alarm arm —
   neither reaches the shout tail [M 0x177c18:78-118].

`DroneVision_ShoutFromOtherDrone` 0x177a30 is where the decompiler gives up: 23 "removing
unreachable block" warnings and an empty `return 0`. **The disassembly
(tools/crosscheck/mipsdis.py) shows it is not empty**: it resolves the alert block's sender (the
sender obj must be a drone, `obj+0xFF == 2` → its DCVars at `obj+0xE0`; missing → return 0)
[M 0x177a5c-a8], computes `Vec_Dist3D` between the two feet (`obj+0x30`) [M 0x177a9c-a0], folds
the `NDrone2_InShoutingRange` 0x1788a0 law inline, and then jumps through a 6-entry alert-id
table at **0x2FB6C0** [D ACTION.ELF bytes]:

| alert | table entry | target | effect |
|---|---|---|---|
| 0x11 | 0x2FB6C0 | 0x177b80 | mode-0x1E receivers → the MP arm; else the Attack arm |
| 0x12 | 0x2FB6C4 | 0x177b80 | same |
| **0x13** | 0x2FB6C8 | **0x177BF4** | **return 0 — no reaction** |
| 0x14 | 0x2FB6CC | 0x177BC4 | the Attack arm (unreachable from EnemyAlerts) |
| **0x15** | 0x2FB6D0 | **0x177BF4** | **return 0 — no reaction** |
| 0x16 | 0x2FB6D4 | 0x177b80 | same as 0x11 |

The Attack arm sets the receiver's `+0x4F8 |= 0x4000000` (the sighting bit `DroneVision_AlertSound`
0x177800:48-50 reads) and `+0x4F8 |= 0x20000000`, and returns 0x56 — the caller stores it into the
SM's pending-state slot `+0x1C` → Attack [M 0x177bc8-bf0; Idle 0x15bf18's alert case]. The mode-0x1E
arm instead checks `MP_areObjectsOnSameTeam` and calls `BOT_reactToDroneAlertMsg` 0x1263c0 → 0xCF
[M 0x177b80-bc0].

So in single-player **the dead-body and surrender shouts are delivered, gated — and then thrown
away**: after the fleeing gate, the property gate and the shouting range, the jumptable returns 0
for exactly 0x13 and 0x15. A guard never goes to a body's alert position, and a surrendering
guard's shout turns nobody. The old doc line "all shouts are dead code" was too broad (0x11/0x12/0x16
do attack) — `docs/systems/drone-alerts.md` is corrected on this branch.

### The bots' path drops the same two alerts

`NDrone2_ReactToDroneAlertMsg` 0x1437a0 is referenced only by `NDrone2_DSTATE_BotGlobal` 0x16E060,
whose alert arm (state type not in {0,1,2,4,10}) forwards alerts 0x11..0x16/0x1E with no override
[M 0x16e060]. For a bot receiver the mode byte `+0xC5` is 0x1E, and:

- the 0x15 arm: prop 0x29 → fleeing gate → `InShoutingRange` → `+0x50C = 1.0` → **mode 0x1E →
  return 0** [M 0x1437a0:298-326, the drop at :320-321];
- the 0x13 arm: `InShoutingRange` or fleeing → prop 0x4F → **`case 0x1e` → the return-0
  LAB_00143e48** [M 0x1437a0:267-292].

The arms that DO land bots somewhere are live (0x11/0x12/0x16 → 0x56/0xA0 via the break tail,
0x14 → `BOT_reactToDroneAlertMsg` → 0xEE, alarm 0x1E → same-team → 0xCF) — but for 0x15/0x13 the
only computor of 0x9E/0x9F refuses its only callers. Nothing else writes those ids: the grep over
the decompile finds 0x9E/0x9F constants only in `HandleTalking` 0x13f688 / `CivilianScaredTalk`
0x13f900 (speech ids, not states) and `ReactToDroneAlertMsg` itself; the state table still maps
them [D 0x29b300+0x278 = 0x16acd0, +0x27C = 0x16ade0], but no code reaches a SetState with them.

### The shouting-range law (shared by both paths)

`NDrone2_InShoutingRange` 0x1788a0 (called at 0x1437a0:181/:227; inlined in 0x177a30): the 3D
feet distance (`obj+0x30`) against two bounds picked by the level id global at 0x2A003774 [M
0x1788a0:14-25] — level 0x0700000C..=0x0700000F → 40.0 then 100.0, every other level → 15.0 then
20.0. Inside the first bound always in range; inside the second only when
`*(sender obj+0x20) == *(receiver obj+0x20)` (the cel word); else out [M 0x1788a0:30-44, the f32
`c.lt.s` compares]. This matches the note already in mp/bots.rs (BOT_GLOBAL's entry comment).

### What the port has and lacks

| piece | game | port on this branch |
|---|---|---|
| 0x15 sender (Death ENTER/0xd arms) | 0x163fe8, prop 0x31 + `+0x3A` gates | not modelled (drone.rs "Prop 0x31 … not modelled (G-026)") |
| 0x13 sender (Surrender_Anim msg 0xc) | 0x1626a0 | not modelled (the port has SURRENDER_ANIM 0x3f/SURRENDERED 0x40 states; they send nothing) |
| broadcast delivery | 0x14a538 → 0x175aa8 → 0x172128 | none (no alert outbox/inbox; the Phase-4 "shout outbox" is still a plan) |
| SP receiver gates (fleeing, props 0x29/0x4F, resume/home save) | 0x177c18 | `enemy_alerts` handles only msg 0x14 (sound) / 0x1E (alarm); shouts never reach it — no sender exists |
| shout range law | 0x1788a0 / inline 0x177a30 | **ported now**: `mp::bots::in_shouting_range` (pure, test-pinned) |
| SP shout jumptable | 0x177a30 + table 0x2FB6C0 | **ported now**: `mp::bots::sp_shout_outcome` (pure, test-pinned) |
| bots' SetState tail | 0x1437a0 via 0x16E060 | `mp::bots::react_to_drone_alert_msg` + `bot_global_dispatch` (test-only; the mode-0x1E drops are [M] and pinned) |
| `+0x50C = 1.0` on a 0x15/0x13 accept | 0x1437a0:309/:272 — an effect, not a gate | not modelled; moot — no accept can happen (the arms drop first) |

Net behaviour: the port's SP guards do nothing on body/surrender shouts (nothing is broadcast),
which is exactly what the disc's SP guards do (broadcast, gate, discard). The port's bots have no
alert delivery at all, so they also do nothing — the disc's bots would react to
0x11/0x12/0x14/0x16/0x1E but not to 0x13/0x15; delivering those is the Phase-4 shout-outbox work,
out of scope here.

### The task's fork, resolved

- *"If a live path already matches, say so and delete the unused copy."* The live outcome matches
  (nothing reacts, in the disc and in the port), but `react_to_drone_alert_msg` is not a redundant
  copy: it ports `NDrone2_ReactToDroneAlertMsg` 0x1437a0, which is live code for the other alert
  ids (BotGlobal forwards every alert to it), and its 0x9E/0x9F arms carry the [M] mode-0x1E drops
  that ARE the reason the Seen states are unreachable. Deleted, the port would lose the faithful
  port of a live function. **Kept.**
- *"If not, wire the shared rule into the alert delivery, in the game's order."* Wiring a
  *reaction* would diverge from the disc (the disc's SP jumptable discards 0x13/0x15; the bots'
  arms drop mode 0x1E). The shared rule — the range law and the SP jumptable — is now ported for
  real (`in_shouting_range`, `sp_shout_outcome`) so the future delivery wiring folds them in
  instead of re-deriving; the delivery itself (senders + inboxes) stays unported, as on master.
- Rand draws: the two new functions are pure (no RNG); no draw count changes anywhere. The 0x9F
  Rand(6) bark lives in the 0x16ade0 ENTER body, which nothing executes — unchanged.

## Which levels carry the receiver props [D]

`behaviour_util_getProperty(0x29/0x4F, Drone+0x4D8)` — the dead-body receiver gate [M 0x1437a0:300-303]
and the surrender gate [M :271-275] — reads the placement data per level. Measured through the
port's parser (`Drone::prop`, the ISO level bins) by the new
`shout_receiver_props_census_07000001_and_07000009` test (counts in the run log):

- 07000001: (filled from the run)
- 07000009: (filled from the run)

The census test pins `> 0` for prop 0x29 on both levels; extending the census to the remaining
levels is listed under "What's left".

## Verification

- `shouting_range_law_and_the_sp_shout_jumptable` (new, guard_ai_other4_live): the 15/20 and
  40/100 bands incl. the 15.0/40.0/100.0 boundaries and the cel-word mid band; the jumptable
  outcomes 0x15/0x13 → None, 0x11/0x12/0x16 → 0x56; `state::ATTACK == 0x56`.
- `shout_receiver_props_census_07000001_and_07000009` (new): the [D] census above.
- The existing `bot_alert_shouts_resolve_to_the_seen_states` pins stay (the 0x9E/0x9F arms with
  the mode drops — the [M] behaviour this task explains).
- Runs: see "Binaries run" below.

### Binaries run

(filled after the runs)

### Full suite

(filled after the final run)

## Pin-through-real-play status

The task's pin scenario — "a nearby guard with the prop goes to the body's alert position" — is
not satisfiable, and the tests above pin the *actual* disc behaviour instead: the SP jumptable
returns 0 for 0x13/0x15 [D 0x2FB6C8/0x2FB6D0], so a nearby prop-0x29 guard stays in its state
(the "one out of range does not" half is subsumed: nobody reacts at any range). A surrendering
guard's shout likewise turns nobody. If the main agent can spend live time on it, the checks
below would confirm the chain end-to-end on hardware.

## Live checks for the main agent

1. 07000001 fresh boot (no save-state): kill the first guard with a second guard ~8 units away.
   Watch the second guard's SM current state (the id the `NDrone2_StateFuncs` 0x29b300 dispatch
   holds): expect it unchanged (no 0x9E), and `+0x4F8` gaining no shout bits beyond what sight
   alone sets (compare a control run with the second guard's prop 0x29 cleared).
2. Field-probe `*(0x2A003774)` right after boot: expect 0x0700000X (the level code) — the value
   the range bands key on [M 0x1788a0:14-22].
3. Force a guard into `Surrender_Anim` 0x1626a0 (or wait for a natural surrender in 07000009's
   cell block): watch a nearby guard's state through the msg 0xc shout — expect no state change
   (the jumptable's return 0), and the surrendering guard's `+0xCC4` timer armed at now+15 frames.

## What's left

- The alert delivery itself (senders, inboxes, the Phase-4 shout outbox) — unchanged from master;
  this task establishes that wiring it must reproduce the disc's *discard* of 0x13/0x15 in SP and
  the mode-0x1E drops for bots.
- The props census for the levels other than 07000001/07000009 (the test is the template).
- `NDrone2_DSTATE_SeenDeadBody`/`SeenSurrenderedDrone` state bodies stay unported entries — dead
  code in the disc; porting them has no observable effect until something SetStates there, which
  the disc never does.

## Usage

not visible to me
