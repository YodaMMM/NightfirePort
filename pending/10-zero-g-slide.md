# Zero-g: thrust into a sloped face slides in the game, freezes in the port

Branch from current `origin/master`, push to `s/zero-g-slide`. On Equinox a player thrusting against a sloped underside stops dead in the port.

Measured in PCSX2 on 0700001b, Bond put at (44.63, -8.64, 1.73), the pocket under the bank at the console 7/8 end:
- up thrust held 2.4 s: (44.63, -8.64) -> (44.33, -8.59) -> (44.02, -8.53) -> (43.72, -8.48), z unchanged. He slides along the underside at about 0.5 u/s.
- down thrust held: -8.64 -> -10.39 -> -12.19 -> -14.04.
Port, same spot (docs/reports/1b-zero-g-floor.md on `s/1b-zero-g-floor`): position held to +/-0.05 through 3000+ ticks of full climb.

Find what the game does to the move at that face: the report says it clips the move, which slides, where the port's radial push cancels it [M Intersect_CylGeom 0x109da0:431-447; check this, do not take it as given]. Fix it for the zero-g substates only. The walk, climb and pick paths must stay byte-identical: a broad collision change failed the suite 31 times on `s/level-14-float-on-master`.

Test on the real level, proved red on the old code: from that spot, hold lift, and assert the same slide. Gate on `walkreal_0700001b_live`, `mp_perf_16_live`, `mp_net_slots_post_move_live`. Zero-g feel is hands-on, so say plainly if the speed differs from the numbers above.
