# HostageKiller 0x15c728 / HostageKillerAttack 0x15c940 (states 8/9)

The DTYPE-4 executioner and its attack beat. Handler addresses are `NDrone2_StateFuncs[8]` /
`[9]` [L 0x29b300 table read]. Evidence tags: [M] decompile function+address, [D] disc data,
[L] ELF/snapshot read, [S] suggested.

## Reachability [M/D]

- Entry: Global state's ENTER dispatches the drone's start state `+0x5a2` [M 0x15b830:39-42]; mode 4
  maps to start state 8 (`GetDroneTypeAttackTypeFriend` 0x14cbd8 case 4 [M]); HostageKillerAttack 9
  is entered from Global's timer case 10 on a DTYPE-4 killer with flags 0x100 [M 0x15b830:50-57].
- **Five DTYPE-4 placements exist on the disc, all in 07000012 map 38** (object `oDrone`, skin
  0x05000006, parameter 5 = 0x64) [D census over all 99 level archives / 555 drone placements]. The
  shipped game runs these states on 07000012.

## Behaviour ported (drone.rs)

- `HOSTAGE_KILLER` 8 ENTER: vision bits on, aim anim 9 [M 0x15c728:19-26]; the pairing
  (`SetupHostageKiller` 0x144030 — nearest unpaired hostage, links both) runs once in the `Drones`
  tick; no hostage → Idle 4 [M :27-29]. Tick 3: sight → combat [M :33-39]. Msg 5 (ANIM_DONE): combat
  0x56 (the marker shout rides the shared shout path) [M :40-46]. Timer 10: with a live hostage
  link, the execution anim 0x58 [M :63-68]. Impacts: case 8 (0, allow 0) and the 7/9/0x17/0x18/0x19
  group (allow 1) via `impact_args` + the `on_explosion` raise [M :47-62].
- `HOSTAGE_KILLER_ATTACK` 9 ENTER: with a hostage link, the 1 s execute timer (FRAME_RATE) + aim
  anim 9; without, combat 0x56 [M 0x15c940:22-51]. Timer 10: the execute anim 0x58 [M :86-96].
  Tick/impacts as above [M :55-100].
- Port fields: `Drone.hostage_target` (the killer-side link) + `Drone.killer_searched`, bound in the
  `Drones` tick beside the existing hostage-side `captor` machinery. **[S] deviation:** the game
  binds both halves of +0x2b0 in the one `SetupHostageKiller` call; the port binds the killer side
  here and leaves the tested hostage-side `captor` search unchanged, so the two searches can pick
  different partners.

## Tests / live checks

The port is compile- and regression-covered (nf-game lib 352/0 at the time of writing). A 07000012
real-path live test (walk to map 38's executioners, assert they pair and take state 8, then 9 on the
Global beat) is future work — see the report's live checks.

## Mission dialogue (the 0x1A monitor) — hostage-killer-dialogue, 2026-10-22

`NDrone2_DSTATE_HostageKiller` case 3 calls `NDrone2_MonitorTalkToMissionObj` 0x144340 before the
opponent look [M 0x15c728:34]; case 0x1A runs `NDrone2_TalkToMissionObj` 0x14a8b8 [M :86-89].

- The sequencer [M 0x144340:10-73]: step counter `+0x4CE` (u16, DefaultInit 0x14b300:80 zeroes it),
  the hostage obj at `+0x2B0`. Step 0: msg 0x1A to the captor's own obj, step 1 [M :18-22]. Step 1:
  wait on the captor's own `+0x4D0` (the busy test clears a finished line in place [M :10-24]), then
  msg 0x1A to the hostage, step 2 [M :26-29]. Step 2: wait on the HOSTAGE's `+0x4D0` — read live
  through hostage obj+0xE0 → its drone struct — clearing a finished one [M :47-56], then msg 0x1A to
  the captor, step 1 [M :58-73]. Captor and hostage alternate lines, each gated on the other's voice.
- The 0x1A handler [M 0x14a8b8:8-21]: `+0xC5` 4 → `0x29 + Rand(6)`, 5 → `0x2F + Rand(4)`, else
  nothing; `SpeechSFX(idx, 0, 0, 0)`.
- Port (`drone.rs`): `Drone.talk_step` (+0x4CE) and `monitor_talk_to_mission_obj`, called first in
  the `(HOSTAGE_KILLER, UPDATE)` arm (gated on the pairing link — a linkless HostageKiller UPDATE is
  unreachable [M 0x15c728:21-29]); `(HOSTAGE_KILLER, MISSION_TALK)` reuses `talk_to_mission_obj`.
  Self-sends ride `self_msg` (`Drone_SM_SendMsgSelf`'s port model — the game's `Drone_Message`
  0x13a678 ends in `Drone_SM_SendMsgSelf` for any target), the hostage send rides `msg_out` →
  `Drones::delayed_msgs`. The hostage's tick-head voice state is snapshotted in `Drones::tick`
  (`voice_pool`) into `Drone.hostage_voice` — the game reads it live through the hostage's obj+0xE0
  [M 0x144340:47-56], which the walk's borrow can't reach. The game's step-2 clear of the hostage's
  finished `+0x4D0` [M :53] is flagged (`hostage_voice_clear`) and applied post-walk, before the
  speech pass: `speech()`'s non-forced gate refuses a line while any voice is referenced [M
  0x141848's `+0x4D0` gate], so a lingering finished voice would silence every later hostage reply.
  **[S] port model:** the game's SpeechSFX sets `+0x4D0` synchronously inside the 0x1A handler; the
  port's voice lands in the post-walk speech pass, so a pending `talks` entry counts as the
  in-flight `+0x4D0` in both busy tests. The `+0x2B0 == 0` broadcast (`Drone_Message` 0 →
  `Drone_SM_BroadcastMsg` [M 0x13a678:14-17]) is unreachable — the pairing fail arm leaves the
  state before any UPDATE [M 0x15c728:27-29]; the port sends nothing there.

> Correction (review 2026-10-21, nf-soak "review other-3b"): the shipped-game reachability of these
> states is the mode-4 killer pairs on 07000002 (×2) and 07000003 (×2) — placement mode param 4 →
> `+0xC5` = 4 via `NDrone2_GetDTYPENEW` 0x14c938 case 4 [M]; the k16=4 census shows no mode-4
> placements on 07000012 (its dtype-4 placements carry mode param 0 [D drone-props]). The
> 07000012-map-38 census above stands as recorded; the live coverage below is 07000002.
