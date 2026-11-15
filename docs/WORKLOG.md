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
