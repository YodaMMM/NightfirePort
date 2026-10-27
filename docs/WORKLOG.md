
---
rigid-body-wiring (branch s/rigid-body-wiring, worker 2026-10-27)
Progress:
- Branch from origin/master (ec917093) + pushed. Sandbox was reset: repo/ISO/work re-fetched
  (work-slim extracted, NF_ISO set, rust 1.99.1). Reading the remainder report's "What's
  left" (dispatch wiring, RestingContact/Rot/CalcTorque, EulerEvolve tail, wind census) and
  the port's load/tick/collide/draw paths next.
- WIRED (the window's deliverable): Rbodies::create (RB_Create 0x1d6bf0 exact), the kind-7
  control-walk tick (RB_Update 0x1d6e38 at the 0x26f9c8 dispatch), the spawn_blast class-5
  arming through RB_CollisionHandler 0x1d7a80 (RestingContact 0x1d7168 / Rot 0x1d73a0 per-type
  masks), CalcTorque 0x1d7658 + Vec_MakeOrthonormalBasis 0x1e4b78 ported, the obj+0x90
  matrix() accessor + the set_dynamic parts (bit 43). Live tests (07000006, NF_ISO): the
  resting barrels bit-identical over 600 ticks, the off-centre blast moving the matrix by the
  arm (the routing pinned against a replayed rb_explosion_hit_arm + rb_update). The [S]
  off-axis contact-point stand-in is documented (axis-aligned r gives zero torque; the game's
  gas volume tips). nf-game lib 578/0 (-j1; -j2's two sdk disc failures = the sandbox's
  concurrent-ISO limit, pass in isolation). The full suite did NOT fit (sandbox reset ate
  ~40 min of re-setup) — next window runs it chunked. Report:
  docs/reports/rigid-body-wiring.md.
