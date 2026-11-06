# 14 hover: a narrower fix

`origin/s/level-14-float-on-master` is the hover fix on current master, plus a real test: `probe14_pocket_settle` now asserts Bond grounds, red without the fix. It must not merge as it is. The full suite failed broadly: lifts ("Bond settles onto the lift platform"), stair descents on 11, nav stalls, the 4-human and 16-player digests, and many walks. So the spine fallback's small pushes are doing real work; "a hold that never does work" was wrong.

Branch from it, push to `s/level-14-float-narrow`.

1. Measure where the small up-facing fallback push fires on a lift and on an 11 stair flight, and why those need it while the 14 pocket does not. Cite what the game does there. The port's capsule is a stand-in, so a push absent from `Intersect_CylGeom` 0x109da0:431-447 may be standing in for something else: the game's floor band, or `Player_FeetOnPoint`'s grounding.
2. Prefer a fix to what stops Bond grounding at the pocket: the walk's floor band missing by an ulp (dy 0.65 against cc + 0.1) is a candidate, and a fix there would touch far less than the push.
3. Before anything else, run `fix_autoaim_elevator_live`, `walkreal_07000011_route_live`, `mp_net_slots_post_move_live`, `mp_perf_16_live`, `fix_npc_doors_live` and `probe14_pocket_settle`. All must pass, with the MP digests unmoved. Then the walkreal set whole.

Do not re-pin digests or walks to absorb this change.
