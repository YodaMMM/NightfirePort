# CivilianMissionWait (drone state 0x17) — the mission civilian's wait-channel pause

Written against the decompile (2026-10-22).

## What it is

A mission civilian (CivilianMission 0x16, `NDrone2_DSTATE_CivilianMission` 0x15d860 — the
prop-0x1D walkers with a kind-8 route) that receives msg 0x22 parks in
`NDrone2_DSTATE_CivilianMissionWait` 0x15dbb0 instead of walking on: she stands (or plays the
alert look-around), holds a 2 s timer, extends the timer while Bond stands in front of her, and
resumes 0x16 when he does not.

- State-table entry: `NDrone2_StateFuncs[0x17] = 0x15dbb0` [L ee_snapshot.bin 0x29b300+0x17·4,
  read 2026-10-22: `0x15dbb0`; entry 0x16 → `0x15d860`].
- The entry point: CivilianMission case 0x22 [M 0x15d860:130-136] → `SetState(0x17)`, except the
  voice-0x13 skins (KikoMission) stay — the same carve-out the 0x16 ENTER applies.

## The state function 0x15dbb0 (size 884)

`switch(*param_4)` — the msg id (decompile
`work/decomp/ACTION.ELF/0015dbb0_NDrone2_DSTATE_CivilianMissionWait__FP10DCVars_tagP9Drone_tagP7obj_tagP9MsgObject.c`,
read 2026-10-22):

- **case 1 (ENTER)**: `Drone+0x4F8 |= 0x10000`, then `|= 0x2000`; if the alert status
  `+0x50c < 0.66` → `DroneAnim_SetStandIdleAnim(0)`, else `DroneAnim_CallAnim(0x21)` (the alert
  look-around; `anim_state::ALERT_IDLE` in the port); `+0xCC4 = GameState._52_4_ +
  FRAME_RATE_INT·2` (2 s; FRAME_RATE_INT = 60 at 0x30d0cc); `+0xCC8 = 0`; `+0xCC0 = 1`;
  `+0xCD8 = 0xff`; `+0xCDC = 0` [M :20-35]. The port models `+0xCC4/+0xCC8/+0xCC0` as
  `timer_c`/`timer_c_payload`, `+0xCD8` as `wait_channel = 0xff` (no channel — the tick's
  `wait_channel_on` reads false for it); `+0xCDC` has no port model (no reader found; [S] fold).
- **case 3 (UPDATE)**: `NDrone2_NearArmedPerson(0x41200000)` (the 10.0 arg is dead in the callee
  [M 0x1787c8]) — none near → `NDrone2_BondTalk` and stay [M :39-44]; someone near →
  `+0x4F8 |= 0x8000000`, then the prop-0x33 armed-intruder chain [M :45-62] (unmodelled in the
  port, matching the sibling CivilianMission UPDATE), then voice `+0xD8 == 0x10` → `+0x44 = 1` +
  `SetState(0x56)` (Attack) else `SetState(0x12)` (CivilianScared) [M :63-67].
- **case 6 / case 8** → `DroneFunc_HandleImpact(…, 0x56, 0)` [M; case 6 `uVar4 = 0`, case 8
  `uVar4 = 0`]: the bullet/punch resume is Attack, allow false. The port joins the state to the
  CIVILIAN_MISSION rows of the `impact_args` / punch tables (drone.rs, "0x15dbb0 case 8/6").
- **case 7 / 9 / 0x17 / 0x18 / 0x19** → `HandleImpact(…, 0x56, 1)` (the explosive-class impacts
  pass allow = 1) [M]. The port's explosive family rides the pre-dispatch tables with the same
  rows.
- **case 0xc (TIMER_C)**: `NDrone2_PreDroneControl` 0x148db0:97-111 sends msg 0xc to self when
  the armed `+0xCC0/+0xCC4` clock expires [M]. The state: `NDrone2_BondWithinAngle(0x3f060a92)`
  (0.525 rad) AND a player exists AND `Vec_Dist2D(drone+0x30, player+0x30) < 3.0` → re-arm
  (`+0xCC4 = now + 120`, `+0xCC8 = 0`, `+0xCC0 = 1`); anything else → `SetState(0x16)` [M
  :87-111]. `NDrone2_BondWithinAngle` 0x154f98 is the |yaw-diff| ≤ cone test; the port's
  `bond_band(3.0, 0.525)` computes exactly that conjunction (drone.rs).
- **case 0x11 / 0x12 / 0x14 / 0x15 / 0x16 / 0x1e** → `DroneVision_EnemyAlerts`, then
  `SetState(result)` when non-zero [M] — the same arms CivilianMission carries; the port models
  them with the `(state, msg::SOUND | msg::ALARM)` arm and the shared shout-scare civilian
  shortcut.
- **case 0x1f** → `DroneFunc_DoForcedAttack` [M] — the port's `msg::FORCED_ATTACK` arm.
- **case 0x21** → `DroneFunc_ConsiderExplosive(…, 0x56)` [M :132-135]. The port's EXPLOSIVE
  pre-dispatch's final `else { ATTACK }` fallback gives exactly resume 0x56 for a state listed in
  none of its arms — `CIVILIAN_MISSION_WAIT` is deliberately NOT added to the civilian-resume-0
  list (drone.rs).
- **case 0x22** → re-arm the wait clock (`+0xCC4 = now + 120`, `+0xCC8 = 0`, `+0xCC0 = 1`)
  [M :136-141] — a second wait request while waiting extends the pause.

## Where msg 0x22 comes from

The decompile holds **no literal sender** for DSTATE msg 0x22 (an earlier sweep, 2026-10-16): every `Drone_SM_SendMsgSelf` 0x175a38 / `Drone_SM_SendMsg` 0x13a630 /
`Drone_SM_BroadcastMsg` 0x175aa8 / `Drone_Message` 0x13a678 call site was disassembled and none
passes 0x22; no `li reg, 0x22` in the DSTATE region is a message id (they are
`DroneAnim_CallAnim` anim ids). The id arrives data-driven — the level side supports it:
07000005's drone placements carry wait channel 0x22 (placement param 3 → `Drone+0x134`) and
trigger 305 / mux 25 raise channel 0x22 [D docs/systems/triggers.md]. The port's tests drive the
id through `Drone::send` — the real state-machine entry any data-driven sender feeds.

## Port location

`crates/nf-game/src/drone.rs`: `state::CIVILIAN_MISSION_WAIT = 0x17`, `msg::WAIT_PAUSE = 0x22`,
the CivilianMission `(CIVILIAN_MISSION, msg::WAIT_PAUSE)` arm, the `(CIVILIAN_MISSION_WAIT, …)`
arms, and the CIVILIAN_MISSION_WAIT joins in the `impact_args` / punch / shout-scare tables.
Tests: `crates/nf-view/tests/civilian_mission_wait_live.rs` (opt-in NF_ISO, level 07000007).
