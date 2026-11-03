
## level-0a-low-beam-ride-body-2 (2026-11-03)

Task: the standing beam fires (live-confirmed) without stalling 05's truck. [L 2026-11-02
fresh boot] on 0700000a Bond walking STANDING across low beam #621's line ((-12.4, 0.608,
-31.0) south to z -33.78) fires ch 0x24 in the real game — walkreal_0700000a_live::
low_beam_621_fires_standing's expectation is the game's; the port's standing toucher (the
measured box pos-0.05..pos+1.26, r 0.55) is too high. Both earlier attempts stalled
fix_truck_live when swapping in the game's standing body (Player_Collision 0x194df0 default
arm + Intersect_CylGeom 0x109da0) — 07000005's ch-0x70 park plate fired early / the truck
chain never started. Do: (1) find Bond's substate/attachment during the 05 truck ride and
which Player_Collision arm (or none — the cut-scene's disabled window) runs then [M]; build
the trigger toucher per substate; (2) both green together (low_beam_621_fires_standing
un-ignored + both fix_truck_live tests) and every interactables_*_live + walkreal_*_live;
(3) the full suite. Shared helper changes opt-in per level, every walkreal re-run.

Branch cut from origin/master 6f16146e. Sandbox carried from the previous session; the
pending level-12 finish steps (queue attach + ntfy-failed report) closed first.
