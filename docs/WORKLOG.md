# WORKLOG

One entry per task, append only (never rewrite or reorder others' entries).

## guard-dead-body-shout

2026-11-07 07:28 UTC, branch `s/guard-dead-body-shout` (from origin/master 22d5836b7).

Task: guards react when another guard shouts about a body or a surrender.
react_to_drone_alert_msg in crates/nf-game/src/mp/bots.rs ports the SetState tail for alerts
0x15 (dead body → state 0x9e NDrone2_DSTATE_SeenDeadBody 0x16acd0) and 0x13 (surrender →
state 0x9f NDrone2_DSTATE_SeenSurrenderedDrone 0x16ade0) [M 0x1437a0:267-326;
Surrender_Anim 0x1626a0:74], but only guard_ai_other4_live.rs calls it. Find who raises alert
0x15 and alert 0x13 in the game (every caller) and in the port, and whether the receivers'
gates (props 0x29 and 0x4f, shouting range, +0x4F8 & 0x800000, the +0x50C share, the mode
bytes) are checked on the live path. If a live path already matches, delete the unused copy;
else wire the shared rule into the alert delivery, in the game's order, same Rand-draw count
(0x9f has its Rand(6) bark). List the levels with guards carrying those props [D]. Pin through
real play: a guard finds a body and a nearby guard with the prop goes to the body's alert
position, one out of range does not, and a surrendering guard's shout turns a nearby guard.
Run guard_ai_other4_live, the ai binaries, walkreal_07000001_live, walkreal_07000009_live,
nf-game, and the full suite total line.

- [x] setup: repo cloned, branch created from origin/master 22d5836b7, pushed.
- [08:10 UTC] The research answered the task's first question with a surprise: NOTHING moves an SP
  guard into 0x9e/0x9f — and the game doesn't either. Senders [M]: DSTATE_Dead 0x163fe8 (ENTER case 1
  and msg 0xd: prop 0x31, +0x3a == 0 → DroneAlertToObject 0x14a538 msg 0x15) and BotDead 0x171518
  broadcast 0x15; Surrender_Anim 0x1626a0 case 0xc broadcasts 0x13. Delivery: Drone_SM_BroadcastMsg
  0x175aa8 → RouteMsg 0x172128 → every drone's current state (skip ids 0xf/1/3/0x47/0x48). SP
  receivers: the states' alert arm → DroneVision_EnemyAlerts 0x177c18 (fleeing gate, props
  0x29/0x4f, the resume/home save) → DroneVision_ShoutFromOtherDrone 0x177a30 — the decompiler
  DROPS that body; the disassembly (tools/crosscheck/mipsdis.py) shows the inlined InShoutingRange
  0x1788a0 law (level 0x0700000c..f → 40/100, else 15/20, mid band = same cel word obj+0x20) and
  the jumptable at 0x2fb6c0: 0x11/0x12/0x16 → the Attack arm (bits 0x4000000|0x20000000, 0x56),
  **0x13/0x15 → return 0** [D 0x2fb6c8/0x2fb6d0]. Bots: BotGlobal 0x16e060's alert arm →
  ReactToDroneAlertMsg 0x1437a0 — whose 0x15 arm (:320-321) and 0x13 arm (:283-290) drop mode-0x1e
  receivers. So SeenDeadBody/SeenSurrenderedDrone are unreachable in the disc; the port's
  react_to_drone_alert_msg is a faithful port (its mode-0x1e drops are [M]), NOT an unused copy of
  a live rule — kept, and the shared range/jumptable law ported beside it as in_shouting_range +
  sp_shout_outcome (bots.rs) with tests + the 0x29/0x4f props census for 07000001/07000009
  [D placement data]. docs/systems/drone-alerts.md's "all shouts are dead code" line corrected.
