# DroneMove_SetBoundryFlags 0x14f140 — what it does and how the port folds it

`DroneMove_SetBoundryFlags__FP10DCVars_tag` (0x14f140, 1276 bytes) runs in
`NDrone2_ControlSTANDARD` 0x1490f0:202-206, behind the drone+0x23 no-collision flag (skins
0x5000078/94/ba and the abseil band skip it [M 0x14b300:236-238/763-765]).

## The game's mechanism [M 0x14f140]

- Clears drone+0x320 (the four blocked-direction bits), then — only while `+0x4F8 & 1`
  (the lateral push-move flag; its setter is unaccounted in the decompile) — probes four
  axis directions from the feet: (0,0,±0.5)/(±0.5,0,0), scaled by bot speed for mode-0x1E
  bots, rotated through `ApplyMatrixLVI(obj+0x90)`, staged into the `DroneReachCheckPos`
  globals, tested with `NDrone2_MoveTest(feet, probe, 0, obj, 0,0,0)`. `MoveTest != 1` sets
  the bit (2 = +z, 4 = −z, 8 = +x, 0x10 = −x). When the drone is movement-locked
  (`+0x4F8 & 0x10`) the probe is skipped and the bit is set anyway.
- Consumers: `Drone_Control` 0x139268:76-83 zeroes the +0x58c lateral velocity against bits
  2/4; `DroneAnim_PreTransCallback` 0x13b490:120-135 zeroes anim root-motion z/x against
  bits 2/4/8/0x10 — the walls stop the walk.

## The port's fold

The port's guards never walk into walls through their own movement probes: the per-move
`can_move`/MoveTest destination-cell check (drone.rs, the `Drone_FeetOnPoint`/`NDrone2_MoveTest`
stand-in near the move application) and the nav-layer route boundaries (crates/nf-game/src/nav.rs)
reject blocked destinations before the position advances — the same observable job with a different
mechanism. The citation at the port's probe site names `DroneMove_SetBoundryFlags 0x14f140` so the
call-tree audit sees the replacement.

**Documented deviation, not a drop:** the game's per-direction bits (+0x320) also gate root motion
differentially per axis (a guard pressed against a wall slides along it); the port's binary
blocked/allowed move check collapses that to stop-or-go. The push-move flag `+0x4F8 & 1`'s setter is
unaccounted in the decompile — a live check (gdbwatch on drone+0x4f8 bit 0 during a guard-vs-guard
push on 07000005) would settle whether the bit ever arms on the shipped path.
