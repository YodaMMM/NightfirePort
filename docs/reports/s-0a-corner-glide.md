# s/0a-corner-glide: the decompiled capsule push tail (0700000a west hall + corner)

Default behaviour is unchanged: the new tail is off unless `NF_CYL_TAIL=1` or
`nf_game::world::set_cyl_tail(true)`. Turning it on moves every walk (and so the shared random
stream); it is kept on this branch, measured on 0700000a only.

## What is proven

- [M disasm] Both `ASH_vecutil_Dist2Tri` 0x109770 calls in `Intersect_CylGeom` 0x109da0 set
  `t2 = s1 + 0x50` (0x10a2fc, 0x10a4f0; s1 is the HITTEST copy `Collide_Intersect` makes).
  t2 is Dist2Tri's 7th argument, the out-pointer it fills with the closest triangle point
  unless it returns flags 99. The push tail's `v = p - HITTEST+0x50` is therefore `p - q`
  against the triangle under test: push `(r - |v|) * v/|v|`, gated by `dot(v, n) > 0` and
  `|v| < r`. The older notes (a dead tail whose reference is the lifetime DeltaP sum) missed
  the register argument. The real +0x50 is still that sum (the add-back in Collide_Intersect),
  but the push never reads it.
- [M disasm] The head-on cull uses the raw spine B - A (stored at 0x109fa0, reloaded at
  0x10a1d0); the normalised copy feeds only the pick-bit block. It never fires for the walk
  spine (|B - A| = 0.7597).
- [M] A vertical face (|n . (B - A)| <= 0.0002) takes p from the triangle centroid and is not
  re-projected onto the spine (0x10a3f0-0x10a508), so its edge contacts carry a vertical part.
  Sloped faces re-project on an edge region (0x10a344-0x10a3e8).
- [M] Dist2Tri's region test is ported as written (`dist2tri_game`). It is not an exact
  closest point: the loop stops once bit 1 is set.

## Measured against the game (tail on)

`cyl_tail_0a_live::the_west_hall_climb_follows_the_game_trace` replays the live-0a-walk3 leg
on its fixed heading. Port compared with the game [L live-0a-walk3.csv]:

- wall stop: x -7.7540 vs -7.7545
- climb start: z 15.839 vs 15.844
- peak: 1.160 vs 1.182 (both over the band edge 1.1347)
- slide reaches the jamb's north edge: z 16.8288 in both
- slide height: 1.1346 vs 1.1324

On master the same leg never climbs (y stays 1.0891) and the test fails.

The walked glide line in `the_west_hall_glide_height_matches_the_game` crosses in 150 ticks to
(-9.28, 16.59). The game crosses in 155 ticks to (-9.27, 1.12, 16.60). The port reads y 1.1327
against the pinned 1.1285. That pin comes from a probe placed at 1.12, and placed starts vary
1.102-1.135 in the game [L live-0a-wallrest], so it stays ignored.

The corner placed at the game's start crosses on the game's own path with either model: after
60 ticks the port is at (-17.97, -0.42), the game at (-17.9715, -0.4162). The ring walk's wedge
comes from the approach, not from the corner. With the tail on, the vertical-face arm sinks Bond
to the floor rest (y 1.035) while he presses the opening door at (-19.5, 4.6). From that height
the 0.19 lip at the corner lifts him over the band and back in a loop.

## Not landed and why

With the tail on, two 0a tests go red:
- `walk_office_floor_qworm_keyboard_hatch_real_damage`: the committed input recording no
  longer matches (12919 ticks). This is an expected re-pin.
- `the_pick_ref_accumulator_tracks_the_game_ref`: the walk back east sinks to y 1.04 while
  rubbing the counter face at x -8.66 (a vertical face, the no-re-projection arm) and wedges
  at the jamb.

The other levels are unmeasured.

## Open live checks

1. While Bond presses the door at (-19.5, 4.6) as it opens, does his root y drop below 1.0891?
   If it does, the vertical-face arm is confirmed in play.
2. The same check along the counter face at x -8.66 (z 15.7 to 15.1).
