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

### level-0d-c5-door-laser outcome (2026-10-28)

- Ported: the Explode_Propagate case-0x21 0x20 gate [M 0x17c518:170] (Targets.destroys carries
  tag+0x14 now), the Collide_FilterBullets weapon-word mask in the destroy bullet path
  [M 0x1ece30:26-33; D words 0x0001/0x0008/0x0020 dropped, 0x0010 mounted kept], the hit-colour
  pair obj+0x104/0x105 (−5 on a 0x67 contact tick, +1 recover, death freeze [M 0x12bd48:61-95]),
  the destroy leaves as solid bullet stops (one 50-damage contact per round [L ~50/frame]), the
  merged gun-door test corrected (its one-contact smash contradicted [L] + [M]).
- asm/decomp findings: Destroy_Smash's SetGList arg = ext+0x00 = the LOD pointer or the
  create-time fallback pCelList+idx·0x50+0x50 [M 0x12bbb8 asm, 0x12c318:23-26, 0x1d1b68] — the
  brief's glist+0x50 destroyed entry; the C5 leaves have no block-0x27 LOD [D probe].
- Left: the sustained-fire harness stalls after ~5 landed contacts (heat/trigger quirk — beam
  up, no contacts); the drain test is #[ignore]d with the reason. The draw-side destroyed-glist
  hide needs a per-frame static-instance hook (the 0x3b gap's shape). p6>0 deferred channel
  writes unported (the C5 doors are p6=0).
