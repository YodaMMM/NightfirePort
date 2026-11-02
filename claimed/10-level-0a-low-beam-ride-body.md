# level-0a-low-beam-ride-body — one contact body per player state

Branch from origin/master; start from origin/s/level-0a-low-beam-standing (merge it in: it carries `cylgeom_hits_tris` and the full derivation in its report).

That branch proved the game's standing body (Player_Collision 0x194df0 default arm + Intersect_CylGeom 0x109da0) fires 0700000a's low beam #621 — `walkreal_0700000a_live::low_beam_621_fires_standing` passes — but the same swap stalls `fix_truck_live` (07000005's ch-0x70 park plate stays silent standing and fires crouched in the game).

Do:
1. Take Player_Collision's per-substate arms literally [M 0x194df0 cases 0/3/6/default]: the truck ride is not the walking substate. Find which arm the player is in while riding/attached on 07000005 and which on the 0a corridor, and build the trigger toucher from the arm for the player's CURRENT substate.
2. Both must hold together: #621 fires standing on 0a, the 05 park plate stays silent standing and fires crouched, the truck chain runs.
3. Un-ignore low_beam_621_fires_standing. Run walkreal_0700000a_live, fix_truck_live, every interactables_*_live and walkreal_*_live, then the full suite.

---
claimed-by: sandbox4 at 2026-11-02T07:49Z
