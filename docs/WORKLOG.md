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
