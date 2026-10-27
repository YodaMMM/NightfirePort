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
