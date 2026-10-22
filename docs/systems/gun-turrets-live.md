# gun-turrets-live — the kind-0x27 gun turrets deploy and shoot

The GUNTURRET records (`GT_Create` 0x17ffa0, class 0x36) run `GT_Update` 0x180498 every tick.
Before this task the port's turrets sat in ARMING forever (the state machine waited for the two
turret scripts' done flags; nothing ran them) and a "fire" only played a sound. This task runs the
scripts through the port's script player and fires real bullets.

## The game's chain (all [M] unless tagged)

- **Create** (`GT_Create` 0x17ffa0): gate = placement params 2 AND 3 non-zero (either failing →
  the object self-deletes, :106-109). `Script_Load(param 2 → ext+0xb0)` + `Script_Load(param 3 →
  ext+0xb4)` with the `GT_Register` 0x181538 callback (script objects → ext+0xc0/c4/c8; +0xc4 is
  the barrel whose matrix spins and whose +0xc0 anchors the muzzle). `Script_Play(_, 1)` on BOTH
  is the LOOPING play (`Script_Play` 0x1db588:16-19 — the non-zero arg sets `play |= 9`), then one
  `Script_Update` each (:112-116). ext+0xd0 (the muzzle offset) = script-0's objects' glist bbox
  max.z × 1.5 (:118-140). ext+0x80 = the create-time fire direction: facing 0 → `Mat_GetDir`
  (the matrix's forward row), else `Mat_GetUp` negated (:141-147). Facing 0 → HP 200, else 0xffff
  (:92-96).
- **The scripts park on delays**: the deploy/fold anims are looping one-armed shows whose last
  event is kind 3 (the delay park: `SCRIPTINFO+0x9c = p0 − 1` [M 0x1d94c0:98-117; the delay read
  disasm 0x1da9c4]). 0700000a's wall-gun scripts 0x06000041 (stream 10) / 0x06000042 (stream 8)
  end parked with p0 = 0 → the −1 hold [D the level's script streams]. The park IS the done flag
  the state machine waits on.
- **`GT_Update` 0x180498** (obj+0xf4 = the state):
  - top: `Script_Update` BOTH scripts every tick (:111-112); the gas-damage walk can
    `GT_Disable` (:130-187); the death gate `ext+0xcc < 0 || ext+0xf0 == 0` → state 6 (:189-203).
  - case 0 DORMANT: facing 1 gates on the enable channel (ext+0xe8, param 4) ON and the disable
    channel (ext+0xea, param 5) OFF, then `GT_Track`; the tail clears BOTH +0x9c (the rewind that
    makes the looping anims replay) and → ARMING (:205-230).
  - case 1 ARMING: both scripts' +0x9c != 0 → deploy complete → COMBAT (:231-238).
  - case 2 RESTORING: the 1.2 s slerp home, then clears both +0x9c → FINISHED (:239-265).
  - case 3 FINISHED: both parked → facing 1 drops the 0x2000366 pickup + self-deletes; facing 0 →
    DORMANT (:266-286).
  - case 4 COMBAT: `GT_Track` each tick (:287-289); with a target — `Music_Event(2,1)`, the aim
    slerp (`Mat_LookAt` at the target), the fire timer ext+0xe0 > 0.1 s per shot, the ammo
    ext+0xf0 decrements ONLY while > 0, `Bullet_init(-1, owner, barrel, weapon 0x2c55d0, muzzle,
    dir)` (0x2c55d0 = the weapon table + 96·0x10c = record 96 [M docs/systems/drone-weapon-drop.md:10]),
    the barrel spin, and the flash object toggle (:287-383). Without a target the facing-1
    re-holster gate counts 0.5 s (the channel-gated form the 2026-10-21 review fixed) → RESTORING
    (:385-423).
  - case 5/6/7: the gas sweep, the wreck (MP explode), the mounted minigun (:424-561).
- **`GT_Track` 0x17f9c8**: the current-target validity re-check (:83-99); SP range = the SQUARED
  distance vs 2500 (50 units), MP 500; drones/copters are candidates for facing-0 turrets only
  (:101,119); the drone filters (ext+0x4f8 & 0x600 clear, & 0x100 set, ext+0xac > 0, class ≠ 0x11,
  not deleted, not hidden); the hidden-bit skip (:150); cap 63; distance-sorted, the first two
  with LOS (`Collide_LineOfSight(muzzle, cand+0x80, …)` :258-260) remembered, the current target
  kept while it is one of them, else the first LOS candidate (:246-300); and the FRONT-CONE dot
  test — the candidate's normalised direction dotted with ext+0x80 must reach 0.01 (:252-257).

## The port

- `crates/nf-game/src/gt.rs` — the record, the state machine, `track`, the events.
- The two `Script_Load`s run as **mode-8 script players** (the fusebox/laser mount precedent):
  primed on the first tick (`Script_Play`-looping + one update = the create beat [M 0017ffa0:112-116]),
  ticked by the control walk in placement order. The GT tick syncs `scripts_done` from both
  players' `Script::delay` (the +0x9c model) and zeroes them when the state machine rewinds
  (`GTEvent::RewindScripts` — the two clear sites). The park edges land one tick late vs the game
  (the players tick after the GT pass in the walk; the game ticks the scripts inside `GT_Update`)
  — documented skew, immaterial against multi-second anims.
- `GTEvent::Fire` now spawns a real `Bullet` (weapon record 96, the muzzle + aim direction, owner
  usize::MAX = the game's −1: every hit-time modifier that reads the shooter drone skips) + the
  0x4e7/0x10a muzzle sound. The bullets fly the same `Bullet::update` path the guards' bullets
  fly and damage Bond through the same `pain_from` chain.
- `GT_Track`'s front-cone dot test added (the fire direction keyed from the placement matrix at
  the build). The LOS walk is still caller-supplied true (the port has no turret collide ray —
  the documented stand-in, as before this task).
- Known gaps (draw-side / data-side, listed for the main agent): the `GT_Register` slots
  (ext+0xc0/c4/c8) are not mapped to the script-created objects (the flash-object hide gate and
  the barrel-matrix spin stay events for the renderer); `muzzle_offset` (ext+0xd0) stays 0.0 —
  the game computes it from the script entities' glist bboxes at create, the port's bullets spawn
  at the turret base (the direction carries them; the live test shows hits land).

## Fixtures [D placements dump]

0700000a #636/#637 — "GunSamuri-Placement" wall guns: facing 1, enable ch 18 (#638/639/640/641
carry ch 17/16/15/15), scripts 0x06000041/0x06000042 (the loader's id→hash resolution of params
0x06000101/0x06000102), at (9.84, 2.28, 17.48) yaw −π/2 and (26.18, 2.28, 4.98).

## Live checks for the main agent

1. [L] Fresh boot PCSX2, 0700000a: raise ch 18 (the wall guns' enable channel — the level raises
   it via its own triggers; find the writer with gdbwatch on `switch_channels[18]` 0x26fd90+18·1).
   Measure: the deploy delay from the channel edge to the first shot (expect one loop pass of
   0x06000041 — its length is 180 ticks ≈ 3 s [D the NIS header]); the fire cadence (expect one
   shot per 0.1 s while the target holds [M 0x180498:317-348]); the damage per hit to Bond
   (weapon_data record 96 +0x0C raw — the turret has no shooter drone, so no TooClose/backshot
   modifiers apply [M 0x139f50's drone reads]).
2. [L] The same boot: stand Bond BEHIND a wall gun (outside the −up front cone) — expect no
   deploy/track; then in front — expect the deploy. Confirms the ext+0x80 cone on hardware.
3. [L] `muzzle_offset`: watch the turret's first tracer origin vs the gun model — the game spawns
   the bullet at barrel+0xc0 + dir·(bbox max.z × 1.5); the port spawns at the base (gap above).

## Tests

- `crates/nf-view/tests/gun_turrets_live.rs` (NF_ISO-gated): the sister (ch 18 down → DORMANT, no
  damage, the scripts parked) and the real path (ch 18 raised through the real channel write →
  COMBAT within 15 s, the ammo counter decrements per shot, Bond's health drops).
- `crates/nf-game/src/gt.rs` unit tests (13): the creator map, the script gate, the state machine,
  the re-holster gate, the cadence, the track rules (now cone-aware), the death gate.
