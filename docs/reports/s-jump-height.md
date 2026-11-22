# s/jump-height: Bond's jump against the game

Result: the port's jump already matches the game. No code change.

## The game's jump [M]

- Launch, `Player_HandleJump` 0x1916b0 (state 0, jump action 7): `BL+0x50 = WldGravity * -0.4`
  (disasm 0x1918b4 loads 0xbecccccd = -0.4; 0x1918ec is `vmulx.xyz`, so the x lane of that
  vector is the scale). With the default `WldGravity` (0, -9.8, 0) the launch is 3.92 up.
  The jump is gated by a clear line from the root to (x, y + 0.95, z) of obj+0xC0 (mask 9).
- No hold: state 1 reads no input. It only waits out the 4-tick `BL+0x952` counter, then
  lands when the floor flag 8 is set and the y velocity is not positive. The head-bump flag
  0x20 zeroes the velocity (state 2).
- Each airborne tick, `Player_Collision` 0x194df0:105-116: `vel += WldGravity * REC_FRAME_RATE`,
  clamp vy to +-45, then 0x195058-0x195098: `pos += vel * REC_FRAME_RATE`. REC_FRAME_RATE is
  1/60 in play. Gravity comes first, then the move.
- No step-up assist in the jump path.

## Apex

The discrete sum with v0 = 3.92, g = 9.8, dt = 1/60, gravity applied before the move on every
tick including the launch tick: the rise is (24 * 3.92 - 9.8/60 * 300) / 60 = 0.751. That is
the rest root (1.089 above the floor) plus 0.751, so the root tops out 1.840 above the floor.

The port (`crates/nf-game/src/player.rs`): launch `-GRAVITY * 0.4`, gravity `world_gravity / 60`
before `pos += vel / 60`, the 4-tick land delay, the 0.95 headroom ray. Measured on the flat
test room: rise 0.7513, the same as the game's sum.

A higher measured rise (0.81 and up) comes from walking forward onto a ledge or prop during
the jump, not from the jump itself.

## Reach of raised pickups

`Pickup_Update` 0x18e148:203-214 takes a pickup when the player's root is within
(bounds radius + 1.1) of the pickup's world sphere centre, and a mask 0x721 line is clear.
From floor 0, a jumping root reaches 1.840, so a pickup with centre height h and radius r is
reachable only when Bond can stand within sqrt((r + 1.1)^2 - (h - 1.84)^2) of it horizontally.

On 07000014's shelf room (x -32 to -35, z 30 to 37), the pickups at 2.06 to 2.35 on the 1.309
and 1.938 shelf tops need Bond within 1.43 to 1.49 horizontally. The shelf fronts and Bond's
0.55 walk radius keep him further than that from most of them unless he stands on a lower
prop first. The jump numbers match, so from the same standing spots these are out of reach
in the game too. Unproven: whether the game's shelf collision matches the port's, which
decides exactly how close Bond can stand.
