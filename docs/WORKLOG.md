## bullet-pass-water
Started 2026-11-06 18:22 UTC. Branch s/bullet-pass-water (cut from origin/master d0587e2b5).
Task: segment_surface_bullet returns no 0xE hit on real levels (02: 0 against 1377 for the
plain query), so Geo::bullet_pass treats every water plane as pass. A ray with flags 0xc keeps
those planes in the game: the feet ray reads byte 0xE at 07000002 (-79, 28). Read Collide_
Intersect 0x1ebaa8:311,317 and the node bit behind `bVar7 & 2`, then find what bullet_pass
reads and whether water really carries it. List every caller of the `_bullet` queries and say
for each whether the game's flags would see water. Check what a bullet fired into water does
in the game: a splash, a stop or a pass. Fix each difference with a test on real level data,
proved red on the old code.

Progress log:
- Sandbox reused (repo + work/ + ISO present); branch cut and pushed; WORKLOG started.
- The game chain read whole [M]: Collide_Intersect 0x1ebaa8 (the pass byte = HITDATA+0x51, memset 0
  at alloc; set by cel+0x94 bit 0x40 / obj+0xf0 bit 0x40 / the surface byte's 0x40 AND 0x80 — the
  0x80 `movn` is in a beq delay slot, unguarded, disproving the old port comment; enforced by the
  intersect mask's bit 0x8 only), Collide_Filter 0x1ed888, Intersect_RayGeom 0x10a8c8 (the raw byte
  to +0x50), Collide_Pick 0x1eb320, parsemap 0x1d1998 (cel+0x94 = the entity-params word at +4),
  Bullet_update 0x12e7b0 (the flight's +0x98 = 0, +0x9a = 0x20 — nothing drops) and
  Bullet_CollisionHandler 0x12ce80 (the per-contact pass walk: pass → the Effect_Create burst, no
  ricochet — Effect_RicochetProb 0x1cf290 returns before its draw at prob id 0 — and the flight
  re-arms; solid → state 2).
- Measured on the 02 disc data: the watersurface/southwater placements' glist flags 0x241 (bit 0x40
  SET) on every copy, bathhousewater 0x201, the water raw byte 0x0E everywhere; 1054 pass-flagged
  water tris the port's bullet queries dropped. The [L] "feet ray reads byte 0xE" re-attributed:
  the feet ray's intersect mask is 0x8 (0x1941f8) — it drops the plane; the 0xE byte is the ground
  record's, written by the movement capsule via Player_DealWithObjHit 0x194458.
- The answer: a bullet into the 0x241 water SPLASHES at the surface and PASSES, stopping at the
  submerged floor; into the 0x201 bathhousewater it STOPS at the surface.
- The fix shipped: segment_flight_contacts(_scoped) return the pass contacts beside the stop;
  BulletEvent::World carries the splash sites; the consumer spawns their Effect_Create treatment
  first; the ricochet draw skipped at mask 0 (the old port drew an RNG draw the game never makes).
  bullet_pass's doc corrected; the flight-trace comment corrected.
- Tests: the real-02 splash test green (the mask-8 ray stops at the floor, the flight carries the
  y-3.765 splash + the floor stop; through Bullet::update the event holds both). nf-game lib
  687/687; fix_foliage_bullets_live 3/3; bullet_range_live 3/3. The walkreal family not run whole
  (the deadline) — listed in the report's "what's left". Report: docs/reports/bullet-pass-water.md.
