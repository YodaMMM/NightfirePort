### level-0a-band-settle-8 — start (2026-10-27 ~08:15 UTC)

Branch `s/level-0a-band-settle-8` created from `origin/master` (dcacdf1e, the batch that
carries settle-6-c). Task: the walk's vertical from the game's feet band. Start from the
settle-6-c state (the radial-tail deltas reverted, D2 behind the off-by-default NF_CAPS_D2
gate) and the settle-7 report. Their finding: the port's capsule tail (dir = v̂, mag = r−|v|,
unconditional) is carrying Bond's vertical settle, which the game's tail almost never does
(its operands are bounded by the HITTEST+0x50 accumulator arm [M Intersect_CylGeom 0x109da0:
421-447]); the game's walk vertical is the feet band [M Player_FeetOnPoint 0x194188: the ray
down +0.6, grounded at hitdist <= +0.1] plus the walked dy (settle-7's remaining candidate:
the walk matrix's pitch-y [M Player_SSWalk 0x192310 tail, 0x193600:284-295]).

Plan:
- Port the feet band as the y-settle authority.
- Re-land the [M]-faithful tail deltas (D2 at least) without lifting Bond anywhere:
  interactables_07000015 `the_dead_ladder_never_grabs_at_point_blank` must still settle
  y 40.2..40.7; the 21-test list from settle-6-c stays green.
- Un-ignore `walkreal_0700000a_live::the_west_hall_glide_height_matches_the_game` (and the
  other two 0a ignores if they cross) when green. Full suite green.

## level-0a-band-settle-8 — the walked dy landed (2026-10-27 ~09:35 UTC)

- [M 0x193600:284-295 + 0x192310:70-74] the walk tick's move add is the FULL matrix product
  (obj+0x30 += walk_matrix · local move; the walk matrix = obj+0x90 · RotMatrix(BL+0x30), the
  rate vector's x lane the view pitch) — the forward row's y = sin(pitch·π/2) rides into the
  position with the forward move. Ported: `pos[1] += look()[1]·vf` in the walk branch (the same
  term the swim branch rides). The harness: the walker walks LEVEL (eases the look pitch to 0 on
  walk legs — the door-use/aim legs pitch the view and used to leave it).
- Measured green: interactables_07000015 12/12 (the settle y 40.5 in 40.2..40.7), 01 3/3 (the
  resupply pocket holds), 09 3/3 (the tower walk crosses).
- D2 NOT re-landed: measured this round — on the substitute tail the flags-99 skip re-enters the
  settle-6-b lift (the probe: (85.08,40.4,−2.4) settles 41.315 — 6-c's verdict reproduced). The
  band-grounded anti-lift that would enable it (kill the up-facing faces' lift component while
  band_pre passes; the 0.7 threshold the pick-bit's dot1 [M 0x10a650]) measured 15 12/12 + 01 3/3
  but wedged 09's tower approach — held for the stair/slope live evidence. The sister's stand
  re-pin under the anti-lift was also prepared and reverted with it.
- The 0a trio stays ignored: the west-hall pin (the settled 1.1285 [L]) needs the glide's look
  trace (settle-7's live check 1, unanswered) — the walker walks level, the dy 0, the port rest
  stays 1.1201.
