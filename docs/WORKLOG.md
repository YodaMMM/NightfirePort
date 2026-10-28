# WORKLOG

## level-14-guards-at-locked-door (2026-10-28, branch s/level-14-guards-at-locked-door)

Task: 07000014 (Countdown Part 1) — the guards outside the sabotage room. Once
both west fuseboxes are cut (ch 0x87/0x88 -> AND #717 -> 0x8C), the guard
spawners #720/#1333 start on 0x8C [D]. On the port a player hears guards
trying to open a door outside the room for the whole sequence, and they never
come through. Find what the game's spawned guards do [M the spawner's spawn,
their scripted/route states, the door-use path for a drone at a locked door
(Door_Activate by a drone, the lock channel 0x85 and the hatch doors), their
barks] — wait at the locked door until the cascade opens it, take another
route, or come in at once — and port the missing link. Real-input test from
the fusebox cuts: the guards spawn, their states/positions over time, the door
interaction, and what happens after the E-use cascade opens Door_Slide_11A/B.
Full suite, quote the total line.

Plan:
- Fast runner started in the background in the first minutes.
- [D] level 07000014 placements: find spawners #720/#1333, their spawn
  channels (0x8C), the guards' initial states/routes, the hatch doors and
  ch_lock 0x85.
- [M] the spawner (Guard/Actor spawn on channel), Door_Activate by a drone at
  a locked door, the bark path.
- Port the missing link; real-input test on top of the fusebox-cut +
  E-use-cascade path from s/level-14-delta-exit-walk.
- Full suite on the final tree; push; report; finish steps.

Progress (2026-10-28, first session): the chain pinned and three [M] fixes in.
- [D] spawners #720 (group 9) / #1333 (group 0xd), start 0x8C, kill 0x13; the
  template drones #721/#722/#723/#724/#783/#784/#785 (group 0xd) and
  #778/#779/#780/#781 (group 9), all p3 wait 0x8C; Door_Slide_11A/B #477/#478
  lock 0x85; 0x85's only writer is destroyable #729 (the generator cascade
  0x51 → #726 → #1113 → #727 → #1152 → #728 → #1154 → #730 → #1153 → #729).
- [M] DroneSpawner_Create 0x1378d0 / Init 0x137aa0 / Control 0x137e68 /
  SpawnDrone 0x137cb0; DSTATE_OpenDoor 0x1681f8 handles ONLY
  1/2/3/6-9/0x17-0x19/0xC/0x20/0x21 (ENTER sets +0x4F8 |= 0x10000; tick stays
  while the door is not state 4, then restores +0x5A6); the router 0x171df8
  re-routes unhandled msgs as msg 0 (dropped) and clears the bit on every
  change (0x171df8:185); ProcessDroneSight 0x176a28 (disasm 0x176c60 lui s5,1,
  tests 0x176ce8/0x176db8) never transitions a bit-set drone; CanThrowGrenade
  0x17a798 (count, Rand(2), opponent dist 8-20, anim 0x49); ThrowGrenade
  0x179558 (Bullet_init at Drone+0x1F0, pitch ±π/4 at 10/18);
  HandleExplosives 0x1435f0 (msg 0x21 only for a SEEN registered explosive).
- Port fixes: OpenDoor ENTER sets vision_on (the +0x4F8 bit); SCREAM/SHOUT
  drops for OpenDoor; the msg::EXPLOSIVE resume for OpenDoor is its own state.
- Measured (probe14_guards): the guards wait at the locked door in OpenDoor
  barking 1477 for 4500+ ticks, but the cascade still pulls them out ~30-40
  ticks after the E press (the remaining puller unpinned — the 0-dmg screen
  blasts at the door reaching HandleImpact case 9 is the prime suspect) and
  their own frags (thrown at the half-open leaf) kill the cluster. Not yet
  proven: the guards coming through. Next message: bisect the pull-out with
  the probe harness, then the real-input test.
