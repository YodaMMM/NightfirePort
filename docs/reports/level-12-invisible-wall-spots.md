# level-12-invisible-wall-spots — guards held by nothing visible on 12

Status at the sandbox cutoff (2026-11-05 ~11:25 UTC): **the reported wedge (22.97, 1.06, 36.21) did
not reproduce through the real-path alarm on this master tree — the port's alarmed guards stop
EARLIER, at the corridor mouth and inside the west room, and the stop causes there are named and
addressed.** The probe, the route logs and the stop-cause samples are committed
(`crates/nf-view/tests/walkreal_probe12g.rs`); the next difference to test is pinned below.

## What was run

`walkreal_probe12g::repro_the_real_alarm_charge` (difficulty 1), on `origin/master` ced8324ae + this
branch's two commits:

1. **The alarm through the real path.** No hand-sent messages: switch channel 0x96 rises through the
   switch layer (`Drones::set_channel` — the write the game's alarm press performs [M
   `DroneFunc_SetMissionFailReason` 0x13e560 case 0x10 ← `Switch_GetSwitchChannel` 0x1b0430, the
   nearest 0x29 switch's channel]), and the port's own rebroadcast delivers msg 0x1E to every drone
   every 30 frames [M drone.rs 18601-18602, `Drone_InitComms`]. `ALARM ch 0x96 on true` logged.
2. **Bond walks the real route** (no position writes): the corridor jog through touch plc 30 onto
   the reported wedge — `BOND IN PLACE at (22.56, 1.10, 35.86)` (0.6-arrival on the wedge's nav
   node; Bond stands ~0.6 u from the reported spot).
3. **Per-guard route logs every 150 ticks** for all 32 drones, a 60-tick-stationary wedge tracker,
   and a stop-cause sample at every wedge (goal, route_blocked, nav walls within 3 u, the static
   segment drone→goal, floors at both ends, doors within 6 u, the sight line to Bond).

## Findings

1. **The wedge is not an alarm point.** No aipoint group-8 (the alarm panels, `find_alarm_point`
   [M 0x153008 → `aipoint::closest` group 8]) sits within 14 u of the wedge — the nearest AI point
   is #10, group 2, at (34.72, 0.00, 33.07), 12.16 u away [L the probe's AIPOINT dump]. The
   RunToAlarm→PressAlarm chain does not explain guards standing at that spot.
2. **Nobody reaches the cafeteria on the real-path alarm.** The full 1500-tick track:
   - **drone#6** (plc 12, skin 0x5000008, hp 12) advances from the corridor (10.75, 20.42) to the
     corridor mouth (13.1–14.6, 27.3–28.7) and holds there the whole run, oscillating
     AIM_CROUCH_FIRE (0x70) / STEP_AIM_RIGHT (0x74) / AIM_STAND_FIRE (0x68) / COMBAT_OUT_OF_RANGE
     (0x5a). Its stop cause: **no goal, route_blocked FALSE, no doors within 6 u, no static segment
     to Bond — a combat hold with CLEAR sight at ~9–10 u**, beside the mouth's BLINK walls #6
     (11.18, 0.42, 28.09)–(8.14, 24.86) and #905 (11.18, 28.09)–(10.17, 29.14) [L the probe's
     NAVWALL dump; the BLINK walls are nav.rs's boundary walls, `AIBounds_Link2Cel`'s rays [M]].
   - **drone#7** (plc 13, hp 30 — a captain) holds UNDER_COVER_FIRE (0x96 the state) at
     (13.4–13.5, 30.8–31.0) with the goal pinned at its own cover spot (13.064, 0, 30.649) — it
     never leaves cover to advance.
   - **drones #11/#12** (plc 21/22, the west-wing north-room pair enabled by touch plc 30's ch 0x5)
     oscillate 0x73/0x74 inside the west room at (−33.7, 33.6–35.5) and never leave — consistent
     with the prior task's floor scan (the west room is cut off from the cafeteria by gaps and the
     raised 3.25 floor at x −10..8 [D scan, commit 3fa8715e9]).
3. **The reported spot itself never sees a guard in these runs.** The nearest any guard got is
   drone#6 at (14.61, 27.30) — ~9.9 u south-west of the wedge. The driver's frame-1391 wedge (guards
   pressed against empty space at the wedge, 0.43 u south of the BLINK corner
   (37.23,36.66)→(22.47,36.64)→(20.0,36.14) [D scan, commit 814fca9c5]) requires guards INSIDE the
   cafeteria, which the port's combat hold prevents before any BLINK corner could.

## The next difference to test (pinned)

The guards hold with CLEAR sight at ~9–10 u per the port's `combat_state` [M `DroneFunc_CombatState`
0x148338's port — the AIM_STAND_FIRE case: hold-and-fire when seen and inside `engage_range`;
`routed && out_of_range` → COMBAT_OUT_OF_RANGE]. drone#6's t=149 COMBAT_OUT_OF_RANGE shows the
advance fires but gains only 1–2 u before re-holding. **Compare `engage_range` for skins
0x5000008/0x5000006 against the game's `Drone+0x1A0` [M]**: if the port's is longer than the
game's, the guards hold at 9–10 u where the game would advance into the cafeteria — which is what
puts guards at the wedge in the driver's play (and only then does the 0.43-u BLINK corner matter,
for the no-sight advance). Second: the west-wing pair's path out of its room (the gaps + the 3.25
floor). Third, if guards do advance with Bond out of sight: whether the port's no-sight advance
stops 0.43 u short of the BLINK corner where the game's arrival radius reaches.

## How this was verified

The probe binary runs in ~2.7 s and its full log is reproducible:
`NF_ISO=… cargo test -p nf-view --test walkreal_probe12g -- --ignored --nocapture --test-threads=1`.
Every claim above quotes a logged line from that run (the WEDGE/STOP CAUSE/AIPOINT/t= lines). The
prior task's probes (`walkreal_probe12f`, commits 814fca9c5/3fa8715e9) corroborate the corridor-mouth
parking and the west-room cutoff with the hand-sent-alarm variant.

## What's left

- The `engage_range` comparison and, if it differs, the fix + a visible-proof walk (the wedge under
  the real alarm).
- The west-room pair's pathing (the prior scan's gaps/3.25 floor).
- The no-sight advance vs the wedge's BLINK corner.
- The full suite did not run this session (the branch is additive only: one new #[ignore]d probe
  file + WORKLOG/report; no src changes, so the red-risk to other binaries is nil).

## Live checks for the main agent

1. In PCSX2 at the same spot (Bond at the wedge, alarm raised): do the corridor guards ENTER the
   cafeteria? Expected per the driver's frame-1391: yes, and they stop at (22.97, 1.06, 36.21).
   This settles whether the port's corridor-mouth hold is the difference or the driver's play
   differed (difficulty, position, sight).
2. Watch one skin-0x5000008 guard's `Drone+0x1A0` (engage range) live: read it in the debugger
   during the charge — expected if my hypothesis holds: a SHORT range (the guard advances), vs the
   port's long one.

## Usage

not visible to me
