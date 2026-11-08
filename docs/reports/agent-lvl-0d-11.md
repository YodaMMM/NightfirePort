# agent-lvl-0d-11: levels 0700000d and 07000011

This branch moves the shared random stream on every level with a spawner: the spawner draw range
and retry count changed, and the guards that spawn differ.

## Fixed

| What a player sees | Cause | Evidence |
|---|---|---|
| Spawners with more than one wave only ever brought their first wave. 0700000d group 6 gave 5 guards, not 15. Later "waves" healed a live guard or revived a corpse in place. | The port's list entry was both template and object. A template's record now survives: a later wave restores it, or adds a guard with a pool body while the entry is in use. | [M 0x137cb0 builds a new object per spawn; 0x137e68:145-172] |
| Reinforcements waited for a corpse to fade (up to ~17 s). | The alive scan read `present`. A slot now frees at the death. | [M 0x137e68:96-124] |
| Mode-3 groups (all seven spawners on 07000011) sent one guard, then waited for him to die. | The +0x1C permission is sticky in mode 3 until the rollover. | [M 0x137e68:176-203] |
| Guards could spawn right beside Bond. | p8 is an integer converted to float, read as raw float bits (1.4e-44). | [M 0x1379c0-0x1379d4 `cvt.s.w`] |
| 07000011 guards held off at 12-20 and 07000014's at the class draw. | The level overrides were missing: 07000011 engage 6 / combat 9 (5 / 8 for set 9), 07000014 engage 10 (7 for set 9, set 5 keeps its draw). | [M 0x14b300:429-457, raw 0x14bb60-0x14bc1c; L 6-9 from Bond in docs/live/2026-11-01-07000011-start-corridor-guards.md] |
| The Q-Claw latched through walls, floors and decks. | The pick had no world test. It now uses the bullet ray. | [M Bullet_Delete 0x12e660:50-55] |

## Real paths confirmed

- 0700000d crane boarding: the walk's placed boarding is gone. From the control corner #460 the
  claw reaches pulley #370; Bond hangs, drops on the east girder at the east apex, the aboard plate
  fires ch 0x67 (a Bond-moment channel, sound trigger #516 sits beside the spot), and the ride and
  yard drop follow. No code change was needed. The earlier conclusion that no route existed was
  wrong: it never tried the claw.
- 07000011 vent route: ledge, claw to #265, bridge roof, grill #312, duct, grill #84, plate #235,
  plate #283 unlocks door #213. New test `the_vent_route_reaches_the_storeroom`.
- Objectives, channels and scripted scenes on both levels: writers and readers read against the
  decompile with no further divergence found. Classes checked in depth this round: spawners,
  DefaultInit ranges, hints, level-specific branches, the claw. Doors, triggers, locks and the
  rest were spot-read, not re-derived line by line.

## Tests

New, red on master and green here: `spawner_waves_live` (5 tests, master 5 vs 15 guards, 1 vs 2 in a
fill, 1.4e-44 vs 10.0, 14.3 vs 6.0) and `interactables_0700000d_live::the_claw_stops_at_a_wall`.
`the_vent_route_reaches_the_storeroom` and the walk's boarding are route pins, green on old code
too.

Pins corrected because they encoded the old behaviour: `spawner_kill_live`,
`interactables_07000015_live::the_east_room_touch_wakes_its_squad`,
`qclaw_reels_to_the_crane_pulley` and `grapple_point_reels_bond_up` (both fired through a deck).

Ignored with the measured reason, both walk scripts rather than game faults, and both need new
stands: `walk_arrival_to_the_c5_door_real_damage` (the exposed hang and ride) and
`walk_spawn_to_the_lower_floor_load_point_real_damage_00agent` (guards now close to 6-9).

## Unproven

- The crane boarding and the vent route have not been watched in the real game. The data points
  to both, but the question for a live check is: from the corner #460, does the claw latch #370 and
  does the fall to the gantry fire ch 0x67?
- A later-wave guard added beside a live template draws with a pool body (3 per skin). The body
  carries the skin's first placement's held weapon, so a mixed-weapon group can draw the wrong gun.
  No render was taken; a spawn waits when no body is free.
- The reel's abort when grounded after 15 ticks [M 0x192860] is not ported, and the reel does not
  collide. Both are rare with a clear line.
- The Bond-moment plate #235 does not fire for a standing Bond on the grill, only on the fall or
  crouched. The route's fall fires it. The contact law for 0x50 plates stays open.
