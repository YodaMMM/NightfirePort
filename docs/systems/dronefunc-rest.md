# DroneFunc / Drone helpers: the alert getters, the behaviour switch, the target registry, the message pump

Sources: ACTION.ELF decompile (`work/decomp/ACTION.ELF`, function address = filename prefix). [M] = read in
the code, [D] = disc bytes, [L] = live, [S] = suggested. This chapter closes the call-tree gap for the 13
functions listed in task 845: every one is ported, or cited at the port code that does its work, or its
absence is a documented deviation — no silent skips. Port: `crates/nf-game/src/drone.rs`,
`crates/nf-game/src/mp/bots.rs`, and the new `crates/nf-game/src/target_registry.rs`.

## The object adapter and the message pump

- **`Drone_DCVfromOBJ` 0x13a0a0** [M]: obj → DCVars adapter; 1 only for obj class +0xff == 2 (a drone
  object) or 0x11, filling `{obj, drone = obj+0xe0, obj+0x20, drone+0x108}`. Every obj-level entry
  (`NDrone2_DealWithObjHit` 0x13fff0, `NDrone2_Collision` 0x140e90, `DroneVision_ProcessDroneSight`
  0x176a28, `Drone_SM_SetState` 0x1758c0, `DroneAnim_SetAnimD` 0x13ddd0 / `SetEndAIState` 0x13dd00,
  `NDrone2_SpeechSFXd` 0x149d58, `NDrone2_initDTYPE_SniperAlert` 0x14e580, `MP_sendBotMessage` 0x18bb60,
  `MP_setPlayerStatus` 0x18a518, `BOTSTATE_getPreferredTraitOpponentObjIndex` 0x128b50) funnels through
  it. The port has no generic obj layer — drones are typed Rust structs owned by `Drones`, and the
  class gate is the port's own drone/agent split; the adapter's work is the type system itself. Cited at
  `Drones::step` (drone.rs, the per-drone tick entry the obj-level callers land in).
- **`Drone_SM_RouteMsg` 0x172128** [M]: THE message pump at obj level — every send path calls it
  (`Drone_SM_SendMsg` 0x13a630, `SendDelayedMsgs` 0x1759a0, `SendMsgSelf` 0x175a38, `BroadcastMsg`
  0x175aa8, `MP_setPlayerStatus` 0x18a518 ×3, `MP_objectBeingDeleted` 0x18ac90,
  `MP_sendTeamBotMessage` 0x18bbc8, `MP_getObjExtObj` 0x18bca8); it resolves the obj to a DCVars and
  hands the message to `Drone_SM_RouteMsgDCV` 0x171df8 (deliver, then apply pending state changes).
  The port's pump is `Drone::step`'s deliver-then-apply dispatch (drone.rs, the `Drone_SM_RouteMsgDCV`
  port) plus `Drones`' message queues the send helpers push to (SendMsgSelf's port at drone.rs:830, the
  hurt/stun-grenade broadcasts at drone.rs:314/348). Cited there.
- **`Drone_AlertStatus` 0x13a910** [M]: the status-byte getter — returns the ADDRESS of drone+0x51c
  (the discrete alert status 0..3 `Drone_AlertStatusSet` 0x13a8d8 writes) when class == 2, else 0. Its
  only caller is `AnimFacialUpdate` 0x1be310 (the facial-expression pick), which the port does not model
  (the drone sim carries no face state; character rendering is the draw side). The port's alert is the
  LEVEL float +0x50c with the discrete thresholds mapped where the game's statuses are consumed
  (drone.rs:2835 "the port's alert-status-2 = alert ≥ 0.66"); cited at that mapping. The status byte
  itself is not modeled — a documented deviation (the facial system is the only reader in the game).

## The behaviour switch (second behaviour block)

A placement parameter 5 == 100 gives the drone a SECOND behaviour prop block (`+0x4E8`; the port's
`alert_props`, parsed at `DefaultInit` [M 0x14b300]; the port computes the block's alertness level at
drone.rs:1468). The game points `+0x4D8` (the active props) at it when `DroneFunc_FirstAttack`
0x148010:124-127 sees `+0xC48 == 100`, latching `+0x29 = 1`; the restore path:

