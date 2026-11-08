# agent-qclaw-walls: the Q-Claw stops at walls, floors and ceilings

Random stream: a claw fire that used to latch through geometry now fails, so the gun path fires a
real claw bolt instead (as the game does). The draws on those ticks change. Only scenes that fired
the claw through geometry are touched; the 07000014 walks were re-cut for it (below).

## The game's rule [M]

- The claw fire spawns a flying bolt like any round (`Player_WeaponInitBullet` 0x1a4bf0). It
  starts at the head, moved to the midpoint of the head and view-model datum 0 for a record with a
  first-person model and `+0x68 & 0x10000` (the claw 0x50 has 0x124a0 [L]). It aims at
  head + aim·256 (:216-245). The claw records carry no gravity bit (`+0x6c` = 0x4000 [L]), so the
  bolt flies straight, out to the record's `+0x1c` range.
- `Collide_Update` 0x1ea398 sorts the contacts by distance. `Bullet_CollisionHandler` 0x12ce80
  ends the flight (state 2) on the first contact without the pass bit (:117-179). The pass bit is
  HITDATA+0x51 bit 1, set by `Collide_Intersect` 0x1ebaa8 for a glist node flagged 0x40 or a
  surface byte with 0x40/0x80 (:93-97, :143-144, :304-310). Glass (surface 0x10) also passes for a
  record with `+0x6c & 0x800` clear (:136-139). For the claw ids the handler skips its own
  0x40/0x80 material re-test (:145-160). That changes nothing, because those surfaces already
  carry bit 1.
- `Bullet_Delete` 0x12e660:42-55 latches only on state 2 with the head contact's object of class
  'Q'. A wall, floor or ceiling hit first means no wire.

This is exactly the player rounds' existing `segment_flight_scoped` trace. The port now runs that
trace along the bolt's path, from the gun's spawn point and the start cel. The claw latches the
point whose window (sphere radius + the existing bolt radius 1.0) the path enters first, and only
when that entry comes before the world stop (`gadget::pick_grapple_point_walled`,
`grapple_bolt_world_stop`, `Sim::claw_bolt_ray` / `Sim::claw_latch`). Remote online bodies use the
same test from the head.

## Why the earlier wall check broke the SkyBridge latch

It traced from the head to the point's centre. The bolt flies along the aim, not to the centre.
That made no difference for 07000014, though. The pinned stance was not legal:

- The stance (40.8, 2.40, 65.1) is inside the west duct. Its roof sits at 3.86 over a 2.40 floor.
  The test forced `crouched = false`, but a real crouch press does not stand Bond up there (the
  headroom check refuses).
- The duct's east wall at x 41.53 (mat 5, floor to roof, z 63.6..66+) stands between the duct and
  the point (45.29, 5.10, 62.94). A sweep of 21 yaws × 91 pitches from the crouched head finds no
  aim that enters the point's window before the world. The old latch went through that wall.
- From a clear stance the latch works: on the east walkway (45.0, 2.38, 64.5) almost every aim
  latches, and the reel reaches the hang.

The same through-geometry latch sat under four more pins:
- 0700000c's pulley test fired from the level start, 42 u out, through a wall 5.2 u away.
- 0700000d's dock stance fired through the crane deck's side.
- 07000011's office stance fired through the ceiling.
- 07000012's parapet stance stands under a beam 0.42 above the head.

Each now uses a stance with a clear line, found by scanning stances with the walled pick.

## Tests

- New, red on the old latch and green here (checked by swapping the latch back to the bare pick):
  - `interactables_07000014_live::the_qclaw_aimed_through_the_duct_wall_does_not_latch`: walked
    from the spawn, real crouch, aimed at the point, no wire. Sister: the bare pick takes the aim.
  - `interact_0700000c_live::the_q_claw_aimed_from_the_start_meets_the_wall`: the old pulley
    shot from the start, no wire. The wall sits within 10 u.
- New, green: `interactables_07000014_live::the_qclaw_latches_the_skybridge_point_from_a_clear_stance`
  (latch, reel and hang), plus two gadget unit tests (the entry vs the world stop, the trace in units).
- Restanced (they encoded through-geometry latches): the 0c pulley, 0d crane pulley, 11 SkyBridge
  and 12 SkyBridge ride tests.
- walkreal_07000014: the SkyBridge crossing is now a documented stand-in
  (`the_crossing_stand_in`). The duct leg is walked, then Bond is placed at the old hang and the
  old release tail runs. The #626 shaft shot is tried on real input first, then falls back to a
  latch stand-in: from the segment the bolt start meets the segment's south wall top 0.3 u out. 7
  pass, 3 ignored. The two `real_damage` walks are newly ignored with the measured reason (the
  shifted clock: the pit-band jump and the descent landing).
- Green here: gadget lib grapple tests 9/0, fix_wallhug_prompt 2, gadget_grapple 1,
  gadget_grapple_user_spot 2, interact_07000005 11, interact_0700000c 10, interactables_0700000d 11,
  interactables_07000011 16, interactables_07000014 11, interactables_07000041 18, qclaw_live 3,
  qclaw_ride_07000012 1, walkreal_07000005 14, walkreal_07000014 7/0/3i, walkreal_celprobe 2.

## Unproven / open

- How the game's Bond gets from the 07000014 west duct to the trench. The east wall blocks the
  claw, and Bond cannot stand in the duct. Question for a live check: from the vent start, which
  way out of the duct reaches the trench, and from where does the SkyBridge point #658 get clawed?
- The #626 shot from the stairwell segment depends on where the bolt starts. The port's view-model
  datum 0 pose decides that, and it has not been compared live for the claw.
- Still not ported: the latch is instant, while the game's bolt flies at `+0x20` per tick. The
  latch reads the raw look, not the auto-aimed direction (`Player_GetAimingPoint`). The reel
  targets the point's centre, while `Player_SetGrapplePoint` 0x195da0 stores the bolt's contact
  position. The bolt window radius of 1.0 is an existing claim (`obj+0xe8` is the object scale
  word) and was not re-derived here. A non-glass breakable in the path does not stop the claw yet.
