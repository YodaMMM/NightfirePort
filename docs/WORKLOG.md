# WORKLOG

## level-0d-operative-walk-10 (started 2026-11-15 ~00:58 UTC, branch s/level-0d-operative-walk-10)

- Claimed. Base: origin/master 820819cb3 (36db1e785, which carries rounds 7-9, is an
  ancestor). Task: 0700000d's Operative real-damage walk
  (`walkreal_0700000d_live::walk_arrival_to_the_c5_door_real_damage`) clears the ladder-top
  phase — the death is t4161 at (213.07, 17.37, -234.51), 10 hits for 100.0, the trio's
  first 64.0 (#29 32.0 at 3042/3222, #27 at 3398/3578) lands before the sweep opens ~3800.
  Round-10 lever from the driver + the main agent's 11-14 scans: answer #29/#27 before the
  climb, from a line the ground scans already found (the yard 186..216, -240..-222 scan:
  #27 63 clear spots, #29 66; the measured-dead dedicated dwells at (210,-237)/(204,-237)
  killed one then died on the open ground, so the answer must be fast/positioned, not a
  900-tick dwell). Real input only, no top-ups, never lower damage or health; helpers stay
  gated to the level + Operative so the plain replay and the d2 twin stay byte-identical.
  Judge each lever on its own real run; remove the ignore when green, else push the
  refreshed ledger + next lever. Then run the whole walkreal_0700000d_live binary and
  interactables_0700000d_live.
- (00:58) Branch cut from origin/master 820819cb3, WORKLOG started, first checkpoint.
- (02:05) Recon: probe_0d_ray_stands (fresh) — every ground stand's ray to the trio's load
  placements dies 0.83-0.90 (the slab edge from below). probe_0d_ground_wake (new): wake at
  the drop point + 700-tick settle, then (a) the 14 route-stand matrix, (b) the full
  186..216 x -240..-222 grid at 2 u. Result: the settled roof snipers carry HEALTH 10 (one
  aimed hit each); the CLEAR ground lines exist ONLY to g29 (56 spots: x 204-216, z
  -234..-240, trench a (210,-240) d 59 stable across rounds); g27 has ZERO ground spots
  (her south spawn's clear-ray law is the ladder-top wake dump only); g28 absent from the
  parked timeline (the run shows the strip-mouth #10 kill shot wakes her).
- (02:05) Run 1 (the lever's own real run): the trench-a leg walked +79 ticks, the act
  found the whole trio (g27 (151.35,-255.80), g28 (151.43,-241.25), g29 (151.39,-245.26),
  health 10 each), the ray to g29 full CLEAR, two shots, and #29 DIED pre-climb (hp 0 by
  frame 3854) with ZERO incoming during the shuttle+act (no hits before 3367 — the
  strip-mouth #10 kill also landed, #10 never fired). Death t5132 (the crouched baseline
  t5532 on this dice): the bill this roll is #27 x3 48.0 (her south spawn) + #29 16.0
  (pre-death) + #28 16.0. The ground-answer shape is proven safe; #27 is ground-unreachable.
- (02:08) Run 2 (+ the trench-f #28 attempt, cap 300): the trench-a act missed again (the
  identical deterministic shots), the ladder-top FIRST pass caught its guard at +222, #16's
  deck fire landed 32.0, the phase's later acts never ran — death t4336 WORSE. The walk-6
  act-fire dice law: every added pre-climb act re-rolls the stream lethal-earlier (5532 ->
  5132 -> 4336). THE VERDICT: the d-60 far-target MISS law (the aim gate's ray reads CLEAR,
  the bullets do not land — the walk-7 base-act law, the run-4379 graze law, the walk-8 #10
  audit family) — the ground lever family cannot kill the trio. THE LEGS ARE REMOVED (the
  #18-leg precedent); the route is the walk-9 shape again. The round-10 ledger + the next
  lever (the trench-a bullet audit with NF_0D_AUDIT=1 -> the port-side capsule refine, its
  own task) are in the ignore message; the walk stays ignored/red.
- (02:08) Final tree verified: walkreal_0700000d_live whole = 2 passed (the plain replay +
  the d2 agent twin, byte-identical) + 1 ignored (the ledger); interactables_0700000d_live
  = 11 passed 0 failed. The full four-crate suite did not fit the window (the sandbox lost
  cargo/7z to a reset and the build alone exceeded the remaining time); this branch touches
  only two nf-view test files (walkreal_0700000d_live.rs + the new probe_0d_ground_wake.rs)
  — no src/, no shared-helper changes.
- (02:11) Audit run (the reverted baseline + NF_0D_AUDIT=1, read-only): death t5532 EXACT
  (the walk-9 baseline restored deterministically — the revert verified), and the audit
  confirmed the ladder-top torso bullets LAND on the trio (the g29 crossing kill part
  Some(4,0.836)→hit, trv 52.2, world stop BEYOND her) — the far-target miss is specific to
  the GROUND up-hill lines (eye ~1.8 → y 23.0 at d 59-63); prime suspect the flight's
  START-CEL re-feed [M 0x12f0cc-0x12f0d8] on the steep multi-cel ray. The ignore message's
  next lever refined accordingly (the audit rides a probe-style leg, never a shipped leg).

## level-0d-operative-walk-11 (started 2026-11-15 ~03:22 UTC, branch s/level-0d-operative-walk-11)

- Claimed. Base: origin/master 92ee90c01; merged origin/s/level-0d-operative-walk-10
  (672082d57, not on master) — one conflict, the walk_arrival_to_the_c5_door_real_damage
  ignore message, resolved as a union: master's ledger (rounds to update 7, death t20383
  on the plate leg) + walk-10's round-10 findings appended verbatim (the d-60 far-target
  MISS law, the NF_0D_AUDIT addendum — the ladder-top torso bullets LAND, the miss is
  specific to the GROUND up-hill lines; suspects the flight's per-tick START-CEL re-feed
  [M Bullet_update 0x12f0cc-0x12f0d8] vs the port's boxes-only refine missing the
  origin-centred two-sphere capsule [M NDrone2_Collision 0x140e90:91-121]). Next lever per
  the walk-10 report + the merged ignore message: the trench-a bullet audit (probe-style,
  NF_0D_AUDIT=1, read-only), then the capsule fix if confirmed.
- (04:53) The trench-a bullet audit ANSWERED (NF_0D_AUDIT=1 + the posed-box dump, the
  NF_0D_AUDIT_CHEST probe leg): the bullets PASS THROUGH the guard (the world stop BEYOND her,
  no early stop) — the START-CEL re-feed is NOT the cause; the two-sphere capsule is NOT the
  hit volume [M Collide_Jointy 0x1ec180:104-158 = the same AnimGetCollData boxes the port
  parses; the spheres only in Drone_CollisionHandler 0x140968]. THE POSE LAW: the settled
  sniper state 0x93 poses the torso at origin −0.76 (head −0.60) — the chest aim (origin +0.3)
  flies over the body; state 0x94 stands and still misses — the shot lands +0.010-0.013 rad
  (~0.7 u) HIGH on EVERY trench shot across five runs, aim-systematic, independent of the
  measure origin. PRIME SUSPECT the free_aim aim_offset residual [M scene.rs:4554-4560]. The
  box-act lever wired (TORSO_FIRST/SETTLE_TAPS opt-ins, torso-first, settled, the box-fit
  tolerances) and measured dead on its own runs. walkreal whole: 2 passed 0 failed 1 ignored;
  interactables: 11 passed 0 failed. The round-11 ledger + the aim-offset next lever are in
  the ignore message. Report: docs/reports/level-0d-operative-walk-11.md.
