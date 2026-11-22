# s/drone-collision-standins

No behaviour change. The random stream does not move. This branch adds a measurement probe
(`NF_COLLPROBE=1`, off by default) and a pure door-contact helper (`Doors::mover_hits`, split out
of `collide_step` with the same logic).

## What was measured

The guard's collision body in the game is one HITTEST, the capsule `NDrone2_Collision` 0x140e90
builds each tick: centres pos + up·r and pos + up·(r − hip), r = 0.4 (0.55 for voice set 0xC, 0.3
stacked when down), mask +0x464 = 0x800 when `NDrone2_DoCollision` 0x149ec8 returns 1, else 0
[M]. `Collide_Update` 0x1ea398:78 skips the pick of a zero-mask HITTEST, so that tick the guard
touches no trigger, door or object [M]. The corpse arm writes 0x800 unconditionally [M 0x140e90].

Four port sites stand in for that body with a larger or always-on shape:

| Site | Port stand-in | Game rule |
|---|---|---|
| Trigger touch (`Trigger_Calc` touchers) | segment pos−1.0 .. pos+0.6, r 0.4, every enabled guard | the capsule at the guard's hip, only when the mask is on |
| Door contact (`collide_step` movers) | `capsule(1.0)` (hip fixed at 1.0), every present guard | `capsule(hip)`, only when the mask is on |
| Body contact with Bond (msg 0x22) | `capsule(1.0)` | `capsule(hip)`, only when the mask is on |
| Smoke cloud touch (msg 0x17) | point in cloud radius + 1.05 | capsule vs cloud sphere, only when the mask is on |

The probe evaluates the port verdict and the game verdict side by side each tick and prints every
disagreement (`COLLPROBE` lines; `NF_COLLPROBE_DETAIL=1` adds the guard and trigger data).

## Results (all walkreal binaries for 01, 05, 0c, 12, 14, plus enemy_accuracy_live)

| Site | Evaluations | Disagreements | From the shape | From the mask gate |
|---|---|---|---|---|
| Trigger touch | 1.78 M | 9 303 | 0 | 9 303 |
| Door contact | 11.2 M | 5 866 | 0 | 5 866 |
| Body contact | 5.1 M | 0 | 0 | 0 |
| Smoke touch | 0 | 0 | | |

The larger shapes never changed a verdict on these routes. Every disagreement comes from the mask
gate: guards in a no-physics animation (anim info bit 2), not in view, and more than 2.0 from Bond.

- 01: Mayhew (skin 0x0500004F, state 0x23) on the exit plate #180 (out 0x10, mask 0x18) at the
  exit tick, in 3 of 7 walks. Under the gate the exit waits until he is in view or within 2.0.
- 05: the gate guard (skin 0x05000030, idle) on plate #318 (ch 0x72, part of the truck chain),
  frames 1336 to 4103. Under the gate ch 0x72 stays off while he is out of view.
- 12 / 14 / 4a: unseen guards in route and combat states inside a door's contact reach; 4a's are
  disabled guards whose contact the door already ignores.

With the gate and the game shape applied to the trigger touch, the walks stay green: 05 17/17,
01 7/7 (1 ignored as on master). The gate only delays those contacts by a few ticks on these routes.

## Why nothing was ported

The shapes never disagree in play, so porting them changes nothing a player would see. The gate
does disagree, but its verdict hinges on the port's `in_frustum` stand-in for the game's draw
pass. Applied wrongly, it could hold the 01 exit or the 05 truck chain while Bond looks away.

Open question for a live check: in the game, does plate #180 on 01 (or #318 on 05) fire while the
guard standing on it is off screen and more than 2.0 from Bond? A watch on obj+0xFC and
Drone+0x464 of that guard with Bond turned away settles it.

Also unported: Oddjob's hat (0x45) kills on a point inside 1.05 of a guard; the game runs the hat
through the bullet path (`Collide_PickObj` sphere, then `Collide_Jointy` boxes). The hat never
appears on these routes, so it was not measured.
