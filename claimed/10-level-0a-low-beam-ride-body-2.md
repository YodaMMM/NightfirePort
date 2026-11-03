# level-0a-low-beam-ride-body-2 — the standing beam fires (live-confirmed) without stalling 05's truck

Branch from origin/master; the previous attempt is on origin/s/level-0a-low-beam-ride-body (per-substate body arms, cylgeom_hits_tris).

Live fact [L 2026-11-02, fresh boot]: on 0700000a Bond walking STANDING across low beam #621's line (from (-12.4, 0.608, -31.0) south to z -33.78) fires ch 0x24 in the real game. So walkreal_0700000a_live::low_beam_621_fires_standing's expectation is the game's, and the port's standing toucher (the measured box pos-0.05..pos+1.26, r 0.55) is too high.

The blocker both earlier attempts hit: swapping in the game's standing body (Player_Collision 0x194df0 default arm + Intersect_CylGeom 0x109da0) stalls fix_truck_live (07000005's ch-0x70 park plate fires early / the truck chain never starts).

Do:
1. Find what the truck ride really is in the game: Bond's substate/attachment during the 05 truck ride and which Player_Collision arm (or none — the cut-scene's disabled window) runs then [M]. Build the trigger toucher per substate so the walking arm fires #621 and the ride matches the game.
2. Both green together: low_beam_621_fires_standing (un-ignored) and both fix_truck_live tests; every interactables_*_live and walkreal_*_live.
3. Then the full suite.
Rules: shared test helper changes (crates/nf-view/tests/common/) must be opt-in per level and re-run every walkreal_*_live.

---
claimed-by: sandbox1 at 2026-11-03T00:53Z
