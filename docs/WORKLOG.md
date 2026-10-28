# WORKLOG

## level-0d-c5-door-laser (2026-10-28, branch s/level-0d-c5-door-laser)

Task: 07000d (Chain Reaction Part 2) C5 doors — class 0x21 Destroy objects
(placements #93 bigdoor01 / #96 bigdoor02, p0 ch 0x1f, p1 trigger 0x1f, p2 type
5/6, p3 health 1000/10000, p4 mask 0x10, p5 blast 10, p6 delay 0 [D]) — must
take beam damage over ~200 contact frames, not smash on first contact.

Plan:
- Port the 0x20 gate in Explode_Propagate case 0x21 [M 0x17c518], the
  Collide_FilterBullets 0x1ece30 ext+0x14 mask filter (player guns 0x08 blocked,
  mounted guns 0x10 pass), GunImp_Update 0x1320d8 -> Destroy_CollisionHandler
  beam damage (~50 hp/frame) + hit-colour (obj+0x104/0x105 -> 0, recover ~3/f),
  Destroy_Update colour/trigger read, Destroy_Smash 0x12bb60 (ext+0x17 latch,
  class 0x21->0x01, destroyed glist, light removal, 0x6000052 x3, channel
  write) and the same-frame channel smash of bigdoor01 [M].
- Correct the merged level-0d-gun-door-objective test that pins the
  one-contact smash.
- Tests on real input from the mount; full suite before push.
