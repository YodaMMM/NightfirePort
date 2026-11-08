# agent-capsule-push: guard pushes can no longer carry Bond through walls

## What a player notices
On 07000007, walking through the hall doorway where two guards stand, Bond no longer gets shoved
into the doorway's west jamb and falls out of the level. The same thing on 07000003's courtyard
flank past the gatehouse wall (two guards) is fixed too.

## Cause
The port applies the actor-actor capsule push (`Collide_Jointy` 0x1ec180:229-281) only on
airborne ticks (a port choice, documented at the push in `player.rs`). While Bond walks grounded
through guards, their overlap builds up. On the first airborne tick (the feet band drops out on
the doorway threshold slope) the whole overlap landed at once: -0.70 x at (93.50, 44.14, -48.04).
That put Bond's spine at x 92.79, past the jamb plane at x 92.855. The world pass then pushed him
out the far side to (92.61, 44.46, -47.17), inside the jamb where there is no floor, and he fell
to y -115.

## Game order [M]
- `Collide_Update` 0x1ea398 intersects every HITDATA node in one pass. Each `Collide_Intersect`
  0x1ebaa8 (:353-373) and each `Collide_Jointy` (:253-266) adds its shift to the HITTEST arms
  +0x20/+0x30 and to +0x10 in place, so each node after a push tests the shifted capsule.
- `Player_CollisionHandler` 0x1949a8:136-140 adds the summed +0x10 to obj+0x30 once.
- Jointy has no ground gate, so in the game each tick's shift is small and the wall nodes absorb it.

## Fix [S port structure]
`clip_push_to_world` (player.rs) sweeps the push's horizontal part from the post-wall-pass capsule
(rays from the top and middle along the push). It stops the push one radius short of the first
wall (|ny| < 0.7) it would cross. The vertical part is unchanged, so the sleeper drop is unchanged.

## Tests
- New `capsule_push_07_doorway_live`: a real walk with no teleports (the probe_07_route legs, the
  E press, then the doorway's west side). Red on the old code (Bond at y -36.9 after 240 ticks),
  green with the fix.
- `flank_fall_repro::the_old_courtyard_flank_stays_on_the_floor` is un-ignored. It was red on the
  old code (fall from y 6.65 at tick 20855) and is green with the fix.

## Unproven
- The per-bot ON gates (+0x4f8 0x20/0x200, +0x570 bit 2) and the grounded-tick push are still not
  modelled. The sweep is a port guard, not the game's mechanism.
- probe_07_route's later legs (x 92.5, z -47.5 .. -45.3) were picked on the buggy walk and lie
  inside the jamb. They are assertion-free and were left unchanged.
