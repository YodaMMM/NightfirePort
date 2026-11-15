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
