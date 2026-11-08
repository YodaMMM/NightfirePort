# s/agent-player-moves: Bond's movement rules against the game

Compared `Player_Update` 0x193600, `Player_Move` 0x191dd0, `Player_SSWalk` 0x192310,
`Player_SSCrouch` 0x1924c0, `Player_HandleJump` 0x1916b0, `Player_Collision` 0x194df0 and
`Player_CollisionHandler` 0x1949a8 with `crates/nf-game/src/player.rs`.

## Fixed (tests: `crates/nf-view/tests/player_moves_live.rs`, all red on the old code)

1. **07000007 party walk.** `Player_Move` multiplies its argument by 0.9 while channel 0x48 is
   off on 07000007 [M 0x191dd0; disasm 0x1921d4-0x19220c]. Through the scaled-lane ease the walk
   settles at 0.0675/tick instead of 0.1. The port had no such factor. The gate reuses the
   Sim's existing `jump_locked` flag, which is the same condition [M 0x1916b0:106-113].
2. **Pitched swim stroke.** The swim quad is local: its forward lane is dir.z times the lane,
   with dir the RotMatrixX(view pitch) row [M 0x191dd0:149-165], and that is the value the next
   tick's ease reads back. A 45-degree dive therefore eases from cos(45)·lane and settles slower
   (lane 0.048, read-back 0.034). The port eased the bare lane, and `move_local` carried the
   world z component instead of the local one.
3. **Fall timer.** `Player_CollisionHandler` zeroes BL+0x928 on every tick of substates 1, 2, 3,
   5, 6, 7, 8, 9, 0xd, 0xe and 0xf [M 0x1949a8:83-95; jump table @0x2fc050 read from the ELF]
   and on its climb branch [M :54-71]. It also zeroes the timer on any airborne tick whose y
   velocity is not negative [M :97-100]. The port kept the count through swims, climbs and
   wires and only cleared it on landing, so a long drop into water was charged as fall damage
   on the next landing.

## Found, not changed

- **Crouch walk speed (fix on its own branch, s/agent-player-moves-crouch).** `Player_Update`
  copies the whole +0x10 move quad into the +0x20 smoothers after the move [M 0x193600:296-299;
  disasm 0x193cd4 `lq v0,0x10(s2)`, 0x193cdc `sq v0,0x20(s2)`], and `Player_Move` stores the
  lanes already multiplied by its argument [M 0x191dd0:166-170]. The crouch passes 0.75
  [M 0x1924c0:58], so the game's ease runs on the scaled lane and settles at 0.15/0.4 = 0.375 of
  the target: 0.039/tick at full stick. The port moves 0.075/tick, about twice the game's
  crouch speed. With the fix, five `walkreal_0700000a_live` stealth-route tests go red: the
  routes' timeline shifts and the alarm runner / guards catch Bond. A first re-route of the #607
  crouched crossings did not recover them. This branch keeps the port's crouch pace and marks
  it as a known divergence in `physics_tick`.
- **The move quad zeroes on every tick, in every substate.** The `Vec_Zero(BL+0x10)` /
  `Vec_Zero(BL+0x30)` pair in `Player_Update` sits under `obj+0xF4 == 1` (the player STATE,
  alive), not under substate 1 [disasm 0x1937e0 `lh v1,0xf4(s3)`, 0x193828-0x19383c]. The switch
  reads `obj+0xF6`. `Player_Wire` 0x197658 writes no BL+0x10 lane, so in the game a wire rider
  reads as still to the guards' moving test (`DroneWeap_DoOpponentTargetting` 0x179a48:34-41).
  The port zeroes the quad only on climb ticks (`tick_ctx`), and the 0d wire-crossfire route in
  `walkreal_0700000d_live` depends on that. Changing it moves guard accuracy (and with it the
  random stream), and the 0d route would need rework, so it is left for a separate task.
- The jump launch uses the constant −9.8·−0.4. The game scales `WldGravity` [M 0x1916b0, param_3
  = 0x2d8890]. The two are the same on every walk-substate level, so nothing changes in play.
- On an object floor (anim+0x58 set) the grounded tail scales the velocity by 2/60 rather than
  zeroing it [M 0x194df0]. The port zeroes it. No player-visible case was found.

## Checked, no divergence found

Walk/back/strafe targets and the 0.2 ease, the dead-zone and the ±0.1 cap. The jump gate
(history, head bump, 0.95 headroom line), its 3.92 launch and the 0xA3/0x5FB landing sounds.
The crouch toggle/hold stand rule. The fall damage edge (timer − 60 ≥ 10). The swim entry
threshold and surface clamp. The capsule arms and the feet ray.

## Not verified

The view's head bob reads `local[1]` (`scene.rs`). On the crouch branch the crouched bob follows
the slower lane. That is a display consequence and was not looked at.

The crouch's own anim transition timing (the port's 60-tick lockout) is still a port stand-in.
The party, swim and crouch speeds are decompile-derived, not measured on PCSX2.
