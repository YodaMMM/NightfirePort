
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
