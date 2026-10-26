
## level-0a-band-settle

- (14:07 UTC, 10-26) Branch created from origin/master (7e35bd77). Task: un-ignore the three
  walkreal_0700000a_live tests that pin the doorway base climb's +0.0256 band settle [L 1.1147],
  the corner walk-through (y 1.127 -> 1.120 [L]) where the port wedges at the band edge 1.1347,
  and the ring walk (:227) wedge at 'the ai lane to the N-S corridor' — find the game's
  step/ground-settle rule on band edges (Player_Update ground probe + step-up [M]), port it,
  keep every other walkreal test green. Sandbox kept repo/ISO/work/target from the previous
  session; branch pushed as checkpoint.

- (15:35 UTC, 10-26) level-0a-band-settle SESSION: root-caused deeper, no code landed. New [M]
  evidence: Intersect_RayGeom 0x10a8c8:179 writes param_1[5] = from + dir·t (the ray hit point)
  into the WORKING COPY's +0x50 per crossing — the 0x201 branch's copy is discarded and the real
  +0x50 still only takes the Collide_Intersect DeltaP add-back, so hatch-10's accumulator identity
  stands (the ray write does not rescue the walk tail). [L] probe scripts: the player's BL+0x790
  boots (0,0,0) on 05 AND 0a (BL 0x1b78750 on 0a, obj 0x1b78640), and the ee_snapshot mid-play
  value (1.208, 9.084, −14.449) sits ~153 m from its owner — the walked tail is dead, reconfirmed.
  [L] floor probes: the whole ring route's floor is y≈0 (spawn cab, doorway, hall, corner, lane —
  no slab steps); the -2 report's per-tick wedge numbers re-read. Two experiments built and
  MEASURED, then reverted (01 live 1 fail / 05 live 6 fails — the below-B lift is load-bearing for
  stairs): (a) the ball tail with the stale floor-hit ref + the below-B displacement kill — the 0a
  corner CROSSED for the first time (but over budget) and the lift dither vanished, the lane leg
  still wall-stuck; (b) + the anti-motion wall projection — froze the doorway approach earlier.
  The three tests stay #[ignore] (updated notes pending the driver's live checks); the tree is
  master + docs only; walkreal_0700000a_live 4 passed / 3 ignored on the final tree.

## level-0a-band-settle-2

- (15:53 UTC, 10-26) Branch created from origin/s/level-0a-band-settle (f65c48f5), merged
  origin/master (82362142). Task: port the collision-handler tail (PC 0x194c70 =
  Player_CollisionHandler 0x1949a8 + 0x2c8, the live-proven wall stop [L 2026-10-26]) in place
  of the stand-in, re-land experiment (a)'s below-B kill if needed, re-pin + un-ignore the three
  ignored walkreal_0700000a_live tests to the driver's fresh [L] numbers (settled values), keep
  walkreal_07000001/05 green. Setup starting: work-slim + onfoot ISO downloads next.

- (17:40 UTC, 10-26) level-0a-band-settle-2 SESSION: the tail decoded (0x194c70 = obj+0x30 +=
  BL+0x750 [M disasm]; BL+0x750 = HITTEST+0x10, zeroed per tick at Collide_Update 0x1ea398:50,
  fed by the per-object DeltaP add-back 0x1ebaa8:353-373 — the port already has the shape at
  player.rs:1845). The per-face rule decoded (Intersect_CylGeom 0x109da0:421-470: v = q − ref,
  dot(v,n) > 0, |v| < radius, push = normalize(v)·(radius − |v|), _DeltaP accumulates, A/B/bounds
  shift); the ref = the lifetime accumulator whose boot (0,0,0) [L] contradicts the live 33
  tail writes — the ref's walked value is the open fact (live checks in the report). Two
  radial-dir ports built and measured, both reverted (85-tick corner cross with an up-lift to
  1.7773; horizontal radial re-wedges). The below-B kill re-landed GATED (NF_WALK_KILL, default
  off): 05 8/8 green, the 0a corner crosses (870 ticks, was a wedge), 01's contact walk
  regressed → gated until re-verified. The three 0a tests re-pinned to the settled [L] values
  (1.1285 / 1.1159), still ignored. Branch pushed; suite on the final tree started.