- **`DroneFunc_Using2ndBehaviour` 0x14a5e8** [M]: `return drone+0x29` (the latch).
- **`DroneFunc_Set1stBehaviour` 0x14a5f0** [M]: 0 unless `+0xC48 == 100`; clears `+0x29`, points
  `+0x4D8` back at the first block `+0x4DC`, returns 1. Called from `NDrone2_DSTATE_GoToGoalPosition`
  0x165640's blocked tail (a class-0xC5==7 escort whose second behaviour is 0xB4 resumes it) together
  with `DroneFunc_Set2ndBehaviour`/`ChangeToAttackMode`.

Port status [S deviation]: the port parses the block (and its alertness level) but never swaps the
active props at runtime — every prop ask reads the first block — and `+0x29` has no port field. The
GoToGoalPosition tail's escort arm is the documented [S] stand-in (drone.rs, the GO_TO_GOAL fail arm).
Cited at the `alert_props` parsing site; consequences listed in the task report.

## The first-attack stamps and the comms-alert state

- **`DroneFunc_SetAsAttacking` 0x14aed8** [M]: `+0x13a = 0`, `+0x4F8 |= 0x2000000 | 0x4000000`,
  `+0x135 = 0`, `+0x524` = the frame (once), `+0x50C = 1.0`, `+0xE8 = max(+0xE8, +0xFC)`, `+0xE4 = 0`.
  Called from `NDrone2_DefaultInit` 0x14b300:686 when the spawn return state (`+0x5A2`) is 0x56 (Attack)
  or 0x12 — the already-alerted spawn. The port performs the same stamps in the ATTACK ENTER first-attack
  blocks (drone.rs:5691-5714: `first_attack_done`, `alert = 1.0`, `sight_range.max(combat_range)`,
  `half_fov = 0.0`, `attack_frame`) — cited there.
- **`DroneFunc_FirstAlertState` 0x147f58** [M]: the COMMS-alerted first-attack pick. Reads the alert
  flags `+0x4FC`; any of bits 2/0x10/0x20/0x20000/0x40000/0x80000/0x100000 set → 0 (fall through to the
  sight flow); bit 0x200000 set → state **0x39** but only on the levels 0x7000005, 0x7000007 and
  0x7000041 (the ≥ 0x7000008 band admits only 0x7000041), else 0. Its caller: `DroneFunc_FirstAttack`
  0x148010:23-30 — a drone with an alert-broadcast bit (0x70000000 family) but not its own sighting goes
  to 0x39 instead of the FirstSightState flow. The port lifts the pick as `drone.rs::first_alert_state`
  (pure, [M]-exact) and the ATTACK ENTER comms arm consults it; the port's `alert_bits` (+0x4FC) models
  the bits it writes (0x10, the prop-0x33 bit), the 0x70000000 broadcast bits are not yet modeled
  [S — the function is exact, the caller-side bit wiring is partial].

## The target registry (NPCGlobals)

The game keeps 8 slots of 0x20 bytes at DAT_00299110 (per slot: +0 flag word, +4 the target obj,
+8 the last-seen frame (GameState+52), +0x10 the 16-byte last-seen position) — the guards' shared
last-seen-position pool:

- **`DroneFunc_AllocateTargetID` 0x141f28** [M]: `drone+0x44 != 1` → 0xff (only full NPCs allocate);
  scan the 8 slots for a slot already claiming this obj → its index; else the first free slot (flag bit
  0 clear) is claimed (obj, flag 1, stamp +0x18/+0x8/+0x10/+0x14 zeroed) and returned; all full → 0xff.
  Caller: `NDrone2_SetOpponent` 0x1422d0 (every opponent acquisition).
