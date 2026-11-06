# WORKLOG

## level-14-float-narrow
Started 2026-11-06 11:21 UTC. Branch s/level-14-float-narrow (from origin/s/level-14-float-on-master
80f472e88, the hover fix + probe14_pocket_settle on current master d6245281a). Task: "14 hover: a
narrower fix" — the broad suite failure of the spine-fallback push means it does real work; measure
where the small up-facing fallback push fires on a lift and on an 11 stair flight and why those need
it while the 14 pocket does not (cite the game; the port capsule is a stand-in, so the push may stand
in for the game's floor band or Player_FeetOnPoint grounding). Prefer a fix to what stops Bond
grounding at the pocket (walk floor band missing by an ulp: dy 0.65 against cc + 0.1, candidate).
First run: fix_autoaim_elevator_live, walkreal_07000011_route_live, mp_net_slots_post_move_live,
mp_perf_16_live, fix_npc_doors_live, probe14_pocket_settle — all must pass with MP digests unmoved;
then the walkreal set whole. No digest or walk re-pins to absorb the change.
  Measured (2026-11-06 12:30 UTC): A/B'd the hold skip behind a scratch env gate. Master
  behaviour (the fallback push everywhere): elevator PASS, 11 route PASS, pocket FAIL (the
  dither: grounded tick at dy 0.64727783, the spine push lifts +0.00272, dy 0.6500001 =
  0x3f266668, band FALSE — one ulp outside cc+0.1 = 0x3f266667 — hist never reaches 0xf).
  Hold skip on: pocket PASS but elevator FAIL (the boarding dither at dy 1.145 outside the
  standing band edge 1.1347) and 11 route FAIL (flight C stalls y 32.5025 > 32.5). The push
  is the ride holder: on the standing capsule its r-equilibrium (dy 1.032) sits INSIDE the
  band; at the crouched pocket it parks 1 ulp outside. Narrow fix: REVERTED the hold skip
  (master collision restored) and fixed the feet ray's hit distance instead — the game's
  Intersect_RayGeom t is the PLANE form [M 0x10a8c8:176-185, one division] while the port
  cast Möller–Trumbore; new segment_hit_scoped_feet (feet band only, 2 sites) returns
  dot(v0−from,n)/dot(d,n) on the hit triangle's plane. Trio re-measured: probe14 PASS,
  elevator PASS, 11 route PASS.
  Narrowed further (2026-11-06 12:56 UTC): the plane-form feet ray moved the MP digest
  (mp_perf_16 pinned digest differs) — any band-arithmetic change reaches the MP sims.
  Final shape: master's band/collision everywhere + the base branch's flat-hold skip
  CROUCH-GATED at the two walk call sites (self.cc == CROUCH_ROOT_HEIGHT). Gate re-run:
  probe14 PASS, elevator PASS, 11 route PASS, npc_doors PASS, mp_net_slots PASS;
  mp_perf_16's pinned digest STILL differs (the MP sim reaches the crouched hold regime —
  left for the next message). The plane-form feet ray lives in history 834ee5974 as the
  [M]-faithful alternative.
  Final tip (2026-11-06 12:50 UTC): the crouch gate ALSO broke the walks (master A/B:
  walkreal_07000004_live 10/0 green on master vs 8/2 with the gate; 01/05/06/0c/11/12/13/15/16
  reds of the same class — the crouched regime is common). Reverted the tip to master's
  collision behaviour: segment_hit_scoped_feet (the game's plane-form t [M 0x10a8c8:176-185])
  is PRESENT but INACTIVE; the only suite red vs master is probe14_pocket_settle itself (the
  hover the task targets). All four measured variants + the ledger are in the report; the
  next candidate is the contact-dist arithmetic vs Dist2Tri 0x109770 (the push equilibrium's
  1-ulp height, not the band).