- **`DroneFunc_TargetSeen` 0x14a008** [M]: `drone+0x174` (the drone's allocated id) != 0xff → stamp
  GameState+52 into the slot, `flag |= 2`, store the 16-byte position argument at slot+0x10. Callers:
  `NDrone2_AttackTalk` 0x13eac0, `DroneFunc_NewSightTalk` 0x13f420 (the drone's +0x250 last-seen
  vector), `DroneFunc_LostSightTalk` 0x13f550.
- Readers [M]: `Drone_GetOpponentInfo` 0x138f68 (a claimed slot whose stamp is OLDER than the drone's
  own sighting +0x238 gets refreshed from the drone's +0x250 — the shared pool converges on the freshest
  sighting), `Drone_IsCoverNodeUsable` 0x1386d8, `Drone_InitComms` 0x139520.

Port status: the registry is lifted [M]-exact as `crates/nf-game/src/target_registry.rs`
(`TargetRegistry::allocate` / `target_seen`, the port's own drone id playing +0x44/+0x174) with unit
tests pinning allocation reuse, the full-table 0xff, and the seen stamp/flag/position write, each with a
sister. The port's per-drone sighting fields (`sighted`/`opp_target`) stand in for the shared pool in
the search behaviours [S deviation — the pool's convergence effect on guard searches is not modeled];
the draw/AI side can consume the struct as-is.

## The vision helpers

- **`DroneFunc_ClampPosToCollisionCylinder` 0x1499d0** [M]: clamp the given position to within **0.4**
  of the obj's x/z centre (keep y): d = pos − obj+0x30; if |d_xz| > 0.4 → pos = centre +
  normalize(d_xz)·0.4. Callers: the whole vision family — `DroneVision_CanSeePosition` 0x178438,
  `DroneVision_CanSeeObjectFrom` 0x176f18, `DroneVision_GunThroughWall` 0x178520, `NDrone2_CanSeeObject`
  0x1785e0, `NDrone2_NearArmedDrone` 0x177420 / `NotNearArmedDrone` 0x177640. The port's sight probes
  start at the drone centre (`head()`, pos + 0.6) — already inside the 0.4 cylinder, so the clamp is a
  no-op for the port's probe starts; cited at the sight entry with that note.
- **`DroneFunc_GetBonePos` 0x149bd0** [M]: the named bone's world position through
  `AnimGetBoneWorldTrans`, gated on the anim object being present (`obj+0xDC`) and class 2/3 (class 2
  additionally requires `+0x4F8 & 0x100000` and GameState ≠ 0x700001b, and the anim list's bit-0 active
  word at +200); any failure falls back to the obj position (+0x30). Caller: `CanSeeObjectFrom`'s LOS
  start (+ `AnimFacialUpdate`). The port's sim has no bone pose — `head()` (pos + 0.6) is the LOS start;
  cited there. Bone-exact positions are the draw side's domain.

## The spawn tail and the burst counter

- **`DroneInit_Collision` 0x14b1e8** [M]: the spawn drop — not MP, `+0xBD4 == 0`, `+0x23 == 0`,
  `+0xD8 != 0x13`, `+0xC5 != 0x17`: `+0x4F8 |= 0x80000000`, ray from obj+1.0 down 25.0
  (`Collide_RayIntersect` flags 0x104); a hit moves the obj to hit_y + 1.0 and sets the anim object's
  `+0x60 |= 8`. Called from `DefaultInit` 0x14b300:775. The port spawns at the placement translation
  (authored on the floor [D — the placement census]); the drop's visible effect is nil for on-floor
  spawns, and the flagged states (0x13/0x17 abseilers) enter mid-air by design — cited at `Drone::new`
  with the gate list.
- **`DroneWeap_ManageBurstCount` 0x17a870** [M]: `drone+0xBC0` (the burst rounds left, short)
  decrement; while still > 0, `DroneWeap_NextBulletTime(dcv, 0, 0)`; returns rounds > 0. Caller:
  `DroneWeap_FireWeapon` 0x178f68 (the mp bot's fire). The port's burst pipeline is the same decrement +
  re-arm (bots.rs:3477: `burst_left -= 1`, the 0 arm ends the burst, the else re-arms
  `next_bullet_at += interval/60`) — cited there with the address.

## Tests

- `target_registry` unit tests ([M] numbers, sisters that flip): allocation returns 0 for a fresh NPC,
  reuses the same slot for the same obj, returns 0xff when all 8 are held by others, and honours the
  +0x44 gate; `target_seen` stamps the frame, sets flag bit 1 and stores the position (sister: an
  unallocated id 0xff is a no-op).
- `first_alert_state` unit tests ([M]): each gated bit returns 0; bit 0x200000 on the four level classes
  returns 0x39; sister: 0x200000 on other levels (e.g. 0x7000002) returns 0.
- Real-path: `crates/nf-view/tests/dronefunc_rest_live.rs` — the bathhouse wake scenario asserts the
  port's sighted guards carry opponent state (the SetOpponent/GetOpponentInfo flow) and a guard's
  first-attack stamps (alert 1.0, `first_attack_done`) — sisters: the propless girls never enter Attack
  and an unwoken guard keeps alert 0.
